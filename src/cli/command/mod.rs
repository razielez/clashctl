use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};

mod completion;
mod proxy;
mod server;

pub use completion::*;
use home::home_dir;
use log::debug;
pub use proxy::*;
pub use server::*;

use crate::cli::Config;
use crate::ui::TuiOpt;
use crate::{Clash, Error, Result};

#[derive(Parser, Debug)]
#[clap(
    name = clap::crate_name!(),
    author = clap::crate_authors!(),
    about = clap::crate_description!(),
    license = clap::crate_license!(),
    version = clap::crate_version!(),
)]
pub struct Opts {
    #[clap(subcommand)]
    pub cmd: Cmd,
    #[clap(flatten)]
    pub flag: Flags,
}

#[derive(Subcommand, Debug)]
pub enum Cmd {
    Tui(TuiOpt),
    #[clap(subcommand)]
    Proxy(ProxySubcommand),
    #[clap(subcommand)]
    Server(ServerSubcommand),
    #[clap(alias = "comp")]
    Completion(CompletionArg),
}

#[derive(Clone, Parser, Debug)]
pub struct Flags {
    #[clap(
        short,
        long,
        parse(from_occurrences),
        about = "Verbosity. Default: INFO, -v DEBUG, -vv TRACE"
    )]
    pub verbose: u8,
    #[clap(
        short,
        long,
        about = "Timeout of requests, in ms",
        default_value = "2000"
    )]
    pub timeout: u64,
    #[clap(
        short,
        long,
        about = "Path of config file. Default to ~/.config/clashctl/config.ron"
    )]
    pub config: Option<PathBuf>,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            verbose: 0,
            timeout: 2000,
            config: None,
        }
    }
}

impl Flags {
    pub fn get_config(&self) -> Result<Config> {
        let conf_file = self
            .config
            .to_owned()
            .or_else(|| home_dir().map(|dir| dir.join(".config/clashctl/config.ron")))
            .ok_or(Error::ConfigFileOpenError)?;

        if !conf_file.is_file() {
            return Err(Error::ConfigFileTypeError(conf_file));
        }

        if !conf_file.exists() {
            debug!("Config directory does not exist, creating.");
            std::fs::create_dir_all(&conf_file).map_err(Error::ConfigFileIoError)?;
        }
        debug!("Path to config: {}", conf_file.display());
        Config::from_dir(conf_file)
    }

    pub fn connect_server_from_config(&self) -> Result<Clash> {
        let config = self.get_config()?;
        let server = config
            .using_server()
            .ok_or(Error::ServerNotFound)?
            .to_owned();
        server.into_clash_with_timeout(Some(Duration::from_millis(self.timeout)))
    }
}
