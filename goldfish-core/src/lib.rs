#![allow(dead_code, unused_variables)]

mod common;
mod parse;
mod state;

use anyhow::Result;

use common::Statement;
use parse::Input;
use state::State;

#[derive(Debug, Default)]
pub struct Goldfish {
    state: State,
}

fn new_state_from_file(file: &str) -> Result<State> {
    let mut state = State::read_from_file(file)?;
    state.start_new_game()?;

    Ok(state)
}

impl Goldfish {
    pub fn new(file: &str) -> Result<Self> {
        let state = new_state_from_file(file)?;

        Ok(Self { state })
    }

    pub fn load(&mut self, file: &str) -> Result<()> {
        std::mem::replace(&mut self.state, new_state_from_file(file)?);

        Ok(())
    }

    pub fn print_state(&mut self) {
        self.state.print();
    }

    pub fn print_help(&self) {
        println!("Input one of the following commands: ");
        println!("    `bounce <card name | $index>`      - move card from battlefield to hand");
        println!("    `discard <card name | $index>`     - move card from hand to graveyard");
        println!("    `draw [n]`                         - draw cards (default: 1)");
        println!("    `fetch <card name | $index>`       - play card from library");
        println!("    `help`                             - print this help message");
        println!("    `inspect [n]`                      - print top cards of deck (default: 1)");
        println!("    `load <file>`                      - load a new deck from the file");
        println!("    `move <card name | $index>         - move a card between locations");
        println!("       from <location> to <location>`  ");
        println!("    `play <card name | $index>`        - move a permanent from the hand to");
        println!("    `print`                            - print the current state of the game");
        println!("                                         battlefield or a spell from hand");
        println!("                                         graveyard");
        println!("    `restart`                          - restart the game");
        println!("    `sac <card name | $index>`         - move a card from battlefield to");
        println!("                                         graveyard");
        println!("    `shuffle`                          - shuffle the deck");
        println!("    `tutor <card name | $index>`       - move a card from the deck to hand");
    }

    pub fn exec(&mut self, command: &str) -> Result<bool> {
        let statement = Input::new(command).parse()?;
        let mut print_state = true;

        match statement {
            Statement::Nop => {
                print_state = false;
            }
            Statement::Help => {
                self.print_help();
                print_state = false;
            }

            Statement::Bounce(card) => self.state.bounce(&card)?,
            Statement::Discard(card) => self.state.discard(&card)?,
            Statement::Draw(count) => self.state.draw_n(count)?,
            Statement::Fetch(card_name) => self.state.fetch(&card_name)?,
            Statement::Inspect(count) => self.state.inspect(count),
            Statement::Load(file) => self.load(&file)?,
            Statement::Move { card, from, to } => self.state.move_card(&card, from, to)?,
            Statement::Play(card) => self.state.play(&card)?,
            Statement::Print => {
                // `print_state` is already true, so we do nothing.
            }
            Statement::Restart => self.state.start_new_game()?,
            Statement::Sacrifice(card) => self.state.sacrifice(&card)?,
            Statement::Shuffle => self.state.shuffle(),
            Statement::Tutor(card) => self.state.tutor(&card)?,
        };

        Ok(print_state)
    }
}
