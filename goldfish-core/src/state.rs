mod card;

use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader},
    path::PathBuf,
};

use anyhow::{bail, Result};
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use scryfall::card::Card;

use self::card::CardExt;
use crate::common::{PrintTarget, Specifier, ZoneType};

lazy_static! {
    static ref GOLDFISH_DIR: Option<PathBuf> = dirs::home_dir().map(|path| path.join(".goldfish"));
    static ref GOLDFISH_CACHE_DIR: Option<PathBuf> =
        GOLDFISH_DIR.as_ref().map(|path| path.join("cache"));
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
            if self.cards[i].is_named(name) {
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

#[derive(Debug, Default)]
pub(crate) struct State {
    zones: HashMap<ZoneType, Zone>,
}

impl State {
    pub(crate) fn read_from_file(file: &str) -> Result<Self> {
        let file = File::open(file)?;
        let reader = BufReader::new(file);

        let mut cards = Vec::new();

        for (i, line) in reader.lines().enumerate() {
            let original_line = line?;
            let trimmed_line = original_line.trim();

            let mut parts = trimmed_line.splitn(2, char::is_whitespace);

            let first_part = match parts.next() {
                Some(s) if s.is_empty() || s.starts_with("//") || s.starts_with("SB:") => continue,
                Some(s) => s,
                None => continue,
            };

            let mut card_name = match parts.next() {
                Some(part) => part.trim(),
                None => bail!("missing card name on line {}", i + 1),
            };

            if card_name.starts_with('[') {
                if let Some(end_of_set) = card_name.find(']') {
                    if let Some(first_non_space_after_set) =
                        card_name[end_of_set + 1..].find(|c: char| !c.is_whitespace())
                    {
                        card_name = &card_name[end_of_set + 1 + first_non_space_after_set..];
                    }
                }
            }

            let count: usize = match first_part.parse() {
                Ok(count) => count,
                Err(..) => bail!("invalid card count for `{}`: {}", card_name, first_part),
            };

            let card = get_card_and_cache(card_name)?;

            for _ in 0..count {
                cards.push(card.clone());
            }
        }

        let mut zones = HashMap::new();
        zones.insert(ZoneType::Deck, Zone { cards });

        Ok(Self { zones })
    }

    fn get_zone(&mut self, zone_type: ZoneType) -> &mut Zone {
        self.zones.entry(zone_type).or_insert_with(Default::default)
    }

    fn play_card(&mut self, card: Card) -> Result<()> {
        if card.is_permanent() {
            let battlefield = self.get_zone(ZoneType::Battlefield);
            battlefield.cards.push(card);
        } else {
            let graveyard = self.get_zone(ZoneType::Graveyard);
            graveyard.cards.push(card);
        }

        Ok(())
    }

    pub(crate) fn bounce(&mut self, card: &Specifier) -> Result<()> {
        self.move_card(card, ZoneType::Battlefield, ZoneType::Hand)
    }

    /// Discards a card.
    pub(crate) fn discard(&mut self, card: &Specifier) -> Result<()> {
        self.move_card(card, ZoneType::Hand, ZoneType::Graveyard)
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

    pub(crate) fn exile(&mut self, card: &Specifier, from: ZoneType) -> Result<()> {
        self.move_card(card, from, ZoneType::Exile)
    }

    /// Plays a card from the deck. For permanents, this will move the card from the deck to the
    /// battlefield. For non-permanents, this will move the card from the deck to the graveyard.
    pub(crate) fn fetch(&mut self, card: &str) -> Result<()> {
        let card = self
            .get_zone(ZoneType::Deck)
            .remove_card(&Specifier::CardName(card.into()))?;

        self.play_card(card)?;
        self.shuffle();

        Ok(())
    }

    pub(crate) fn mill(&mut self, n: usize) -> Result<()> {
        for _ in 0..n {
            self.move_card(&Specifier::Index(0), ZoneType::Deck, ZoneType::Graveyard)?;
        }

        Ok(())
    }

    /// Moves a card from one zone to another.
    pub(crate) fn move_card(
        &mut self,
        card: &Specifier,
        from: ZoneType,
        to: ZoneType,
    ) -> Result<()> {
        // Allow cards to be moved from the deck to iself, since tucking is useful.
        if from == to && to != ZoneType::Deck {
            return Ok(());
        }

        let from_zone = self.get_zone(from);
        let card = from_zone.remove_card(card)?;

        if to == ZoneType::Battlefield && !card.is_permanent() {
            bail!(
                "cannot move {} to the battlefield because it isn't a permanent",
                card.name
            );
        }

        let to_zone = self.get_zone(to);
        to_zone.cards.push(card);

        Ok(())
    }

    /// Moves a permanent from the hand to the battlefield or a spell from the hand to the
    /// graveyard.
    pub(crate) fn play(&mut self, card: &Specifier) -> Result<()> {
        let hand = self.get_zone(ZoneType::Hand);
        let card = hand.remove_card(card)?;

        self.play_card(card)
    }

    /// Randomizes the order of the cards in the deck.
    pub(crate) fn shuffle(&mut self) {
        self.get_zone(ZoneType::Deck)
            .cards
            .shuffle(&mut rand::thread_rng());
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

    /// Display the top `n` cards in the deck.
    pub(crate) fn inspect(&mut self, n: usize) {
        let deck = self.get_zone(ZoneType::Deck);

        if n == 0 {
            return;
        }

        if deck.cards.is_empty() {
            println!("no cards in deck");
            println!();
            return;
        }

        println!("cards on top of deck:");

        for i in 0..std::cmp::min(n, deck.cards.len()) {
            println!("    {}) {}", i, deck.cards[i].name);
        }

        println!();
    }

    /// Moves a card from the battlefield to the graveyard.
    pub(crate) fn sacrifice(&mut self, card: &Specifier) -> Result<()> {
        self.move_card(card, ZoneType::Battlefield, ZoneType::Graveyard)
    }

    pub(crate) fn sort_battlefield(&mut self) {
        let battlefield = self.get_zone(ZoneType::Battlefield);

        if battlefield.cards.is_empty() {
            return;
        }

        battlefield.cards.sort_by_key(|card| {
            if card.is_land() {
                3
            } else if card.is_creature() {
                1
            } else {
                2
            }
        });
    }

    pub(crate) fn tuck(&mut self, card: &Specifier, from: ZoneType) -> Result<()> {
        self.move_card(card, from, ZoneType::Deck)
    }

    pub(crate) fn tutor(&mut self, card: &str) -> Result<()> {
        self.move_card(
            &Specifier::CardName(card.into()),
            ZoneType::Deck,
            ZoneType::Hand,
        )?;
        self.shuffle();

        Ok(())
    }

    pub(crate) fn print(&mut self, target: PrintTarget) {
        let location = match target.as_zone_type() {
            Some(loc) => loc,
            None => return,
        };

        let zone = self.get_zone(location);

        if zone.cards.is_empty() {
            println!("no cards in {}", location.name());
            println!();
            return;
        }

        println!("cards in {}:", location.name());

        for i in 0..zone.cards.len() {
            println!("    {}) {}", i, zone.cards[i].name);
        }

        println!();
    }

    pub(crate) fn print_game_state(&mut self) {
        println!("battlefield:");
        self.print_battlefield();
        self.print_hand();
        self.print_zone_count(ZoneType::Deck);
        self.print_zone_count(ZoneType::Graveyard);
        self.print_zone_count(ZoneType::Exile);
    }

    fn print_battlefield(&mut self) {
        self.sort_battlefield();

        let mut count = self
            .print_battlefield_line("creatures", 0, |card| card.is_creature() && !card.is_land());

        count += self.print_battlefield_line("permanents", count, |card| {
            card.is_permanent() && !card.is_land()
        });

        self.print_battlefield_line("lands", count, |card| card.is_land());

        println!();
    }

    fn print_battlefield_line(
        &self,
        line_name: &str,
        previous_count: usize,
        filter: impl Fn(&Card) -> bool,
    ) -> usize {
        let battlefield = match self.zones.get(&ZoneType::Battlefield) {
            Some(zone) => zone,
            None => return 0,
        };

        let mut current_count = 0;

        while previous_count + current_count < battlefield.cards.len()
            && filter(&battlefield.cards[previous_count + current_count])
        {
            if current_count == 0 {
                print!("    {}: ", line_name);
            } else {
                print!("  ");
            }

            print!(
                "{}) {}",
                previous_count + current_count,
                battlefield.cards[previous_count + current_count].name
            );
            current_count += 1;
        }

        if current_count > 0 {
            println!();
        }

        current_count
    }

    fn print_hand(&self) {
        let hand = self
            .zones
            .get(&ZoneType::Hand)
            .map(|zone| zone.cards.as_slice())
            .unwrap_or_default();

        print!("hand: ");

        if hand.is_empty() {
            println!("[no cards]");
            return;
        }

        let mut first = true;

        for (i, card) in hand.into_iter().enumerate() {
            if !first {
                print!("  ");
            }

            print!("{}) {}", i, card.name);
            first = false;
        }

        println!()
    }

    fn print_zone_count(&self, zone: ZoneType) {
        let count = self
            .zones
            .get(&zone)
            .map(|zone| zone.cards.len())
            .unwrap_or(0);
        println!("{}: [{} cards]", zone.name(), count)
    }
}

fn normalize_card_name(name: &str) -> String {
    let lowercase_name = name.to_lowercase();
    let parts: Vec<_> = lowercase_name.split_whitespace().collect();
    let mut file_name = parts.join("_");
    file_name.push_str(".json");

    file_name
}

fn card_path_from_name(name: &str) -> Option<PathBuf> {
    GOLDFISH_CACHE_DIR
        .as_ref()?
        .join(name.chars().next()?.to_string().to_lowercase())
        .join(normalize_card_name(name))
        .into()
}

fn lookup_card_in_cache(name: &str) -> Option<Card> {
    let card_path = card_path_from_name(name)?;

    if !card_path.exists() {
        return None;
    }

    let file = File::open(card_path).ok()?;
    serde_json::from_reader(file).ok()
}

fn cache_card(card: &Card) -> Option<()> {
    let path = card_path_from_name(&card.name)?;

    std::fs::create_dir_all(path.parent()?).ok()?;

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .ok()?;

    serde_json::to_writer_pretty(file, card).ok()?;

    None
}

fn get_card_and_cache(name: &str) -> Result<Card> {
    if let Some(card) = lookup_card_in_cache(name) {
        return Ok(card);
    }

    let card = match Card::named(name) {
        Ok(card) => card,
        Err(..) => bail!("card named `{}` doesn't exist", name),
    };

    cache_card(&card);

    Ok(card)
}
