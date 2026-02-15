use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let prerelease = args.iter().any(|a| a == "--pre");
    let restart = !args.iter().any(|a| a == "--no-restart");

    match centy_installer::install(None, prerelease, restart) {
        Ok(path) => {
            println!("{}", path.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
