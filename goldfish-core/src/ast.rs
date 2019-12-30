use crate::model::{Specifier, ZoneType};

#[derive(Debug)]
pub(crate) enum Statement {
    Nop,

    Discard(Specifier),
    Draw(usize),
    Fetch(String),
    Inspect(usize),
    Move {
        card: Specifier,
        from: ZoneType,
        to: ZoneType,
    },
    Play(Specifier),
    Sacrifice(Specifier),
}
