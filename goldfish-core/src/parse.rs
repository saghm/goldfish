use anyhow::{bail, Result};

use crate::common::{PrintTarget, Specifier, Statement, ZoneType};

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
            "exile" => self.parse_exile()?,
            "fetch" => self.parse_fetch(),
            "help" => self.parse_help()?,
            "inspect" => self.parse_inspect()?,
            "load" => self.parse_load(),
            "mill" => self.parse_mill()?,
            "move" => self.parse_move()?,
            "play" => self.parse_play()?,
            "print" => self.parse_print()?,
            "restart" => self.parse_restart()?,
            "sac" => self.parse_sacrifice()?,
            "shuffle" => self.parse_shuffle()?,
            "tuck" => self.parse_tuck()?,
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

    fn parse_exile(mut self) -> Result<Statement> {
        // Split off everything after "from" and throw away "from".
        let source = match self.split_off_at("from") {
            Some(rest) => rest,
            None => bail!("`exile` needs to specify source with `from`"),
        };

        if source.is_empty() {
            bail!("`exile` needs source after `from`");
        }

        if source.len() > 1 {
            bail!("`exile` needs a single-word source");
        }

        let from = ZoneType::parse(source[0])?;

        let card = self.parse_specifier()?;

        Ok(Statement::Exile { card, from })
    }

    fn parse_fetch(self) -> Statement {
        Statement::Fetch(self.parts.join(" "))
    }

    fn parse_help(&self) -> Result<Statement> {
        if !self.parts.is_empty() {
            bail!("`help` shouldn't have any words following it");
        }

        Ok(Statement::Help)
    }

    fn parse_inspect(self) -> Result<Statement> {
        if self.parts.is_empty() {
            return Ok(Statement::Inspect(1));
        }

        if self.parts.len() > 1 {
            bail!("`inspect` needs a single-word count");
        }

        let count = match self.parts[0].parse() {
            Ok(count) => count,
            Err(_) => bail!(
                "`{}` is not a valid numeric count for `inspect`",
                self.parts[0]
            ),
        };

        Ok(Statement::Inspect(count))
    }

    fn parse_load(self) -> Statement {
        Statement::Load(self.parts.join(" "))
    }

    fn parse_mill(self) -> Result<Statement> {
        if self.parts.len() != 1 {
            bail!("`mill` needs a single word count");
        }

        let count = match self.parts[0].parse() {
            Ok(count) => count,
            Err(..) => bail!(
                "`{}` is not a valid numeric count for `mill`",
                self.parts[0]
            ),
        };

        Ok(Statement::Mill(count))
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

    fn parse_print(&self) -> Result<Statement> {
        if self.parts.is_empty() {
            return Ok(Statement::Print(PrintTarget::Default));
        }

        if self.parts.len() > 1 {
            bail!("`print` either needs no target or a one-word target");
        }

        let target = PrintTarget::parse(self.parts[0])?;

        Ok(Statement::Print(target))
    }

    fn parse_restart(&self) -> Result<Statement> {
        if !self.parts.is_empty() {
            bail!("`restart` shouldn't have any words following it");
        }

        Ok(Statement::Restart)
    }

    fn parse_sacrifice(self) -> Result<Statement> {
        Ok(Statement::Sacrifice(self.parse_specifier()?))
    }

    fn parse_shuffle(&self) -> Result<Statement> {
        if !self.parts.is_empty() {
            bail!("`shuffle` shouldn't have any words following it");
        }

        Ok(Statement::Shuffle)
    }

    fn parse_tuck(mut self) -> Result<Statement> {
        // Split off everything after "from" and throw away "from".
        let source = match self.split_off_at("from") {
            Some(rest) => rest,
            None => bail!("`tuck` needs to specify source with `from`"),
        };

        if source.is_empty() {
            bail!("`tuck` needs source after `from`");
        }

        if source.len() > 1 {
            bail!("`tuck` needs a single-word source");
        }

        let from = ZoneType::parse(source[0])?;

        let card = self.parse_specifier()?;

        Ok(Statement::Tuck { card, from })
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
