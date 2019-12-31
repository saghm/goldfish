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

    pub fn exec(&mut self, command: &str) -> Result<()> {
        let statement = Input::new(command).parse()?;

        match statement {
            Statement::Nop => Ok(()),

            Statement::Bounce(card) => self.state.bounce(&card),
            Statement::Discard(card) => self.state.discard(&card),
            Statement::Draw(count) => self.state.draw_n(count),
            Statement::Fetch(card_name) => self.state.fetch(&card_name),
            Statement::Inspect(count) => self.state.inspect(count),
            Statement::Load(file) => self.load(&file),
            Statement::Move { card, from, to } => self.state.move_card(&card, from, to),
            Statement::Play(card) => self.state.play(&card),
            Statement::Restart => self.state.start_new_game(),
            Statement::Sacrifice(card) => self.state.sacrifice(&card),
            Statement::Tutor(card) => self.state.tutor(&card),
        }
    }
}
