use anyhow::{bail, Result};

#[derive(Debug)]
pub(crate) enum Statement {
    Nop,

    Bounce(Specifier),
    Discard(Specifier),
    Draw(usize),
    Fetch(String),
    Help,
    Inspect(usize),
    Load(String),
    Move {
        card: Specifier,
        from: ZoneType,
        to: ZoneType,
    },
    Play(Specifier),
    Print,
    Restart,
    Sacrifice(Specifier),
    Tutor(String),
}

#[derive(Debug)]
pub(crate) enum Specifier {
    CardName(String),
    Index(usize),
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
    pub(crate) fn name(&self) -> &str {
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
