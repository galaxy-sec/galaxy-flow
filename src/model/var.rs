use std::collections::HashMap;
use std::env::Vars;
use std::fmt::{Debug, Display};

use orion_variate::vars::{ValueDict, ValueType};
use unicase::UniCase;

use super::execution::DictUse;
use super::sec::{NoSecConv, SecFrom, SecValueType, ToUniCase, ValueGetter};
use super::traits::{Getter, Setter};

#[derive(Clone, PartialEq, Eq)]
pub enum VarMeta {
    Security,
    Normal,
}

pub type UniString = UniCase<String>;
pub type UniCaseMap<T> = HashMap<UniCase<String>, T>;
#[derive(Debug, Clone, Default, Getters, PartialEq)]
pub struct VarDict {
    useage: DictUse,
    maps: UniCaseMap<SecValueType>,
}

impl From<Vars> for VarDict {
    fn from(value: Vars) -> Self {
        let mut maps = UniCaseMap::new();
        for (k, v) in value {
            maps.insert(UniCase::from(k), SecValueType::nor_from(v));
        }
        Self {
            useage: DictUse::Global,
            maps,
        }
    }
}
impl From<ValueDict> for VarDict {
    fn from(data: ValueDict) -> Self {
        let mut dict = Self::default();
        for (k, var_def) in data.dict().clone() {
            dict.set(k, SecValueType::nor_from(var_def));
        }
        dict
    }
}

impl Display for VarDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut keys: Vec<_> = self.maps().keys().collect();
        keys.sort(); // 对键进行排序
        for k in keys {
            if let Some(v) = self.maps.get(k) {
                writeln!(f, "{k:30}: {v}")?;
            }
        }
        Ok(())
    }
}

impl VarDict {
    pub fn global_new() -> Self {
        VarDict {
            useage: DictUse::Global,
            maps: HashMap::new(),
        }
    }
    pub fn new<S: Into<String>>(name: S) -> Self {
        VarDict {
            useage: DictUse::Named(name.into()),
            maps: HashMap::new(),
        }
    }

    pub fn export(&self) -> HashMap<String, ValueType> {
        let mut map = HashMap::new();
        for (k, v) in &self.maps {
            map.insert(k.to_uppercase(), v.clone().no_sec());
        }
        map
    }
    //todo sec  to nor
    pub fn export_str_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (k, v) in &self.maps {
            map.insert(k.to_uppercase(), v.to_string());
        }
        map
    }
    pub fn merge(&mut self, map: UniCaseMap<SecValueType>) {
        for (k, v) in map {
            self.maps.insert(k, v);
        }
    }
    pub fn merge_dict(&mut self, dict: Self) {
        for (k, v) in dict.maps {
            self.maps.insert(k, v);
        }
    }
    pub fn sec_set<S: Into<String>>(&mut self, key: S, val: ValueType) {
        self.maps
            .insert(UniCase::from(key.into()), SecValueType::sec_from(val));
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.maps().contains_key(&UniCase::from(key.to_string()))
    }
    pub fn is_empty(&self) -> bool {
        self.maps().is_empty()
    }
}

impl From<HashMap<String, String>> for VarDict {
    fn from(map: HashMap<String, String>) -> Self {
        let mut dict = VarDict::global_new();
        for (k, v) in map {
            dict.set(k, v);
        }
        dict
    }
}
impl Getter<&UniString, SecValueType> for VarDict {
    fn must_get(&self, key: &UniString) -> &SecValueType {
        if let Some(val) = self.maps.get(key) {
            val
        } else {
            panic!("un get key {key}",);
        }
    }
    fn get(&self, key: &UniString) -> Option<&SecValueType> {
        self.maps.value_get(key)
    }
}
impl Getter<&str, SecValueType> for VarDict {
    fn must_get(&self, key: &str) -> &SecValueType {
        if let Some(val) = self.maps.get(&key.to_unicase()) {
            val
        } else {
            panic!("un get key {key}");
        }
    }
    fn get(&self, key: &str) -> Option<&SecValueType> {
        self.maps.value_get(key)
    }
}

impl Setter<&String, String> for VarDict {
    fn set(&mut self, key: &String, val: String) {
        //self.maps.insert(key.clone(), val);
        self.maps
            .insert(key.to_unicase(), SecValueType::nor_from(val));
    }
}

impl Setter<String, String> for VarDict {
    fn set(&mut self, key: String, val: String) {
        self.maps
            .insert(key.to_unicase(), SecValueType::nor_from(val));
    }
}
impl Setter<&str, SecValueType> for VarDict {
    fn set(&mut self, key: &str, val: SecValueType) {
        self.maps.insert(key.to_unicase(), val);
    }
}
impl Setter<&String, SecValueType> for VarDict {
    fn set(&mut self, key: &String, val: SecValueType) {
        self.maps.insert(key.to_unicase(), val);
    }
}
impl Setter<String, SecValueType> for VarDict {
    fn set(&mut self, key: String, val: SecValueType) {
        self.maps.insert(key.to_unicase(), val);
    }
}

impl Setter<&str, String> for VarDict {
    fn set(&mut self, key: &str, val: String) {
        self.maps
            .insert(key.to_unicase(), SecValueType::nor_from(val));
    }
}

impl Setter<&str, &str> for VarDict {
    fn set(&mut self, key: &str, val: &str) {
        self.maps.insert(
            UniString::from(key.to_string()),
            SecValueType::nor_from(val.to_string()),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn context_use() {
        let mut def = VarDict::default();
        def.set("src", "hello src");
        def.set("dst", "hello dst");
        let src = def.must_get("src");
        let dst = def.must_get("dst");
        assert_eq!(*src.to_string(), String::from("hello src"));
        assert_eq!(*dst.to_string(), String::from("hello dst"));
    }
}
