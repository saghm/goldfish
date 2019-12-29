use goldfish_core::Goldfish;
use rustyline::{error::ReadlineError, Config, Editor};

fn main() {
    let mut goldfish = Goldfish::new();

    let config = Config::builder().auto_add_history(true).build();
    let mut prompt = Editor::<()>::with_config(config);

    let _ = prompt.load_history("goldfish_history");

    loop {
        let input = match prompt.readline("##> ") {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => panic!("{}", e),
        };

        if let Err(e) = goldfish.exec(&input) {
            eprintln!("Error: {}", e);
        }

        let _ = prompt.save_history("goldfish_history");
    }
}
