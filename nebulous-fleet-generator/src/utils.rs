use chumsky::prelude::*;
use chumsky::stream::Stream;
use thiserror::Error;

use std::hash::Hash;
use std::ops::Range;
use std::str::FromStr;
use std::fmt;



#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, )]
pub struct FmtList<'t, T>(pub &'t [T]);

impl<'t, T> fmt::Display for FmtList<'t, T> where T: fmt::Display {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for (i, value) in self.0.into_iter().enumerate() {
      if i != 0 { f.write_str(", ")? };
      fmt::Display::fmt(value, f)?;
    };

    Ok(())
  }
}



#[derive(Debug, Error)]
pub enum Errors {
  #[error("error(s) occured in parser stage 1: {}", FmtList(.0))]
  Stage1(Vec<Simple<char>>),
  #[error("error(s) occured in parser stage 2: {}", FmtList(.0))]
  Stage2(Vec<Simple<Token>>)
}

impl From<Vec<Simple<char>>> for Errors {
  fn from(errors: Vec<Simple<char>>) -> Self {
    Errors::Stage1(errors)
  }
}

impl From<Vec<Simple<Token>>> for Errors {
  fn from(errors: Vec<Simple<Token>>) -> Self {
    Errors::Stage2(errors)
  }
}

pub fn run<P: Parseable<Token>>(source: &str) -> Result<P, Errors> {
  let tokens: Tokens = run_stage1(source)?;
  let value: P = run_stage2(source, tokens)?;
  Ok(value)
}

pub fn run_stage1<P: Parseable<char>>(source: &str) -> Result<P, Vec<Simple<char>>> {
  P::parser().parse(source)
}

pub fn run_stage2<P, I, T>(source: &str, tokens: I) -> Result<P, Vec<Simple<T>>>
where P: Parseable<T>, I: IntoIterator<Item = (T, Range<usize>)>, T: Clone + Eq + Hash {
  let eoi = source.len()..source.len() + 1;
  let stream = Stream::from_iter(eoi, tokens.into_iter());
  P::parser().parse(stream)
}

pub trait Parseable<T: Clone + Eq + Hash>: Sized {
  fn parser() -> impl Parser<T, Self, Error = Simple<T>>;
}

impl<I: Clone + Eq + Hash, O: Parseable<I>> Parseable<I> for Option<O> {
  fn parser() -> impl Parser<I, Self, Error = Simple<I>> {
    O::parser().or_not()
  }
}

pub type Span = Range<usize>;
pub type Tokens = Vec<(Token, Span)>;

impl Parseable<char> for Tokens {
  fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
    Token::parser()
      .map_with_span(|token, span| (token, span))
      .padded().repeated().then_ignore(end())
  }
}



#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
  Symbol(Symbol),
  Ident(Ident)
}

impl Parseable<char> for Token {
  fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
    choice((
      Symbol::parser().map(Token::Symbol),
      Ident::parser().map(Token::Ident)
    ))
  }
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Token::Symbol(symbol) => fmt::Display::fmt(symbol, f),
      Token::Ident(ident) => fmt::Display::fmt(ident, f)
    }
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
  Slash,
  Comma,
  Ellipsis,
  SquareBracketOpen,
  SquareBracketClose,
  RoundBracketOpen,
  RoundBracketClose
}

impl Symbol {
  pub const fn to_str(self) -> &'static str {
    match self {
      Self::Slash => "/",
      Self::Comma => ",",
      Self::Ellipsis => "..",
      Self::SquareBracketOpen => "[",
      Self::SquareBracketClose => "]",
      Self::RoundBracketOpen => "(",
      Self::RoundBracketClose => ")"
    }
  }
}

impl Parseable<char> for Symbol {
  fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
    choice([
      just("/").to(Self::Slash),
      just(",").to(Self::Comma),
      just("..").to(Self::Ellipsis),
      just("[").to(Self::SquareBracketOpen),
      just("]").to(Self::SquareBracketClose),
      just("(").to(Self::RoundBracketOpen),
      just(")").to(Self::RoundBracketClose)
    ])
  }
}

impl fmt::Display for Symbol {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(self.to_str())
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
  pub contents: Box<str>
}

impl Parseable<char> for Ident {
  fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
    filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_')
      .repeated().at_least(1)
      .collect::<String>().map(String::into_boxed_str)
      .map(|contents| Ident { contents })
  }
}

impl fmt::Display for Ident {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.contents)
  }
}

pub type ParserSymbol = chumsky::combinator::To<chumsky::primitive::Just<Token, Token, Simple<Token>>, Token, ()>;
pub type ParserDelimitedBySymbol<P> = chumsky::combinator::DelimitedBy<P, ParserSymbol, ParserSymbol, (), ()>;
pub type ParserSeparatedBySymbol<P> = chumsky::combinator::SeparatedBy<P, ParserSymbol, ()>;

