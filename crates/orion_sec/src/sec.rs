use std::{
    fmt::{Display, Formatter},
    net::IpAddr,
};

use derive_more::From;
use indexmap::IndexMap;
use orion_variate::vars::ValueType;
use serde_derive::{Deserialize, Serialize};
use unicase::UniCase;

use crate::types::{UniCaseMap, UniString};

pub trait ToUniCase<T> {
    fn to_unicase(self) -> UniCase<T>;
}

impl ToUniCase<String> for &str {
    fn to_unicase(self) -> UniCase<String> {
        UniString::from(self.to_string())
    }
}

impl ToUniCase<String> for String {
    fn to_unicase(self) -> UniCase<String> {
        UniString::from(self)
    }
}
impl ToUniCase<String> for &String {
    fn to_unicase(self) -> UniCase<String> {
        UniString::from(self.clone())
    }
}
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
impl<T> PartialOrd for SecValue<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

pub trait NoSecConv<T> {
    fn no_sec(self) -> T;
}
pub trait SecConv {
    #[must_use]
    fn to_nor(self) -> Self;
    #[must_use]
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
        for x in self.iter_mut() {
            x.is_secret = false;
        }
        self
    }

    fn to_sec(mut self) -> Self {
        for x in self.iter_mut() {
            x.is_secret = true;
        }
        self
    }
}

impl<T> SecConv for UniCaseMap<SecValue<T>> {
    fn to_nor(mut self) -> Self {
        self.iter_mut().for_each(|(_, x)| x.is_secret = false);
        self
    }

    fn to_sec(mut self) -> Self {
        self.iter_mut().for_each(|(_, x)| x.is_secret = true);
        self
    }
}
impl SecFrom<IndexMap<String, ValueType>> for SecValueType {
    fn sec_from(value: IndexMap<String, ValueType>) -> Self {
        SecValueType::Obj(
            value
                .into_iter()
                .map(|(k, v)| (UniString::from(k), SecValueType::sec_from(v)))
                .collect(),
        )
    }

    fn nor_from(value: IndexMap<String, ValueType>) -> Self {
        SecValueType::Obj(
            value
                .into_iter()
                .map(|(k, v)| (UniString::from(k), SecValueType::nor_from(v)))
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
pub type SecValueObj = UniCaseMap<SecValueType>;
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

impl PartialOrd for SecValueType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (SecValueType::String(f), SecValueType::String(s)) => f.partial_cmp(s),
            (SecValueType::Bool(f), SecValueType::Bool(s)) => f.partial_cmp(s),
            (SecValueType::Number(f), SecValueType::Number(s)) => f.partial_cmp(s),
            (SecValueType::Float(f), SecValueType::Float(s)) => f.partial_cmp(s),
            (SecValueType::Ip(f), SecValueType::Ip(s)) => f.partial_cmp(s),
            _ => {
                unreachable!("not support array , obj compare");
            }
        }
    }
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
    #[must_use]
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
    #[must_use]
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
        self.into_iter().map(SecValueType::to_nor).collect()
    }

    fn to_sec(self) -> Self {
        self.into_iter().map(SecValueType::to_sec).collect()
    }
}

impl SecConv for UniCaseMap<SecValueType> {
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
        self.into_iter().map(NoSecConv::no_sec).collect()
    }
}

impl NoSecConv<IndexMap<String, ValueType>> for UniCaseMap<SecValueType> {
    fn no_sec(self) -> IndexMap<String, ValueType> {
        self.into_iter()
            .map(|(k, x)| (k.to_uppercase(), x.no_sec()))
            .collect()
    }
}
pub trait ValueGetter<T> {
    fn value_get(&self, path: &str) -> Option<T>;
}

impl ValueGetter<SecValueType> for SecValueObj {
    fn value_get(&self, path: &str) -> Option<SecValueType> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        //let mut current_value: Option<&SecValueType> = None;
        let mut current_target = SecValueType::from(self.clone());

