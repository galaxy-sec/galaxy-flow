use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
    net::IpAddr,
};

use derive_more::From;
use orion_variate::vars::ValueType;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SecValue<T> {
    is_secret: bool,
    value: T,
}
impl<T> SecValue<T> {
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn is_secret(&self) -> bool {
        self.is_secret
    }
}

pub trait NoSecConv<T> {
    fn no_sec(self) -> T;
}
pub trait SecConv {
    fn to_nor(self) -> Self;
    fn to_sec(self) -> Self;
}

impl<T> SecConv for SecValue<T> {
    fn to_nor(mut self) -> Self {
        self.is_secret = false;
        self
    }
    fn to_sec(mut self) -> Self {
        self.is_secret = true;
        self
    }
}

impl<T> SecConv for Vec<SecValue<T>> {
    fn to_nor(mut self) -> Self {
        self.iter_mut().for_each(|x| x.is_secret = false);
        self
    }

    fn to_sec(mut self) -> Self {
        self.iter_mut().for_each(|x| x.is_secret = true);
        self
    }
}

impl<T> SecConv for HashMap<String, SecValue<T>> {
    fn to_nor(mut self) -> Self {
        self.iter_mut().for_each(|(_, x)| x.is_secret = false);
        self
    }

    fn to_sec(mut self) -> Self {
        self.iter_mut().for_each(|(_, x)| x.is_secret = true);
        self
    }
}
impl SecFrom<HashMap<String, ValueType>> for SecValueType {
    fn sec_from(value: HashMap<String, ValueType>) -> Self {
        SecValueType::Obj(
            value
                .into_iter()
                .map(|(k, v)| (k, SecValueType::sec_from(v)))
                .collect(),
        )
    }

    fn nor_from(value: HashMap<String, ValueType>) -> Self {
        SecValueType::Obj(
            value
                .into_iter()
                .map(|(k, v)| (k, SecValueType::nor_from(v)))
                .collect(),
        )
    }
}
impl SecFrom<Vec<ValueType>> for SecValueType {
    fn sec_from(value: Vec<ValueType>) -> Self {
        SecValueType::List(value.into_iter().map(SecValueType::sec_from).collect())
    }

    fn nor_from(value: Vec<ValueType>) -> Self {
        SecValueType::List(value.into_iter().map(SecValueType::nor_from).collect())
    }
}
impl SecFrom<ValueType> for SecValueType {
    fn nor_from(value: ValueType) -> Self {
        match value {
            ValueType::String(v) => SecValueType::nor_from(v),
            ValueType::Bool(v) => SecValueType::nor_from(v),
            ValueType::Number(v) => SecValueType::nor_from(v),
            ValueType::Float(v) => SecValueType::nor_from(v),
            ValueType::Ip(v) => SecValueType::nor_from(v),
            ValueType::Obj(v) => SecValueType::nor_from(v),
            ValueType::List(v) => SecValueType::nor_from(v),
        }
    }

    fn sec_from(value: ValueType) -> Self {
        match value {
            ValueType::String(v) => SecValueType::sec_from(v),
            ValueType::Bool(v) => SecValueType::sec_from(v),
            ValueType::Number(v) => SecValueType::sec_from(v),
            ValueType::Float(v) => SecValueType::sec_from(v),
            ValueType::Ip(v) => SecValueType::sec_from(v),
            ValueType::Obj(v) => SecValueType::sec_from(v),
            ValueType::List(v) => SecValueType::sec_from(v),
        }
    }
}
pub trait SecFrom<T> {
    fn sec_from(value: T) -> Self;
    fn nor_from(value: T) -> Self;
}
impl<T> SecFrom<T> for SecValue<T> {
    fn sec_from(value: T) -> Self {
        Self {
            is_secret: true,
            value,
        }
    }
    fn nor_from(value: T) -> Self {
        Self {
            is_secret: false,
            value,
        }
    }
}
impl<T> Display for SecValue<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_secret {
            write!(f, "***")
        } else {
            write!(f, "{}", self.value)
        }
    }
}
pub type SecString = SecValue<String>;
pub type SecBool = SecValue<bool>;
pub type SecIpAddr = SecValue<IpAddr>;
pub type SecU64 = SecValue<u64>;
pub type SecF64 = SecValue<f64>;
pub type SecValueObj = HashMap<String, SecValueType>;
pub type SecValueVec = Vec<SecValueType>;

