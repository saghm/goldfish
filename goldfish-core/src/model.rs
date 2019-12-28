use std::collections::{HashMap, HashSet};

use anyhow::Result;
use getset::Getters;

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum CardType {
    Artifact,
    Creature,
    Enchantment,
    Instant,
    Land,
    Planeswalker,
    Sorcery,
}

#[derive(Debug, Getters)]
pub struct Card {
    #[get]
    types: HashSet<CardType>,

    #[get]
    name: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ZoneType {
    Battlefield,
    Deck,
    Exile,
    Graveyard,
    Hand,
}

#[derive(Debug)]
pub struct Zone {
    cards: Vec<Card>,
}

#[derive(Debug)]
pub enum Specifier {
    CardName(String),
    Index(usize),
}

#[derive(Debug, Getters)]
pub struct State {
    zones: HashMap<ZoneType, Zone>,
}

impl State {
    /// Moves a card from one zone to another.
    pub fn move_card(&mut self, card: &Specifier, from: ZoneType, to: ZoneType) -> Result<()> {
        todo!()
    }

    /// Draws a card.
    pub fn draw(&mut self) -> Result<()> {
        self.move_card(&Specifier::Index(0), ZoneType::Deck, ZoneType::Hand)
    }

    /// Draws `n` cards.
    pub fn draw_n(&mut self, n: usize) -> Result<()> {
        for _ in 0..n {
            self.draw()?;
        }

        Ok(())
    }

    /// Moves a permanent from the hand to the battlefield or a spell from the hand to the graveyard.
    pub fn play(&mut self, card: &Specifier) -> Result<()> {
        todo!()
    }

    /// Discards a card.
    pub fn discard(&mut self, card: &Specifier) -> Result<()> {
        self.move_card(card, ZoneType::Hand, ZoneType::Graveyard)
    }

    /// Display the top `n` cards in the deck.
    pub fn inspect(&mut self, n: usize) -> Result<()> {
        todo!()
    }
}
