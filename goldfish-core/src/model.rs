use std::{
    collections::{HashMap, HashSet},
    fmt,
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{bail, Result};
use derivative::Derivative;
use getset::Getters;
use rand::seq::SliceRandom;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum CardType {
    Artifact,
    Creature,
    Enchantment,
    Instant,
    Land,
    Planeswalker,
    Sorcery,
}

impl CardType {
    fn parse(s: &str) -> Result<Self> {
        // TODO: Be smarter about case.

        let card_type = match s {
            "artifact" => Self::Artifact,
            "creature" => Self::Creature,
            "enchantment" => Self::Enchantment,
            "instant" => Self::Instant,
            "land" => Self::Land,
            "planeswalker" => Self::Planeswalker,
            "sorcery" => Self::Sorcery,
            other => bail!("invalid card type: {}", s),
        };

        Ok(card_type)
    }

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
struct Card {
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
pub(crate) enum ZoneType {
    Battlefield,
    Deck,
    Exile,
    Graveyard,
    Hand,
}

impl ZoneType {
    fn name(&self) -> &str {
        match self {
            ZoneType::Battlefield => "battlefield",
            ZoneType::Deck => "deck",
            ZoneType::Exile => "exile",
            ZoneType::Graveyard => "graveyard",
            ZoneType::Hand => "hand",
        }
    }

    pub(crate) fn parse(location: &str) -> Result<Self> {
        let loc = match location {
            "battlefield" => Self::Battlefield,
            "deck" => Self::Deck,
            "exile" => Self::Exile,
            "graveyard" => Self::Graveyard,
            "hand" => Self::Hand,
            other => bail!("`{}` is not a known location", other),
        };

        Ok(loc)
    }
}

#[derive(Debug, Default)]
struct Zone {
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
pub(crate) enum Specifier {
    CardName(String),
    Index(usize),
}

#[derive(Derivative)]
#[derivative(Debug, Default)]
pub(crate) struct State {
    zones: HashMap<ZoneType, Zone>,
}

impl State {
    pub(crate) fn read_from_file(file: &str) -> Result<Self> {
        let file = File::open(file)?;
        let reader = BufReader::new(file);

        let mut cards = Vec::new();

        for line in reader.lines() {
            let original_line = line?;
            let parts: Vec<_> = original_line.split(':').map(str::trim).collect();

            if parts.is_empty() {
                continue;
            }

            if parts.len() > 2 {
                bail!(
                    "expected line in the form of 'Opt: instant', but got '{}'",
                    original_line
                );
            }

            let types: Result<_> = parts[1]
                .split(',')
                .map(|s| CardType::parse(s.trim()))
                .collect();

            let card = Card {
                name: parts[0].to_string(),
                types: types?,
            };

            cards.push(card);
        }

        let mut zones = HashMap::new();
        zones.insert(ZoneType::Deck, Zone { cards });

        Ok(Self { zones })
    }

    /// Moves all cards back to the deck, shuffles the deck, and draws seven cards.
    pub(crate) fn start_new_game(&mut self) -> Result<()> {
        let mut cards = Vec::new();

        for zone in self.zones.values_mut() {
            cards.append(&mut zone.cards);
        }

        self.get_zone(ZoneType::Deck).cards.extend(cards);
        self.shuffle();
        self.draw_n(7)?;

        Ok(())
    }

    /// Randomizes the order of the cards in the deck.
    pub(crate) fn shuffle(&mut self) {
        self.get_zone(ZoneType::Deck)
            .cards
            .shuffle(&mut rand::thread_rng());
    }

    /// Moves a card from one zone to another.
    pub(crate) fn move_card(
        &mut self,
        card: &Specifier,
        from: ZoneType,
        to: ZoneType,
    ) -> Result<()> {
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
    pub(crate) fn draw(&mut self) -> Result<()> {
        self.move_card(&Specifier::Index(0), ZoneType::Deck, ZoneType::Hand)
    }

    /// Draws `n` cards.
    pub(crate) fn draw_n(&mut self, n: usize) -> Result<()> {
        for _ in 0..n {
            self.draw()?;
        }

        Ok(())
    }

    /// Moves a permanent from the hand to the battlefield or a spell from the hand to the
    /// graveyard.
    pub(crate) fn play(&mut self, card: &Specifier) -> Result<()> {
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
    pub(crate) fn discard(&mut self, card: &Specifier) -> Result<()> {
        self.move_card(card, ZoneType::Hand, ZoneType::Graveyard)
    }

    /// Display the top `n` cards in the deck.
    pub(crate) fn inspect(&mut self, n: usize) -> Result<()> {
        todo!()
    }

    pub(crate) fn sacrifice(&mut self, card: &Specifier) -> Result<()> {
        self.move_card(card, ZoneType::Battlefield, ZoneType::Graveyard)
    }

    fn display_battlefield(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let creatures = self.display_battlefield_line(fmt, "creatures", &[CardType::Creature])?;
        let permanents = self.display_battlefield_line(
            fmt,
            "permanents",
            &[
                CardType::Artifact,
                CardType::Enchantment,
                CardType::Planeswalker,
            ],
        )?;
        let lands = self.display_battlefield_line(fmt, "lands", &[CardType::Land])?;

        if creatures || permanents || lands {
            writeln!(fmt)?;
        }

        Ok(())
    }

    fn display_battlefield_line(
        &self,
        fmt: &mut fmt::Formatter,
        line_name: &str,
        card_types: &[CardType],
    ) -> Result<bool, fmt::Error> {
        let battlefield = match self.zones.get(&ZoneType::Battlefield) {
            Some(zone) => zone,
            None => return Ok(false),
        };

        let cards = battlefield
            .cards
            .iter()
            .filter(|card| {
                card_types
                    .iter()
                    .any(|card_type| card.types.contains(&card_type))
            })
            .enumerate();

        let mut found = false;

        for (i, card) in cards {
            if i == 0 {
                write!(fmt, "    {}: ", line_name)?;
            } else {
                write!(fmt, ", ")?;
            }

            write!(fmt, "{}", card.name)?;
            found = true;
        }

        if found {
            writeln!(fmt)?;
        }

        Ok(found)
    }

    fn display_hand(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let hand = self
            .zones
            .get(&ZoneType::Hand)
            .map(|zone| zone.cards.as_slice())
            .unwrap_or_default();

        write!(fmt, "    hand: ")?;

        if hand.is_empty() {
            writeln!(fmt, "[no cards]")?;
            return Ok(());
        }

        let mut first = true;

        for card in hand {
            if !first {
                write!(fmt, ", ")?;
            }

            write!(fmt, "{}", card.name)?;
            first = false;
        }

        writeln!(fmt)
    }

    fn display_zone_count(&self, fmt: &mut fmt::Formatter, zone: ZoneType) -> fmt::Result {
        let count = self
            .zones
            .get(&zone)
            .map(|zone| zone.cards.len())
            .unwrap_or(0);
        writeln!(fmt, "    {}: [{} cards]", zone.name(), count)
    }
}

impl fmt::Display for State {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        writeln!(fmt, "battlefield:")?;
        self.display_battlefield(fmt)?;
        self.display_hand(fmt)?;
        self.display_zone_count(fmt, ZoneType::Deck)?;
        self.display_zone_count(fmt, ZoneType::Graveyard)?;
        self.display_zone_count(fmt, ZoneType::Exile)?;

        Ok(())
    }
}
