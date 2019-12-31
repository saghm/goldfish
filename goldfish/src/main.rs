use std::fs::OpenOptions;

use goldfish_core::Goldfish;
use rustyline::{error::ReadlineError, Config, Editor};
use structopt::{clap::AppSettings, StructOpt};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "goldfish",
    author,
    about,
    setting(AppSettings::ArgRequiredElseHelp)
)]
struct Opt {
    /// The deck list to use.
    file: String,
}

fn main() {
    let opt = Opt::from_args();
    let mut goldfish = Goldfish::new(&opt.file).unwrap();

    let config = Config::builder().auto_add_history(true).build();
    let mut prompt = Editor::<()>::with_config(config);

    let history_file = dirs::home_dir().map(|dir| {
        let file = dir.join(".goldfish_history");
        let _ = OpenOptions::new()
            .append(true)
            .create(true)
            .open(file.clone());

        file
    });

    let history_file_load_result = history_file
        .as_ref()
        .map(|file| prompt.load_history(file).map_err(|_| ()));

    if history_file_load_result != Some(Ok(())) {
        eprintln!(
            "WARNING: History file could not be loaded. Input history will not be enabled for \
             this session."
        );
    }

    loop {
        goldfish.print_state();

        let input = match prompt.readline("##> ") {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => panic!("{}", e),
        };

        if let Err(e) = goldfish.exec(&input) {
            eprintln!("Error: {}", e);
        }

        if let Some(Err(_)) = history_file.as_ref().map(|file| prompt.save_history(file)) {
            eprintln!("WARNING: The last command could not be saved into the history file.");
        }
    }
}
