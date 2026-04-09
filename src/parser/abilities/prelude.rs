pub use orion_parse::{atom::starts_with, symbol::wn_desc};
pub use winnow::{
    Parser, Result,
    ascii::{line_ending, till_line_ending},
    combinator::{fail, opt},
    token::{take_till, take_until, take_while},
};

pub use winnow::{
    ascii::multispace0,
    combinator::{alt, repeat},
};
