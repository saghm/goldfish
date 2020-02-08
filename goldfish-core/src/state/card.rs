use scryfall::card::Card;

const PERMANENT_TYPES: [&str; 5] = [
    "artifact",
    "creature",
    "enchantment",
    "land",
    "planeswalker",
];

pub(super) trait CardExt {
    fn is_creature(&self) -> bool;

    fn is_land(&self) -> bool;

    fn is_named(&self, name: &str) -> bool;

    fn is_permanent(&self) -> bool;
}

impl CardExt for Card {
    fn is_creature(&self) -> bool {
        // All cards in Scryfall seem to have a type line, so we just unwrap it.
        self.type_line
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("creature")
    }

    fn is_land(&self) -> bool {
        // All cards in Scryfall seem to have a type line, so we just unwrap it.
        self.type_line
            .as_ref()
            .unwrap()
            .to_lowercase()
            .contains("land")
    }

    fn is_named(&self, name: &str) -> bool {
        self.name.trim().to_lowercase() == name.trim().to_lowercase()
    }

    fn is_permanent(&self) -> bool {
        // All cards in Scryfall seem to have a type line, so we just unwrap it.
        let types = self.type_line.as_ref().unwrap().to_lowercase();

        PERMANENT_TYPES
            .iter()
            .any(|card_type| types.contains(card_type))
    }
}
