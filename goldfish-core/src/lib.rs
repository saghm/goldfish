#![allow(dead_code, unused_variables)]

mod ast;
mod model;
mod parse;

use anyhow::Result;

use ast::Statement;
use model::State;
use parse::Input;

#[derive(Debug, Default)]
pub struct Goldfish {
    state: State,
}

impl Goldfish {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn exec(&mut self, command: &str) -> Result<()> {
        let statement = Input::new(command).parse()?;

        match statement {
            Statement::Nop => Ok(()),

            Statement::Discard(card) => self.state.discard(&card),
            Statement::Draw(count) => self.state.draw_n(count),
            Statement::Inspect(count) => self.state.inspect(count),
            Statement::Move { card, from, to } => self.state.move_card(&card, from, to),
            Statement::Play(card) => self.state.play(&card),
        }
    }
}