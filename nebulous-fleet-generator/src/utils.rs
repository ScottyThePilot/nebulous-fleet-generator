use chumsky::prelude::*;
use chumsky::stream::Stream;
use thiserror::Error;

use std::hash::Hash;
use std::ops::Range;
use std::str::FromStr;



#[derive(Debug, Error)]
pub enum Errors {
  #[error("an error occured in stage 1")]
  Stage1(Vec<Simple<char>>),
  #[error("an error occured in stage 2")]
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

pub fn symbol(symbol: Symbol) -> impl Parser<Token, (), Error = Simple<Token>> + Clone {
  just(Token::Symbol(symbol)).ignored()
}

pub fn ident() -> impl Parser<Token, Box<str>, Error = Simple<Token>> + Clone {
  chumsky::select! { Token::Ident(Ident { contents }) => contents }
}

pub fn keyword(keyword: &'static str) -> impl Parser<Token, (), Error = Simple<Token>> + Clone {
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
