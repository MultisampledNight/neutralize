use std::{env, fs, process::ExitCode};

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let file_contents = fs::read_to_string(
        env::args()
            .skip(1)
            .next()
            .ok_or("This program takes one YAML basenext file as its first argument")?,
    )?;
    let scheme = neutralize::resolve_yaml(file_contents)?;
    println!("{}", scheme);

    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {}", err);
            ExitCode::FAILURE
        }
    }
}
