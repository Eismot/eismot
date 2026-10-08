#![forbid(unsafe_code)]

use std::path::PathBuf;

use anyhow::{bail, ensure, Context, Result};
use eismot::build::{load, Generated};
use eismot::trivia::Manifest;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Command {
    Build,
    Check,
    Plan,
    CheckCi,
}

struct Arguments {
    command: Command,
    root: PathBuf,
    run: Option<String>,
}

impl Arguments {
    fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Option<Self>> {
        let mut arguments = arguments.into_iter();
        let command = match arguments.next().as_deref() {
            Some("build") => Command::Build,
            Some("check") | Some("--check") => Command::Check,
            Some("plan") | Some("--plan") => Command::Plan,
            Some("check-ci") => Command::CheckCi,
            Some("--help" | "-h") | None => {
                println!(
                    "eismot <build|check|plan|check-ci> [--run=NN|--pack=history] [--root=PATH]"
                );
                return Ok(None);
            }
            Some(other) => bail!("Unknown command: {other}; use --help"),
        };
        let mut parsed = Self {
            command,
            root: PathBuf::from("."),
            run: None,
        };
        let mut root_set = false;
        for argument in arguments {
            if let Some(root) = argument.strip_prefix("--root=") {
                ensure!(
                    !root_set && !root.is_empty(),
                    "Select one nonempty repository root"
                );
                root_set = true;
                parsed.root = PathBuf::from(root);
            } else {
                let run = if argument == "--pack=history" {
                    "02"
                } else {
                    argument
                        .strip_prefix("--run=")
                        .with_context(|| format!("Unknown argument: {argument}"))?
                };
                ensure!(parsed.run.is_none(), "Select at most one run");
                ensure!(
                    run.len() == 2 && run.bytes().all(|byte| byte.is_ascii_digit()),
                    "Run ID must contain two digits"
                );
                ensure!(
                    matches!(command, Command::Build | Command::Check),
                    "Only build/check accept run options"
                );
                parsed.run = Some(run.to_owned());
            }
        }
        Ok(Some(parsed))
    }
}

fn run() -> Result<()> {
    let Some(arguments) = Arguments::parse(std::env::args().skip(1))? else {
        return Ok(());
    };
    match arguments.command {
        Command::CheckCi => {
            let report = eismot::safety::check_repository(&arguments.root)?;
            println!(
                "Validated {} static SVGs, {} hardened workflows, and {} source URLs.",
                report.svg_files, report.workflows, report.source_urls
            );
        }
        Command::Plan => {
            let (bank, manifest) = load(&arguments.root)?;
            let planned = Manifest {
                runs: bank.plan(&manifest.seed)?,
                seed: manifest.seed,
            };
            println!("{}", serde_json::to_string_pretty(&planned)?);
        }
        Command::Build | Command::Check => {
            let generated = Generated::create(&arguments.root, arguments.run.as_deref())?;
            if arguments.command == Command::Check {
                generated.verify(&arguments.root)?;
            } else {
                generated.write(&arguments.root)?;
            }
            println!("{} {} runs, {} pool questions, {} game pages and {} HUD images; 15 rounds per run, 2 safety nets, 1 correct route per round, {} local links.",
                if arguments.command == Command::Check { "Verified" } else { "Generated and verified" }, generated.runs, generated.questions, generated.pages, generated.hud_images, generated.local_links);
        }
    }
    Ok(())
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(arguments: &[&str]) -> Result<Option<Arguments>> {
        Arguments::parse(arguments.iter().map(|argument| (*argument).to_owned()))
    }

    #[test]
    fn rejects_invalid_and_conflicting_arguments() {
        for arguments in [
            vec!["unknown"],
            vec!["check", "--run=1"],
            vec!["check", "--run=01", "--pack=history"],
            vec!["plan", "--run=01"],
            vec!["build", "--root="],
            vec!["check", "--other"],
        ] {
            assert!(parse(&arguments).is_err(), "{arguments:?}");
        }
        assert_eq!(
            parse(&["check", "--pack=history"])
                .unwrap()
                .unwrap()
                .run
                .as_deref(),
            Some("02")
        );
        assert!(parse(&["--help"]).unwrap().is_none());
    }
}
