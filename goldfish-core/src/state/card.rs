use std::collections::HashSet;

use anyhow::{bail, Result};
use getset::Getters;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum CardType {
    Artifact,
    Creature,
    Enchantment,
    Instant,
    Land,
    Planeswalker,
    Sorcery,
}

impl CardType {
    pub(super) fn parse(s: &str) -> Result<Self> {
        let card_type = match s.trim().to_lowercase().as_str() {
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

    pub(super) fn is_permanent(&self) -> bool {
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
pub(super) struct Card {
    #[get = "pub(super)"]
    types: HashSet<CardType>,

    #[get = "pub(super)"]
    name: String,
}

impl Card {
    pub(super) fn new(name: &str, types: HashSet<CardType>) -> Self {
        Self {
            name: name.to_string(),
            types,
        }
    }

    pub(super) fn is_named(&self, name: &str) -> bool {
        self.name.trim().to_lowercase() == name.trim().to_lowercase()
    }

    pub(super) fn is_permanent(&self) -> bool {
        self.types.iter().any(|card_type| card_type.is_permanent())
    }
}