        for part in parts {
            // 检查是否为数组访问（如 "A[0]"）
            if part.contains('[') && part.ends_with(']') {
                let (key, index_str) = part.split_once('[').unwrap();
                let index_str = index_str.trim_end_matches(']');
                let index = index_str.parse::<usize>().ok()?;

                if let SecValueType::Obj(obj) = current_target {
                    // 获取数组或对象
                    let value = obj.get(&UniString::from(key.to_string()))?;
                    match value {
                        SecValueType::List(list) => {
                            if index >= list.len() {
                                return None;
                            }
                            current_target = list[index].clone();
                        }
                        _ => return None,
                    }
                } else {
                    return None;
                }
            } else {
                // 普通键访问
                if let SecValueType::Obj(obj) = current_target {
                    if let Some(found) = obj.get(&UniString::from(part.to_string())) {
                        current_target = found.clone();
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
        }
        Some(current_target)
    }
}

#[cfg(test)]
mod tests {

    use crate::types::UniCaseMap;

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
    fn test_obj_get_with_array() {
        let mut obj = UniCaseMap::new();
        let list = vec![
            SecValueType::nor_from(42u64),
            SecValueType::sec_from("secret".to_string()),
        ];
        obj.insert("A".into(), SecValueType::List(list));

        let mut nested_obj = UniCaseMap::new();
        nested_obj.insert("rust".into(), SecValueType::nor_from("awesome".to_string()));
        obj.insert("B".into(), SecValueType::Obj(nested_obj));

        // 测试数组访问
        assert_eq!(obj.value_get("A[0]"), Some(SecValueType::nor_from(42u64)));
        assert_eq!(
            obj.value_get("A[1]"),
            Some(SecValueType::sec_from("secret".to_string()))
        );

        // 测试嵌套路径
        assert_eq!(
            obj.value_get("B.rust"),
            Some(SecValueType::nor_from("awesome".to_string()))
        );
        assert_eq!(obj.value_get("B.rust.not_exists"), None);

        // 测试无效路径
        assert_eq!(obj.value_get("A[invalid]"), None);
        assert_eq!(obj.value_get("A[2]"), None); // 越界
    }
    #[test]
    fn test_obj_get_with_dot_notation() {
        // 构建测试数据
        let mut root = UniCaseMap::new();

        // 嵌套对象：root -> "a" -> "b" -> "c"
        let mut nested_c = UniCaseMap::new();
        nested_c.insert("c".into(), SecValueType::nor_from("value_c".to_string()));

        let mut nested_b = UniCaseMap::new();
        nested_b.insert("b".into(), SecValueType::Obj(nested_c.clone()));
        nested_b.insert("d".into(), SecValueType::nor_from(42u64));

        root.insert("a".into(), SecValueType::Obj(nested_b));
        root.insert("x".into(), SecValueType::nor_from("value_x".to_string()));

        // 测试用例
        // 1. 访问根节点直接键
        assert_eq!(
            root.value_get("x"),
            Some(SecValueType::nor_from("value_x".to_string()))
        );

        // 2. 访问单层嵌套路径 "a.b"
        if let Some(SecValueType::Obj(map)) = root.value_get("a") {
            assert_eq!(
                map.value_get("b"),
                Some(SecValueType::from(nested_c)) // 实际应为嵌套的 `nested_c`
            );
        } else {
            panic!("Expected 'a' to be an object");
        }

        // 3. 访问多层嵌套路径 "a.b.c"
        assert_eq!(
            root.value_get("a.b.c"),
            Some(SecValueType::nor_from("value_c".to_string()))
        );

        // 4. 访问路径中的非对象键（应返回 None）
        assert_eq!(root.value_get("a.d.c"), None); // "d" 是数字，无法继续访问 "c"

        // 5. 访问不存在的路径
        assert_eq!(root.value_get("not_exists"), None);
        assert_eq!(root.value_get("a.not_exists"), None);
        assert_eq!(root.value_get("a.b.not_exists"), None);
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
        let mut obj = IndexMap::new();
        obj.insert("key".to_string(), ValueType::String("value".to_string()));

        let secret_obj = SecValueType::sec_from(obj.clone());
        if let SecValueType::Obj(map) = secret_obj {
            assert!(map[&"key".to_unicase()].is_secret());
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
        let mut obj = UniCaseMap::new();
        obj.insert("nested".into(), SecValueType::nor_from(100u64));
        let sec_obj = SecValueType::Obj(obj);

        if let ValueType::Obj(normal_obj) = sec_obj.no_sec() {
            assert_eq!(normal_obj["NESTED"], ValueType::Number(100));
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