pub fn delimited_round_bracket_list<T, P: Parser<Token, T, Error = Simple<Token>>>(p: P, at_least: usize) -> ParserDelimitedBySymbol<ParserSeparatedBySymbol<P>> {
  delimited_by_round_brackets(separated_by_commas(p, at_least))
}

pub fn delimited_square_bracket_list<T, P: Parser<Token, T, Error = Simple<Token>>>(p: P, at_least: usize) -> ParserDelimitedBySymbol<ParserSeparatedBySymbol<P>> {
  delimited_by_square_brackets(separated_by_commas(p, at_least))
}

pub fn delimited_by_round_brackets<T, P: Parser<Token, T, Error = Simple<Token>>>(p: P) -> ParserDelimitedBySymbol<P> {
  p.delimited_by(symbol(Symbol::RoundBracketOpen), symbol(Symbol::RoundBracketClose))
}

pub fn delimited_by_square_brackets<T, P: Parser<Token, T, Error = Simple<Token>>>(p: P) -> ParserDelimitedBySymbol<P> {
  p.delimited_by(symbol(Symbol::SquareBracketOpen), symbol(Symbol::SquareBracketClose))
}

pub fn separated_by_commas<T, P: Parser<Token, T, Error = Simple<Token>>>(p: P, at_least: usize) -> ParserSeparatedBySymbol<P> {
  p.separated_by(symbol(Symbol::Comma)).allow_trailing().at_least(at_least)
}

pub fn symbol(symbol: Symbol) -> ParserSymbol {
  just(Token::Symbol(symbol)).ignored()
}

pub fn ident() -> chumsky::primitive::FilterMap<impl Fn(Span, Token) -> Result<Box<str>, Simple<Token>>, Simple<Token>> {
  chumsky::select! { Token::Ident(Ident { contents }) => contents }
}

pub fn keyword(keyword: &'static str) -> chumsky::primitive::FilterMap<impl Fn(Span, Token) -> Result<(), Simple<Token>>, Simple<Token>> {
  chumsky::select! { Token::Ident(Ident { contents }) if &*contents == keyword => () }
}

pub fn keyword_match<T, F>(f: F) -> impl Parser<Token, T, Error = Simple<Token>> + Clone
where F: Fn(&str) -> Option<T> + Clone {
  chumsky::primitive::filter_map(move |span, token| match token {
    Token::Ident(Ident { ref contents }) => f(contents).ok_or_else(|| {
      Simple::expected_input_found(span, None, Some(token))
    }),
    Token::Symbol(..) => Err(Simple::expected_input_found(span, None, Some(token)))
  })
}

pub fn keyword_parse<T>() -> impl Parser<Token, T, Error = Simple<Token>> + Clone
where T: FromStr, T::Err: ToString {
  chumsky::primitive::filter_map(move |span, token| match token {
    Token::Ident(Ident { ref contents }) => contents.parse::<T>().map_err(|err| Simple::custom(span, err)),
    Token::Symbol(..) => Err(Simple::expected_input_found(span, None, Some(token)))
  })
}

pub mod serde_base64_cbor {
  use serde::de::{Deserialize, DeserializeOwned, Deserializer};
  use serde::ser::{Serialize, Serializer};
  use singlefile::FileFormatUtf8;
  use singlefile_formats::base64::Base64;
  use singlefile_formats::cbor_serde::Cbor;

  const FORMAT: Base64<Cbor> = Base64::with_standard(Cbor);

  pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
  where D: Deserializer<'de>, T: DeserializeOwned + Serialize {
    String::deserialize(deserializer).and_then(|string| {
      FORMAT.from_string_buffer(&string)
        .map_err(serde::de::Error::custom)
    })
  }

  pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer, T: DeserializeOwned + Serialize {
    FORMAT.to_string_buffer(value)
      .map_err(serde::ser::Error::custom)
      .and_then(|string| string.serialize(serializer))
  }
}

pub mod serde_one_or_many {
  use serde::de::{Deserialize, Deserializer};
  use serde::ser::{Serialize, Serializer};

  #[derive(Deserialize)]
  enum OneOrMany<T> {
    Many(Vec<T>),
    One(T)
  }

  pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
  where D: Deserializer<'de>, T: Deserialize<'de> {
    OneOrMany::deserialize(deserializer).map(|values| match values {
      OneOrMany::Many(many) => many,
      OneOrMany::One(one) => vec![one]
    })
  }

  pub fn serialize<S, T>(values: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
  where S: Serializer, T: Serialize {
    match <&[T; 1]>::try_from(values.as_slice()) {
      Ok([value]) => value.serialize(serializer),
      Err(..) => values.serialize(serializer)
    }
  }
}
