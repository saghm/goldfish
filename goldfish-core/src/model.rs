use std::collections::{HashMap, HashSet};

use anyhow::{bail, Result};
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

impl CardType {
    fn is_permanent(&self) -> bool {
        match self {
            CardType::Artifact
            | CardType::Creature
            | CardType::Enchantment
            | CardType::Land
            | CardType::Planeswalker => true,
            CardType::Instant | CardType::Sorcery => false,
        }
    }
}

#[derive(Debug, Getters)]
pub struct Card {
    #[get]
    types: HashSet<CardType>,

    #[get]
    name: String,
}

impl Card {
    fn is_named(&self, name: &str) -> bool {
        // TODO: Be smarter about case and spacing.
        self.name == name
    }

    fn is_permanent(&self) -> bool {
        self.types.iter().any(|card_type| card_type.is_permanent())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ZoneType {
    Battlefield,
    Deck,
    Exile,
    Graveyard,
    Hand,
}

#[derive(Debug, Default)]
pub struct Zone {
    cards: Vec<Card>,
}

impl Zone {
    fn remove_card(&mut self, card: &Specifier) -> Result<Card> {
        match card {
            Specifier::CardName(name) => self.remove_card_by_name(name),
            Specifier::Index(i) => self.remove_card_by_index(*i),
        }
    }

    fn remove_card_by_name(&mut self, name: &str) -> Result<Card> {
        for i in 0..self.cards.len() {
            if self.cards[i].name == name {
                return Ok(self.cards.remove(i));
            }
        }

        bail!("not found!");
    }

    fn remove_card_by_index(&mut self, i: usize) -> Result<Card> {
        if i >= self.cards.len() {
            bail!("not found!");
        }

        Ok(self.cards.remove(i))
    }
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
        if from == to {
            return Ok(());
        }

        let from_zone = self.get_zone(from);
        let card = from_zone.remove_card(card)?;

        let to_zone = self.get_zone(to);
        to_zone.cards.push(card);

        Ok(())
    }

    fn get_zone(&mut self, zone_type: ZoneType) -> &mut Zone {
        self.zones.entry(zone_type).or_insert_with(Default::default)
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
        let hand = self.get_zone(ZoneType::Hand);
        let card = hand.remove_card(card)?;

        if card.is_permanent() {
            let battlefield = self.get_zone(ZoneType::Battlefield);
            battlefield.cards.push(card);
        } else {
            let graveyard = self.get_zone(ZoneType::Graveyard);
            graveyard.cards.push(card);
        }

        Ok(())
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
