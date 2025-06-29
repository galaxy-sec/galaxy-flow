use crate::scope::ScopeEval;
use std::fmt::Display;
use winnow::ascii::multispace0;
use winnow::combinator::peek;
use winnow::error::ParserError;
use winnow::stream::{Checkpoint, Stream};
use winnow::token::{literal, take};
use winnow::Parser;
use winnow::Result;

pub fn get_scope(data: &mut &str, beg: char, end: char) -> Result<String> {
    multispace0.parse_next(data)?;
    let extend_len = ScopeEval::len(data, beg, end);
    if extend_len < 2 {
        return Err(ParserError::from_input(data));
    }
    literal(beg.to_string().as_str()).parse_next(data)?;
    let group = take(extend_len - 2).parse_next(data)?;
    literal(end.to_string().as_str()).parse_next(data)?;
    multispace0(data)?;
    Ok(group.to_string())
}

pub fn peek_one(data: &mut &str) -> Result<String> {
    let char = peek(take(1usize)).parse_next(data)?;
    Ok(char.to_string())
}

pub trait RestAble {
    fn err_reset<'a>(self, data: &mut &'a str, point: &Checkpoint<&'a str, &'a str>) -> Self;
}

impl<T, E> RestAble for Result<T, E> {
    fn err_reset<'a>(self, data: &mut &'a str, point: &Checkpoint<&'a str, &'a str>) -> Self {
        if self.is_err() {
            data.reset(point);
        }
        self
    }
}

pub fn err_convert<T, E: Display>(result: Result<T, E>) -> Result<T> {
    match result {
        Ok(obj) => Ok(obj),
        Err(_e) => Err(ParserError::from_input(&"loss err")),
    }
}
