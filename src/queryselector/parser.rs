use crate::{stream::Stream, util};

use super::Selector;

/// A query selector parser
pub struct Parser<'a> {
    stream: Stream<'a, u8>,
}

impl<'a> Parser<'a> {
    /// Creates a new query selector parser
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            stream: Stream::new(input),
        }
    }

    fn skip_whitespaces(&mut self) -> bool {
        let has_whitespace = self.stream.expect_and_skip_cond(b' ');
        while !self.stream.is_eof() {
            if self.stream.expect_and_skip(b' ').is_none() {
                break;
            }
        }
        has_whitespace
    }

    fn read_identifier(&mut self) -> &'a [u8] {
        let start = self.stream.idx;

        while !self.stream.is_eof() {
            let is_ident = self.stream.current().copied().is_some_and(util::is_ident);
            if !is_ident {
                break;
            } else {
                self.stream.advance();
            }
        }

        self.stream.slice(start, self.stream.idx)
    }

    fn parse_attribute(&mut self) -> Option<Selector<'a>> {
        let attribute = self.read_identifier();
        let ty = match self.stream.current_cpy() {
            Some(b']') => {
                self.stream.advance();
                Selector::Attribute(attribute)
            }
            Some(b'=') => {
                self.stream.advance();
                let quote = self.stream.expect_oneof_and_skip(b"\"'");
                let value = self.read_identifier();
                if let Some(quote) = quote {
                    // Only require the given quote if the value starts with a quote
                    self.stream.expect_and_skip(quote)?;
                }
                self.stream.expect_and_skip(b']')?;
                Selector::AttributeValue(attribute, value)
            }
            Some(c @ b'~' | c @ b'^' | c @ b'$' | c @ b'*') => {
                self.stream.advance();
                self.stream.expect_and_skip(b'=')?;
                let quote = self.stream.expect_oneof_and_skip(b"\"'");
                let value = self.read_identifier();
                if let Some(quote) = quote {
                    // Only require the given quote if the value starts with a quote
                    self.stream.expect_and_skip(quote)?;
                }
                self.stream.expect_and_skip(b']')?;
                match c {
                    b'~' => Selector::AttributeValueWhitespacedContains(attribute, value),
                    b'^' => Selector::AttributeValueStartsWith(attribute, value),
                    b'$' => Selector::AttributeValueEndsWith(attribute, value),
                    b'*' => Selector::AttributeValueSubstring(attribute, value),
                    _ => unreachable!(),
                }
            }
            _ => return None,
        };
        Some(ty)
    }

    /// Parses a single atomic selector token (tag, id, class, *, or attribute)
    fn parse_atom(&mut self) -> Option<Selector<'a>> {
        let tok = self.stream.current_cpy()?;

        match tok {
            b'#' => {
                self.stream.advance();
                let id = self.read_identifier();
                Some(Selector::Id(id))
            }
            b'.' => {
                self.stream.advance();
                let class = self.read_identifier();
                Some(Selector::Class(class))
            }
            b'*' => {
                self.stream.advance();
                Some(Selector::All)
            }
            b'[' => {
                self.stream.advance();
                self.parse_attribute()
            }
            _ if util::is_ident(tok) => {
                let tag = self.read_identifier();
                Some(Selector::Tag(tag))
            }
            _ => None,
        }
    }

    /// Parses one or more adjacent atoms (no whitespace) into an And chain
    fn parse_compound(&mut self) -> Option<Selector<'a>> {
        let mut result = self.parse_atom()?;

        while let Some(next) = self.parse_atom() {
            result = Selector::And(Box::new(result), Box::new(next));
        }

        Some(result)
    }

    /// Parses a full selector expression with left-associative combinators
    pub fn selector(&mut self) -> Option<Selector<'a>> {
        self.skip_whitespaces();
        let mut left = self.parse_compound()?;

        loop {
            let has_whitespace = self.skip_whitespaces();

            match self.stream.current_cpy() {
                None => break,
                Some(b',') => {
                    self.stream.advance();
                    // Or can be right-recursive; associativity is irrelevant for matching
                    let right = self.selector()?;
                    left = Selector::Or(Box::new(left), Box::new(right));
                    break;
                }
                Some(b'>') => {
                    self.stream.advance();
                    self.skip_whitespaces();
                    let right = self.parse_compound()?;
                    left = Selector::Parent(Box::new(left), Box::new(right));
                }
                _ if has_whitespace => {
                    let right = self.parse_compound()?;
                    left = Selector::Descendant(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }

        Some(left)
    }
}
