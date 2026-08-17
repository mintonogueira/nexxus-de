//! Small executable used for package/post-install validation and manual smoke tests.

#![forbid(unsafe_code)]

use nexxus_backend_x11::X11Service;
use std::env;
use std::process::ExitCode;
use std::thread;
use std::time::Duration;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("nexxus-x11-backend-check: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut serve_seconds = 0_u64;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--check" => {}
            "--serve-seconds" => {
                let value = args.next().ok_or_else(|| "--serve-seconds requires an integer".to_owned())?;
                serve_seconds = value.parse().map_err(|_| "invalid --serve-seconds value".to_owned())?;
            }
            "--help" | "-h" => {
                println!("usage: nexxus-x11-backend-check [--check] [--serve-seconds N]");
                return Ok(());
            }
            _ => return Err(format!("unknown argument '{arg}'")),
        }
    }

    let mut service = X11Service::start(None).map_err(|error| error.to_string())?;
    let output = service.output();
    println!("backend=x11");
    println!("output={}x{}", output.width, output.height);
    println!("wm_claim=ok");
    println!("compositor=not-required");
    if serve_seconds > 0 {
        thread::sleep(Duration::from_secs(serve_seconds));
    }
    service.stop().map_err(|error| error.to_string())
}