impl Display for SecValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SecValueType::String(v) => write!(f, "{v}"),
            SecValueType::Bool(v) => write!(f, "{v}"),
            SecValueType::Number(v) => write!(f, "{v}"),
            SecValueType::Float(v) => write!(f, "{v}"),
            SecValueType::Ip(v) => write!(f, "{v}"),
            SecValueType::Obj(v) => write!(f, "obj:{v:#?}"),
            SecValueType::List(v) => write!(f, "list:{v:#?}"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, From)]
pub enum SecValueType {
    String(SecString),
    Bool(SecBool),
    Number(SecU64),
    Float(SecF64),
    Ip(SecIpAddr),
    Obj(SecValueObj),
    List(SecValueVec),
}

impl<T> SecFrom<T> for SecValueType
where
    SecValue<T>: SecFrom<T>,
    SecValueType: From<SecValue<T>>,
{
    fn sec_from(value: T) -> Self {
        SecValueType::from(SecValue::sec_from(value))
    }

    fn nor_from(value: T) -> Self {
        SecValueType::from(SecValue::nor_from(value))
    }
}

impl SecValueType {
    pub fn to_nor(self) -> Self {
        match self {
            SecValueType::String(v) => Self::String(v.to_nor()),
            SecValueType::Bool(v) => Self::Bool(v),
            SecValueType::Number(v) => Self::Number(v.to_nor()),
            SecValueType::Float(v) => Self::Float(v.to_nor()),
            SecValueType::Ip(v) => Self::Ip(v.to_nor()),
            SecValueType::Obj(v) => Self::Obj(v.to_nor()),
            SecValueType::List(v) => Self::List(v.to_nor()),
        }
    }
    pub fn to_sec(self) -> Self {
        match self {
            SecValueType::String(v) => Self::String(v.to_sec()),
            SecValueType::Bool(v) => Self::Bool(v),
            SecValueType::Number(v) => Self::Number(v.to_sec()),
            SecValueType::Float(v) => Self::Float(v.to_sec()),
            SecValueType::Ip(v) => Self::Ip(v.to_sec()),
            SecValueType::Obj(v) => Self::Obj(v.to_sec()),
            SecValueType::List(v) => Self::List(v.to_sec()),
        }
    }
}

impl SecConv for Vec<SecValueType> {
    fn to_nor(self) -> Self {
        self.into_iter().map(|x| x.to_nor()).collect()
    }

    fn to_sec(self) -> Self {
        self.into_iter().map(|x| x.to_sec()).collect()
    }
}

impl SecConv for HashMap<String, SecValueType> {
    fn to_nor(self) -> Self {
        self.into_iter().map(|(k, x)| (k, x.to_nor())).collect()
    }

    fn to_sec(self) -> Self {
        self.into_iter().map(|(k, x)| (k, x.to_sec())).collect()
    }
}

impl NoSecConv<ValueType> for SecValueType {
    fn no_sec(self) -> ValueType {
        match self {
            SecValueType::String(v) => ValueType::from(v.value),
            SecValueType::Bool(v) => ValueType::from(v.value),
            SecValueType::Number(v) => ValueType::from(v.value),
            SecValueType::Float(v) => ValueType::from(v.value),
            SecValueType::Ip(v) => ValueType::from(v.value),
            SecValueType::Obj(v) => ValueType::from(v.no_sec()),
            SecValueType::List(v) => ValueType::from(v.no_sec()),
        }
    }
}

impl NoSecConv<Vec<ValueType>> for Vec<SecValueType> {
    fn no_sec(self) -> Vec<ValueType> {
        self.into_iter().map(|x| x.no_sec()).collect()
    }
}

