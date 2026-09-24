//! `tiktok accounts add|list|test|remove`.

use std::io::{BufRead, IsTerminal, Write};

use serde_json::Value;

use crate::account::identity;
use crate::cli::AccountsCommand;
use crate::client::Client;
use crate::config::{self, AccountConfig};
use crate::error::{Error, ErrorCode, Result};
use crate::obj;
use crate::output;
use crate::secrets;

pub fn run(cmd: AccountsCommand) -> Result<()> {
    match cmd {
        AccountsCommand::Add { name, api_key, api_key_stdin, force, no_verify } => {
            let api_key = if api_key_stdin { Some(read_stdin_key()?) } else { api_key };
            add(name, api_key, force, no_verify)
        }
        AccountsCommand::List { check } => list(check),
        AccountsCommand::Test { name } => test(&name),
        AccountsCommand::Remove { name, yes } => remove(&name, yes),
    }
}

fn add(name: String, api_key: Option<String>, force: bool, no_verify: bool) -> Result<()> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(Error::invalid("An account name is required."));
    }

    let store = secrets::store()?;
    let config = config::load()?;

    let existing = config.find(&name).map(|(k, _)| k.clone());
    if let Some(existing) = &existing
        && !force
    {
        return Err(Error::invalid(format!("An account named '{existing}' already exists.")).fix(format!(
            "Pick a different name, or replace its key: tiktok accounts add {existing} --api-key <key> --force"
        )));
    }
    let name = existing.clone().unwrap_or(name);

    let key = match api_key.map(|k| k.trim().to_string()).filter(|k| !k.is_empty()) {
        Some(k) => k,
        None => prompt_key(&name)?,
    };

    let mut ident = String::new();
    if !no_verify {
        let client = Client::new(&key);
        ident = identity::probe(&client, &key);
    }

    {
        let _lock = config::lock()?;
        store.set(&secrets::account_key(&name), &key)?;

        let mut config = config::load()?;
        config.accounts.insert(name.clone(), AccountConfig { identity: ident.clone(), added_at: config::now_utc() });
        config::save(&config)?;
    }

    output::write(&obj! {
        "status" => if existing.is_none() { "added" } else { "replaced" },
        "name" => name,
        "identity" => ident,
        "verified" => !no_verify,
        "secretStore" => store.name(),
        "configDir" => config::config_dir().display().to_string(),
        "nextStep" => format!("tiktok profile nike --account {name}"),
    });
    Ok(())
}

fn list(check: bool) -> Result<()> {
    let config = config::load()?;
    let store = secrets::store()?;

    if config.accounts.is_empty() {
        output::write(&Value::Array(Vec::new()));
        return Ok(());
    }

    let mut items = Vec::new();
    for (name, acct) in config.sorted() {
        let mut row = serde_json::Map::new();
        row.insert("name".into(), Value::String(name.clone()));
        row.insert("identity".into(), Value::String(acct.identity.clone()));
        row.insert("addedAt".into(), Value::String(acct.added_at.clone()));
        row.insert("secretStore".into(), Value::String(store.name().into()));

        if check {
            let key = store.get(&secrets::account_key(name))?;
            let status = match key {
                Some(k) => {
                    let client = Client::new(&k);
                    match client.get("users/me") {
                        Ok(_) => "valid",
                        Err(e) if e.code == ErrorCode::AuthRequired => "rejected",
                        Err(_) => "unreachable",
                    }
                }
                None => "missing_key",
            };
            row.insert("status".into(), Value::String(status.into()));
        } else {
            row.insert("status".into(), Value::String("stored".into()));
        }

        items.push(Value::Object(row));
    }

    output::write(&Value::Array(items));
    Ok(())
}

fn test(name: &str) -> Result<()> {
    let config = config::load()?;
    let Some((stored_name, acct)) = config.find(name) else {
        return Err(Error::new(ErrorCode::NoAccount, format!("No account named '{name}'."))
            .detail(crate::account::describe(&config))
            .fix("tiktok accounts list"));
    };

    let store = secrets::store()?;
    let key = store.get(&secrets::account_key(stored_name))?;
    let Some(api_key) = key.filter(|k| !k.trim().is_empty()) else {
        return Err(Error::new(ErrorCode::AuthRequired, format!("Account '{stored_name}' has no stored API key."))
            .detail("The config entry exists but the keystore has nothing under it.")
            .fix(format!("tiktok accounts add {stored_name} --api-key <key>")));
    };

    let client = Client::new(&api_key);
    let probe_ident = identity::probe(&client, &api_key);

    output::write(&obj! {
        "status" => "valid",
        "name" => stored_name,
        "identity" => if probe_ident.is_empty() { acct.identity.clone() } else { probe_ident },
        "secretStore" => store.name(),
    });
    Ok(())
}

fn remove(name: &str, yes: bool) -> Result<()> {
    let config = config::load()?;
    let Some((stored_name, _)) = config.find(name) else {
        return Err(Error::new(ErrorCode::NoAccount, format!("No account named '{name}'."))
            .detail(crate::account::describe(&config))
            .fix("tiktok accounts list"));
    };
    let stored_name = stored_name.clone();

    if !yes && std::io::stdin().is_terminal() {
        eprint!("Remove account '{stored_name}' and its stored API key? [y/N]: ");
        let _ = std::io::stderr().flush();
        let mut reply = String::new();
        let _ = std::io::stdin().read_line(&mut reply);
        if !matches!(reply.trim().to_lowercase().as_str(), "y" | "yes") {
            return Err(Error::invalid("Aborted."));
        }
    }

    let store = secrets::store()?;
    {
        let _lock = config::lock()?;
        let _ = store.delete(&secrets::account_key(&stored_name));
        let mut config = config::load()?;
        config.accounts.shift_remove(&stored_name);
        config::save(&config)?;
    }

    output::write(&obj! {
        "status" => "removed",
        "name" => stored_name,
        "note" => "Key deleted from this machine. It has not been revoked on the Apify platform.",
    });
    Ok(())
}

fn read_stdin_key() -> Result<String> {
    let mut key = String::new();
    std::io::stdin().lock().read_line(&mut key).map_err(|e| Error::invalid(format!("Could not read stdin: {e}")))?;
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err(Error::invalid("--api-key-stdin was given but stdin was empty."));
    }
    Ok(key)
}

fn prompt_key(name: &str) -> Result<String> {
    if !std::io::stdin().is_terminal() {
        return Err(Error::invalid("No API key given and no terminal to prompt on.")
            .fix(format!("pbpaste | tiktok accounts add {name} --api-key-stdin")));
    }

    let key = rpassword::prompt_password(format!("Enter Apify API token for '{name}': "))
        .map_err(|e| Error::other("Could not read API token.").detail(e.to_string()))?;
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err(Error::invalid("API token cannot be empty."));
    }
    Ok(key)
}
