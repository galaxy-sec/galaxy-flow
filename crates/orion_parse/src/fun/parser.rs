use std::net::IpAddr;

use crate::fun::fun_trait::{Fun1Builder, Fun2Builder};
use crate::net::ip;
use crate::symbol::{symbol_bracket_beg, symbol_bracket_end, symbol_comma};
use winnow::ascii::{digit1, multispace0};
use winnow::combinator::separated;
use winnow::{ModalResult, Parser};

use super::fun_trait::WnTake;

pub fn take_call_args2<T: Fun2Builder>(data: &mut &str) -> ModalResult<(T::ARG1, T::ARG2)> {
    multispace0.parse_next(data)?;
    symbol_bracket_beg.parse_next(data)?;
    multispace0.parse_next(data)?;
    let a1 = T::args1.parse_next(data)?;
    (multispace0, symbol_comma, multispace0).parse_next(data)?;
    let a2 = T::args2.parse_next(data)?;
    multispace0.parse_next(data)?;
    symbol_bracket_end.parse_next(data)?;
    Ok((a1, a2))
}

pub fn take_call_args1<T: Fun1Builder>(data: &mut &str) -> ModalResult<T::ARG1> {
    multispace0.parse_next(data)?;
    symbol_bracket_beg.parse_next(data)?;
    multispace0.parse_next(data)?;
    let a1 = T::args1.parse_next(data)?;
    multispace0.parse_next(data)?;
    symbol_bracket_end.parse_next(data)?;
    Ok(a1)
}

pub fn call_fun_args2<T: Fun2Builder>(data: &mut &str) -> ModalResult<T> {
    T::fun_name().parse_next(data)?;
    let args = take_call_args2::<T>.parse_next(data)?;
    let obj = T::build(args);
    Ok(obj)
}

pub fn call_fun_args1<T: Fun1Builder>(data: &mut &str) -> ModalResult<T> {
    T::fun_name().parse_next(data)?;
    let args = take_call_args1::<T>.parse_next(data)?;
    let obj = T::build(args);
    Ok(obj)
}

pub fn take_arr<T: WnTake<T>>(data: &mut &str) -> ModalResult<Vec<T>> {
    (multispace0, "[", multispace0).parse_next(data)?;
    let arr: Vec<T> = separated(1.., T::parse_next, ",").parse_next(data)?;
    (multispace0, "]").parse_next(data)?;
    Ok(arr)
}

impl WnTake<u32> for u32 {
    fn parse_next(input: &mut &str) -> ModalResult<u32> {
        let str = digit1(input)?;
        Ok(str.parse::<u32>().unwrap_or(0))
    }
}

impl WnTake<i64> for i64 {
    fn parse_next(input: &mut &str) -> ModalResult<i64> {
        let str = digit1(input)?;
        Ok(str.parse::<i64>().unwrap_or(0))
    }
}
impl WnTake<IpAddr> for IpAddr {
    fn parse_next(input: &mut &str) -> ModalResult<IpAddr> {
        ip.parse_next(input)
    }
}

#[cfg(test)]
mod test {
    use super::{call_fun_args1, take_arr};
    use crate::fun::fun_trait::Fun1Builder;
    use winnow::{
        //ascii::{digit1, multispace0},
        ModalResult,
        Parser,
    };

    #[derive(Debug, PartialEq)]
    struct A {
        arr: Vec<u32>,
    }
    impl Fun1Builder for A {
        type ARG1 = Vec<u32>;

        fn args1(data: &mut &str) -> winnow::ModalResult<Self::ARG1> {
            take_arr::<u32>(data)
        }

        fn fun_name() -> &'static str {
            "fun_a"
        }

        fn build(args: Self::ARG1) -> Self {
            A { arr: args }
        }
    }
    #[test]
    fn test_arr_args_fun() -> ModalResult<()> {
        let mut data = "fun_a([1,2,3])";
        let x = call_fun_args1::<A>.parse_next(&mut data)?;
        println!("{:?}", x);
        assert_eq!(x, A { arr: vec![1, 2, 3] });
        Ok(())
    }
}
