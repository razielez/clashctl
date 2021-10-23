use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::{ArgEnum, IntoApp, Parser};
use clap_generate::{generate, Shell};
use log::warn;

use crate::cli::{detect_shell, Opts};
use crate::Result;

#[derive(Parser, Debug)]
#[clap(about = "Generate auto-completion scripts")]
pub struct CompletionArg {
    #[clap(possible_values=&[
        "bash",
        "elvish",
        "fish",
        "powershell",
        "zsh",
    ])]
    pub shell: Option<Shell>,
    #[clap(
        short,
        long,
        about = "Output completion script to file, default to STDOUT"
    )]
    pub output: Option<PathBuf>,
}

impl CompletionArg {
    pub fn handle(&self) -> Result<()> {
        match self.shell.or_else(detect_shell) {
            None => {
                warn!("Shell not detected or it's not supported");
                warn!(
                    "Supported shells: {}",
                    Shell::value_variants()
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Some(shell) => {
                // let mut out: Box<dyn Write> = self
                //     .output
                //     .and_then(|x| File::open(x).ok())
                //     .or_else(|| std::io::stdout())
                //     .unwrap();
                let mut out: Box<dyn Write> = match self.output {
                    Some(ref dir) => Box::new(
                        File::create(&dir).expect(&format!("Unable to open {}", dir.display())),
                    ),
                    None => Box::new(std::io::stdout()),
                };
                generate(shell, &mut Opts::into_app(), "clashctl", &mut out);
            }
        }
        Ok(())
    }
}