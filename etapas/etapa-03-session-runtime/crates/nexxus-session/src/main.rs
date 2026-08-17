//! Command-line entry point for the Nexxus Session Runtime.
//!
//! Etapa 03 provides the orchestration engine and CLI contract. Concrete X11
//! and Wayland implementations belong to later stages; until one is integrated
//! the executable exits with an explicit backend-unavailable error rather than
//! selecting or fabricating another backend.

#![forbid(unsafe_code)]

use nexxus_backend_api::BackendKind;
use nexxus_core::NexxusPaths;
use nexxus_session::{SessionConfig, default_config_path, parse_backend};
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Debug, Default)]
struct Cli {
    backend: Option<BackendKind>,
    config: Option<PathBuf>,
    check: bool,
    help: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nexxus-session: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let cli = parse_cli(env::args().skip(1))?;
    if cli.help {
        print_help();
        return Ok(());
    }

    let paths = NexxusPaths::from_environment().map_err(|error| error.to_string())?;
    let runtime_dir = paths
        .prepare_runtime_dir()
        .map_err(|error| error.to_string())?;
    let config_path = cli
        .config
        .unwrap_or_else(|| default_config_path(&paths.config_dir()));
    let config = SessionConfig::load_optional(&config_path).map_err(|error| error.to_string())?;
    let backend = config
        .resolve_backend(cli.backend)
        .map_err(|error| error.to_string())?;

    if cli.check {
        println!("backend={}", backend_name(backend));
        println!("runtime_dir={}", runtime_dir.display());
        println!("control_socket={}", runtime_dir.join("session.sock").display());
        println!("config={}", config_path.display());
        println!("backend_integration=unavailable-until-backend-stage");
        return Ok(());
    }

    Err(format!(
        "selected backend '{}' is explicit but no concrete backend is integrated yet; Etapa 03 will not fall back silently",
        backend_name(backend)
    ))
}

fn parse_cli<I>(arguments: I) -> Result<Cli, String>
where
    I: IntoIterator<Item = String>,
{
    let mut cli = Cli::default();
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        if let Some(value) = argument.strip_prefix("--backend=") {
            cli.backend = Some(parse_backend(value).map_err(|error| error.to_string())?);
        } else if argument == "--backend" {
            let value = arguments
                .next()
                .ok_or_else(|| "--backend requires x11 or wayland".to_owned())?;
            cli.backend = Some(parse_backend(&value).map_err(|error| error.to_string())?);
        } else if let Some(value) = argument.strip_prefix("--config=") {
            cli.config = Some(PathBuf::from(value));
        } else if argument == "--config" {
            let value = arguments
                .next()
                .ok_or_else(|| "--config requires a path".to_owned())?;
            cli.config = Some(PathBuf::from(value));
        } else if argument == "--check" {
            cli.check = true;
        } else if matches!(argument.as_str(), "--help" | "-h") {
            cli.help = true;
        } else {
            return Err(format!("unknown argument '{argument}'"));
        }
    }
    Ok(cli)
}

fn backend_name(backend: BackendKind) -> &'static str {
    match backend {
        BackendKind::X11 => "x11",
        BackendKind::Wayland => "wayland",
    }
}

fn print_help() {
    println!("Nexxus Session Runtime 0.1.0");
    println!("usage: nexxus-session --backend=x11|wayland [--config PATH] [--check]");
    println!("  --backend   explicit graphics backend selection; no silent fallback");
    println!("  --config    optional session TOML path");
    println!("  --check     validate paths/configuration without starting modules");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_both_backend_argument_forms() {
        let cli = parse_cli(["--backend=x11".to_owned()]).unwrap();
        assert_eq!(cli.backend, Some(BackendKind::X11));
        let cli = parse_cli(["--backend".to_owned(), "wayland".to_owned()]).unwrap();
        assert_eq!(cli.backend, Some(BackendKind::Wayland));
    }

    #[test]
    fn rejects_unknown_arguments() {
        assert!(parse_cli(["--mystery".to_owned()]).is_err());
    }
}
