use anyhow::{bail, Result};

#[derive(Debug)]
pub(crate) enum Statement {
    Nop,

    Bounce(Specifier),
    Discard(Specifier),
    Draw(usize),
    Exile {
        card: Specifier,
        from: ZoneType,
    },
    Fetch(String),
    Help,
    Inspect(usize),
    Load(String),
    Mill(usize),
    Move {
        card: Specifier,
        from: ZoneType,
        to: ZoneType,
    },
    Play(Specifier),
    Print(PrintTarget),
    Restart,
    Sacrifice(Specifier),
    Shuffle,
    Tuck {
        card: Specifier,
        from: ZoneType,
    },
    Tutor(String),
}

#[derive(Debug)]
pub(crate) enum PrintTarget {
    Default,
    Exile,
    Graveyard,
}

impl PrintTarget {
    pub(crate) fn as_zone_type(&self) -> Option<ZoneType> {
        match self {
            Self::Default => None,
            Self::Exile => Some(ZoneType::Exile),
            Self::Graveyard => Some(ZoneType::Graveyard),
        }
    }

    pub(crate) fn parse(location: &str) -> Result<Self> {
        let loc = match location {
            "exile" => Self::Exile,
            "graveyard" => Self::Graveyard,
            other => bail!("`{}` is not a known location", other),
        };

        Ok(loc)
    }
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
            Self::Battlefield => "battlefield",
            Self::Deck => "deck",
            Self::Exile => "exile",
            Self::Graveyard => "graveyard",
            Self::Hand => "hand",
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
