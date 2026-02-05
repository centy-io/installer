use std::process::ExitCode;

fn main() -> ExitCode {
    match centy_installer::install(None) {
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
