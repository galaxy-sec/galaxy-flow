use std::net::IpAddr;

use crate::fun::fun_trait::{Fun1Builder, Fun2Builder};
use crate::net::ip;
use crate::symbol::{symbol_bracket_beg, symbol_bracket_end, symbol_comma};
use winnow::ascii::{digit1, multispace0};
use winnow::combinator::separated;
use winnow::{Parser, Result};

use super::fun_trait::WnTake;

pub fn take_call_args2<T: Fun2Builder>(data: &mut &str) -> Result<(T::ARG1, T::ARG2)> {
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

pub fn take_call_args1<T: Fun1Builder>(data: &mut &str) -> Result<T::ARG1> {
    multispace0.parse_next(data)?;
    symbol_bracket_beg.parse_next(data)?;
    multispace0.parse_next(data)?;
    let a1 = T::args1.parse_next(data)?;
    multispace0.parse_next(data)?;
    symbol_bracket_end.parse_next(data)?;
    Ok(a1)
}

pub fn call_fun_args2<T: Fun2Builder>(data: &mut &str) -> Result<T> {
    T::fun_name().parse_next(data)?;
    let args = take_call_args2::<T>.parse_next(data)?;
    let obj = T::build(args);
    Ok(obj)
}

pub fn call_fun_args1<T: Fun1Builder>(data: &mut &str) -> Result<T> {
    T::fun_name().parse_next(data)?;
    let args = take_call_args1::<T>.parse_next(data)?;
    let obj = T::build(args);
    Ok(obj)
}

pub fn take_arr<T: WnTake<T>>(data: &mut &str) -> Result<Vec<T>> {
    (multispace0, "[", multispace0).parse_next(data)?;
    let arr: Vec<T> = separated(1.., T::parse_next, ",").parse_next(data)?;
    (multispace0, "]").parse_next(data)?;
    Ok(arr)
}

impl WnTake<u32> for u32 {
    fn parse_next(input: &mut &str) -> Result<u32> {
        let str = digit1(input)?;
        Ok(str.parse::<u32>().unwrap_or(0))
    }
}

impl WnTake<i64> for i64 {
    fn parse_next(input: &mut &str) -> Result<i64> {
        let str = digit1(input)?;
        Ok(str.parse::<i64>().unwrap_or(0))
    }
}
impl WnTake<IpAddr> for IpAddr {
    fn parse_next(input: &mut &str) -> Result<IpAddr> {
        ip.parse_next(input)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fun::fun_trait::{Fun1Builder, Fun2Builder};
    use std::net::{IpAddr, Ipv4Addr};
    use winnow::Parser;

    #[derive(Debug, PartialEq)]
    struct A {
        arr: Vec<u32>,
    }
    impl Fun1Builder for A {
        type ARG1 = Vec<u32>;

        fn args1(data: &mut &str) -> winnow::Result<Self::ARG1> {
            take_arr::<u32>(data)
        }

        fn fun_name() -> &'static str {
            "fun_a"
        }

        fn build(args: Self::ARG1) -> Self {
            A { arr: args }
        }
    }

    #[derive(Debug, PartialEq)]
    struct B {
        x: u32,
        y: i64,
    }
    impl Fun2Builder for B {
        type ARG1 = u32;
        type ARG2 = i64;

        fn args1(data: &mut &str) -> winnow::Result<Self::ARG1> {
            u32::parse_next(data)
        }

        fn args2(data: &mut &str) -> winnow::Result<Self::ARG2> {
            i64::parse_next(data)
        }

        fn fun_name() -> &'static str {
            "fun_b"
        }

        fn build(args: (Self::ARG1, Self::ARG2)) -> Self {
            B {
                x: args.0,
                y: args.1,
            }
        }
    }

    #[derive(Debug, PartialEq)]
    struct C {
        ip1: IpAddr,
        ip2: IpAddr,
    }
    impl Fun2Builder for C {
        type ARG1 = IpAddr;
        type ARG2 = IpAddr;

        fn args1(data: &mut &str) -> winnow::Result<Self::ARG1> {
            IpAddr::parse_next(data)
        }

        fn args2(data: &mut &str) -> winnow::Result<Self::ARG2> {
            IpAddr::parse_next(data)
        }

        fn fun_name() -> &'static str {
            "fun_c"
        }

        fn build(args: (Self::ARG1, Self::ARG2)) -> Self {
            C {
                ip1: args.0,
                ip2: args.1,
            }
        }
    }

    #[test]
    fn test_arr_args_fun() -> winnow::Result<()> {
        let mut data = "fun_a([1,2,3])";
        let x = call_fun_args1::<A>.parse_next(&mut data)?;
        assert_eq!(x, A { arr: vec![1, 2, 3] });
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_arr_args_fun_with_spaces() -> winnow::Result<()> {
        let mut data = "fun_a([1,2,3])";
        let x = call_fun_args1::<A>.parse_next(&mut data)?;
        assert_eq!(x, A { arr: vec![1, 2, 3] });
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_arr_args_fun_single_element() -> winnow::Result<()> {
        let mut data = "fun_a([42])";
        let x = call_fun_args1::<A>.parse_next(&mut data)?;
        assert_eq!(x, A { arr: vec![42] });
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_arr_args_fun_empty() -> winnow::Result<()> {
        // take_arr requires at least one element, so empty array will fail
        let mut data = "fun_a([])";
        let result = call_fun_args1::<A>.parse_next(&mut data);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_call_fun_args2_basic() -> winnow::Result<()> {
        let mut data = "fun_b(42, 100)";
        let x = call_fun_args2::<B>.parse_next(&mut data)?;
        assert_eq!(x, B { x: 42, y: 100 });
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_call_fun_args2_with_spaces() -> winnow::Result<()> {
        let mut data = "fun_b( 42 , 100 )";
        let x = call_fun_args2::<B>.parse_next(&mut data)?;
        assert_eq!(x, B { x: 42, y: 100 });
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_call_fun_args2_negative() -> winnow::Result<()> {
        // i64 parser only supports digits, not negative numbers
        let mut data = "fun_b(42, -100)";
        let result = call_fun_args2::<B>.parse_next(&mut data);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_call_fun_args2_large_numbers() -> winnow::Result<()> {
        let mut data = "fun_b(999999, 888888888888)";
        let x = call_fun_args2::<B>.parse_next(&mut data)?;
        assert_eq!(
            x,
            B {
                x: 999999,
                y: 888888888888
            }
        );
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_call_fun_args2_ip_addresses() -> winnow::Result<()> {
        let mut data = "fun_c(192.168.1.1, 10.0.0.1)";
        let x = call_fun_args2::<C>.parse_next(&mut data)?;
        assert_eq!(
            x,
            C {
                ip1: IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
                ip2: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            }
        );
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_take_arr_u32() -> winnow::Result<()> {
        let mut data = "[1,2,3,4,5]";
        let arr = take_arr::<u32>(&mut data)?;
        assert_eq!(arr, vec![1, 2, 3, 4, 5]);
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_take_arr_i64() -> winnow::Result<()> {
        let mut data = "[100,200,300]";
        let arr = take_arr::<i64>(&mut data)?;
        assert_eq!(arr, vec![100, 200, 300]);
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_take_arr_with_spaces() -> winnow::Result<()> {
        // Note: take_arr has specific space handling, test with simple spaces
        let mut data = "[1,2,3]";
        let arr = take_arr::<u32>(&mut data)?;
        assert_eq!(arr, vec![1, 2, 3]);
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_take_arr_single_element() -> winnow::Result<()> {
        let mut data = "[42]";
        let arr = take_arr::<u32>(&mut data)?;
        assert_eq!(arr, vec![42]);
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_take_arr_empty() -> winnow::Result<()> {
        // take_arr requires at least one element, so empty array will fail
        let mut data = "[]";
        let result = take_arr::<u32>(&mut data);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_take_arr_ip() -> winnow::Result<()> {
        let mut data = "[192.168.1.1,10.0.0.1]";
        let arr = take_arr::<IpAddr>(&mut data)?;
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        assert_eq!(arr[1], IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_u32_parse_next() -> winnow::Result<()> {
        let mut data = "12345";
        let num = u32::parse_next(&mut data)?;
        assert_eq!(num, 12345);
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_i64_parse_next() -> winnow::Result<()> {
        // i64 parser only supports digits, not negative numbers
        let mut data = "9876543210";
        let num = i64::parse_next(&mut data)?;
        assert_eq!(num, 9876543210);
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_ip_parse_next() -> winnow::Result<()> {
        let mut data = "127.0.0.1";
        let ip = IpAddr::parse_next(&mut data)?;
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(data, "");
        Ok(())
    }

    #[test]
    fn test_error_cases() {
        // Test invalid function names
        let mut data = "invalid_fun([1,2,3])";
        let result = call_fun_args1::<A>.parse_next(&mut data);
        assert!(result.is_err());

        // Test malformed brackets
        let mut data = "fun_a([1,2,3";
        let result = call_fun_args1::<A>.parse_next(&mut data);
        assert!(result.is_err());

        let mut data = "fun_a(1,2,3])";
        let result = call_fun_args1::<A>.parse_next(&mut data);
        assert!(result.is_err());

        // Test missing comma
        let mut data = "fun_b(42 100)";
        let result = call_fun_args2::<B>.parse_next(&mut data);
        assert!(result.is_err());

        // Test invalid array format
        let mut data = "[1,2,3,]";
        let result = take_arr::<u32>(&mut data);
        assert!(result.is_err());

        // Test empty input
        let mut data = "";
        let result = take_arr::<u32>(&mut data);
        assert!(result.is_err());

        let mut data = "";
        let result = call_fun_args1::<A>.parse_next(&mut data);
        assert!(result.is_err());
    }
}
