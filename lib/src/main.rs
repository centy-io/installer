use std::process::ExitCode;

fn main() -> ExitCode {
    let prerelease = std::env::args().any(|a| a == "--pre");

    match centy_installer::install(None, prerelease) {
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
