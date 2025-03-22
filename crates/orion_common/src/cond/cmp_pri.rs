use std::net::IpAddr;

use chrono::NaiveDateTime;
use ipnet::IpNet;

use crate::cond::WildMatchAble;

impl WildMatchAble for String {
    fn wild_match(&self, other: &Self) -> bool {
        wildmatch::WildMatch::new(self.as_str()).matches(other.as_str())
    }
}

impl WildMatchAble for i64 {
    fn wild_match(&self, other: &Self) -> bool {
        *self == *other
    }
}
impl WildMatchAble for bool {
    fn wild_match(&self, other: &Self) -> bool {
        *self == *other
    }
}

impl WildMatchAble for u32 {
    fn wild_match(&self, other: &Self) -> bool {
        *self == *other
    }
}
impl WildMatchAble for u128 {
    fn wild_match(&self, other: &Self) -> bool {
        *self == *other
    }
}
impl WildMatchAble for u64 {
    fn wild_match(&self, other: &Self) -> bool {
        *self == *other
    }
}

impl WildMatchAble for f64 {
    fn wild_match(&self, other: &Self) -> bool {
        if *self > *other {
            (*self - *other) < 0.0001
        } else {
            (*other - *self) < 0.0001
        }
    }
}

impl WildMatchAble for IpAddr {
    fn wild_match(&self, other: &Self) -> bool {
        *self == *other
    }
}

impl WildMatchAble for IpNet {
    fn wild_match(&self, other: &Self) -> bool {
        *self == *other
    }
}

impl WildMatchAble for NaiveDateTime {
    fn wild_match(&self, other: &Self) -> bool {
        *self == *other
    }
}