impl NoSecConv<HashMap<String, ValueType>> for HashMap<String, SecValueType> {
    fn no_sec(self) -> HashMap<String, ValueType> {
        self.into_iter().map(|(k, x)| (k, x.no_sec())).collect()
    }
}
pub trait ObjGetter<T> {
    fn obj_get(&self, path: &str) -> Option<&T>;
}
impl ObjGetter<SecValueType> for SecValueObj {
    fn obj_get(&self, path: &str) -> Option<&SecValueType> {
        let keys: Vec<&str> = path.split('.').collect();
        if keys.is_empty() {
            return None;
        }

        let mut current_map = self;
        for (i, key) in keys.iter().enumerate() {
            if let Some(value) = current_map.get(*key) {
                if i == keys.len() - 1 {
                    return Some(value);
                } else if let SecValueType::Obj(next_map) = value {
                    current_map = next_map;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[test]
    fn test_sec_value_display() {
        let secret_str = SecString::sec_from("password".to_string());
        assert_eq!(format!("{secret_str}"), "***");

        let public_str = SecString::nor_from("username".to_string());
        assert_eq!(format!("{public_str}"), "username");
    }

    #[test]
    fn test_sec_value_type_conversions() {
        // Test basic type conversions
        let secret_num = SecValueType::sec_from(42u64);
        assert!(matches!(secret_num, SecValueType::Number(v) if v.is_secret && v.value == 42));

        let public_bool = SecValueType::nor_from(true);
        assert!(matches!(public_bool, SecValueType::Bool(v) if !v.is_secret && v.value));

        // Test IP conversion
        let ip = IpAddr::from_str("192.168.1.1").unwrap();
        let secret_ip = SecValueType::sec_from(ip);
        assert!(matches!(secret_ip, SecValueType::Ip(v) if v.is_secret));
    }

    #[test]
    fn test_nested_conversions() {
        // Test nested objects
        let mut obj = HashMap::new();
        obj.insert("key".to_string(), ValueType::String("value".to_string()));

        let secret_obj = SecValueType::sec_from(obj.clone());
        if let SecValueType::Obj(map) = secret_obj {
            assert!(map["key"].is_secret());
        } else {
            panic!("Expected Obj variant");
        }

        // Test lists
        let list = vec![ValueType::Bool(true), ValueType::Number(10)];
        let public_list = SecValueType::nor_from(list.clone());
        if let SecValueType::List(vec) = public_list {
            assert!(!vec[0].is_secret());
        } else {
            panic!("Expected List variant");
        }
    }

    #[test]
    fn test_sec_conv_traits() {
        // Test vector conversion
        let values = vec![SecValueType::nor_from(10u64), SecValueType::sec_from(20u64)];

        let secret_values = values.clone().to_sec();
        for val in secret_values {
            assert!(val.is_secret());
        }

        let public_values = values.to_nor();
        for val in public_values {
            assert!(!val.is_secret());
        }
    }

    #[test]
    fn test_no_sec_conv() {
        // Test conversion back to normal types
        let secret_str = SecValueType::sec_from("secret".to_string());
        let normal_str: ValueType = secret_str.no_sec();
        assert_eq!(normal_str, ValueType::String("secret".to_string()));

        // Test nested conversion
        let mut obj = HashMap::new();
        obj.insert("nested".to_string(), SecValueType::nor_from(100u64));
        let sec_obj = SecValueType::Obj(obj);

        if let ValueType::Obj(normal_obj) = sec_obj.no_sec() {
            assert_eq!(normal_obj["nested"], ValueType::Number(100));
        }
    }

    // Helper to check if a SecValueType is secret
    trait SecretCheck {
        fn is_secret(&self) -> bool;
    }

    impl SecretCheck for SecValueType {
        fn is_secret(&self) -> bool {
            match self {
                SecValueType::String(v) => v.is_secret,
                SecValueType::Bool(v) => v.is_secret,
                SecValueType::Number(v) => v.is_secret,
                SecValueType::Float(v) => v.is_secret,
                SecValueType::Ip(v) => v.is_secret,
                SecValueType::Obj(_) => false, // Objects don't have direct secret flag
                SecValueType::List(_) => false, // Lists don't have direct secret flag
            }
        }
    }
}
