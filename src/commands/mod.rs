//! Command dispatch module.

pub mod accounts;
pub mod hashtag;
pub mod login;
pub mod profile;
pub mod search;
pub mod video;

use crate::cli::Command;
use crate::error::Result;
use crate::readme;

pub fn run(cmd: Command) -> Result<()> {
    match cmd {
        Command::Profile(args) => profile::run(args),
        Command::Hashtag(args) => hashtag::run(args),
        Command::Search(args) => search::run(args),
        Command::Video(args) => video::run(args),
        Command::Accounts(cmd) => accounts::run(cmd),
        Command::Login(args) => login::run(args),
        Command::AgentReadme => {
            readme::print();
            Ok(())
        }
    }
}
