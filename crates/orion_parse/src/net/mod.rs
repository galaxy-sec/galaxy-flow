use std::net::{IpAddr, Ipv4Addr};

use winnow::{
    ascii::{multispace0, Caseless},
    combinator::{fail, peek, repeat},
    error::{ContextError, ErrMode, ParserError},
    token::any,
    ModalResult, Parser,
};

use crate::symbol::wn_desc;
use crate::utils::peek_one;

#[derive(PartialEq)]
enum AddrKind {
    Ipv4,
    Ipv6,
}

fn head_ip<'a>(last: &mut Option<AddrKind>) -> impl Parser<&'a str, char, ContextError> + '_ {
    move |input: &mut &'a str| {
        let initial = (peek(any)).parse_next(input)?;
        match initial {
            '0'..='9' => any.parse_next(input),
            'A'..='F' | 'a'..='f' => {
                *last = Some(AddrKind::Ipv6);
                any.parse_next(input)
            }
            '.' => {
                if *last == Some(AddrKind::Ipv6) {
                    fail.parse_next(input)
                } else {
                    *last = Some(AddrKind::Ipv4);
                    any.parse_next(input)
                }
            }
            ':' => {
                if *last == Some(AddrKind::Ipv4) {
                    fail.parse_next(input)
                } else {
                    *last = Some(AddrKind::Ipv6);
                    any.parse_next(input)
                }
            }
            _ => fail.parse_next(input),
        }
    }
}

pub fn ip_v4(input: &mut &str) -> ModalResult<IpAddr> {
    let mut last_kind = None;
    let accurate_ip = repeat(1.., head_ip(&mut last_kind))
        .fold(String::new, |mut acc, c| {
            acc.push(c);
            acc
        })
        .try_map(|ref x| x.parse())
        .parse_next(input)?;
    Ok(accurate_ip)
}
pub fn ip(input: &mut &str) -> ModalResult<IpAddr> {
    multispace0.parse_next(input)?;

    let str = peek_one.parse_next(input);
    if let Ok(s) = str {
        let addr = if s == "l" {
            Caseless("localhost")
                .map(|_| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))
                .context(wn_desc("<localhost>"))
                .parse_next(input)?
        } else {
            ip_v4.context(wn_desc("<ipv4>")).parse_next(input)?
        };
        Ok(addr)
    } else {
        Err(ErrMode::Backtrack(ParserError::from_input(input)))
    }
}
