use anyhow::{bail, Result};

use crate::common::{Specifier, Statement, ZoneType};

pub(crate) struct Input<'a> {
    parts: Vec<&'a str>,
}

impl<'a> Input<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            parts: input.split_whitespace().collect(),
        }
    }

    fn split_off_at(&mut self, item: &str) -> Option<Vec<&str>> {
        for i in (0..self.parts.len()).rev() {
            if self.parts[i] == item {
                let mut rest = self.parts.split_off(i);
                rest.remove(0);

                return Some(rest);
            }
        }

        None
    }

    pub(crate) fn parse(mut self) -> Result<Statement> {
        if self.parts.is_empty() {
            return Ok(Statement::Nop);
        }

        let statement = match self.parts.remove(0) {
            "bounce" => self.parse_bounce()?,
            "discard" => self.parse_discard()?,
            "draw" => self.parse_draw()?,
            "fetch" => self.parse_fetch(),
            "move" => self.parse_move()?,
            "play" => self.parse_play()?,
            "sac" => self.parse_sacrifice()?,
            "tutor" => self.parse_tutor(),
            other => bail!("`{}` is not a known verb", other),
        };

        Ok(statement)
    }

    fn parse_bounce(self) -> Result<Statement> {
        Ok(Statement::Bounce(self.parse_specifier()?))
    }

    fn parse_discard(self) -> Result<Statement> {
        Ok(Statement::Discard(self.parse_specifier()?))
    }

    fn parse_draw(self) -> Result<Statement> {
        if self.parts.is_empty() {
            return Ok(Statement::Draw(1));
        }

        if self.parts.len() > 1 {
            bail!("`draw` needs a single-word count");
        }

        let count = match self.parts[0].parse() {
            Ok(count) => count,
            Err(_) => bail!(
                "`{}` is not a valid numeric count for `draw`",
                self.parts[0]
            ),
        };

        Ok(Statement::Draw(count))
    }

    fn parse_fetch(self) -> Statement {
        Statement::Fetch(self.parts.join(" "))
    }

    fn parse_move(mut self) -> Result<Statement> {
        // Split off everything after "to" and throw away "to".
        let destination = match self.split_off_at("to") {
            Some(rest) => rest,
            None => bail!("`move` needs to specify destination with `to`"),
        };

        if destination.is_empty() {
            bail!("`move` needs destination after `to`");
        }

        if destination.len() > 1 {
            bail!("`move` needs a single-word destination");
        }

        let to = ZoneType::parse(destination[0])?;

        // Split off everything after "from" and throw away "from".
        let source = match self.split_off_at("from") {
            Some(rest) => rest,
            None => bail!("`move` needs to specify source with `from`"),
        };

        if source.is_empty() {
            bail!("`move` needs source after `from`");
        }

        if source.len() > 1 {
            bail!("`move` needs a single-word source");
        }

        let from = ZoneType::parse(source[0])?;

        let card = self.parse_specifier()?;

        Ok(Statement::Move { card, from, to })
    }

    fn parse_play(&self) -> Result<Statement> {
        Ok(Statement::Play(self.parse_specifier()?))
    }

    fn parse_sacrifice(self) -> Result<Statement> {
        Ok(Statement::Sacrifice(self.parse_specifier()?))
    }

    fn parse_tutor(self) -> Statement {
        Statement::Tutor(self.parts.join(" "))
    }

    fn parse_specifier(&self) -> Result<Specifier> {
        if self.parts.is_empty() {
            bail!("missing card specifier");
        }

        let spec = self.parts.join(" ");

        // If the first token doesn't begin with '$', assume it's a card name and join all the
        // remaining tokens.
        if !spec.starts_with('$') {
            return Ok(Specifier::CardName(spec));
        }

        // Otherwise, assert that there's only a single token left, and it's a number prefixed by
        // '$'.
        match &spec[1..].parse() {
            Ok(i) => Ok(Specifier::Index(*i)),
            Err(_) => bail!("`{}` is not numeric after the `$`", spec),
        }
    }
}
