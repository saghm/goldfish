use goldfish_core::Goldfish;
use rustyline::{error::ReadlineError, Editor};

fn main() {
    let mut goldfish = Goldfish::new();
    let mut prompt = Editor::<()>::new();

    loop {
        let input = match prompt.readline("##> ") {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
            Err(e) => panic!("{}", e),
        };

        if let Err(e) = goldfish.exec(&input) {
            eprintln!("Error: {}", e);
        }
    }
}
