use orion_sec::sec::ValueGetter;
use std::collections::HashMap;
use std::env::Vars;
use std::fmt::{Debug, Display};

use indexmap::IndexMap;
use orion_variate::vars::{EnvDict, UpperKey, ValueDict, ValueType};
use unicase::UniCase;

use super::execution::DictUse;
use super::traits::{Getter, Setter};
use orion_sec::sec::{NoSecConv, SecFrom, SecValueObj, SecValueType};

#[derive(Clone, PartialEq, Eq)]
pub enum VarMeta {
    Security,
    Normal,
}

pub type UniString = UniCase<String>;
pub type UpperStrMap<T> = IndexMap<UpperKey, T>;
#[derive(Debug, Clone, Default, Getters, PartialEq)]
pub struct VarDict {
    useage: DictUse,
    maps: UpperStrMap<SecValueType>,
}

impl From<Vars> for VarDict {
    fn from(value: Vars) -> Self {
        let mut maps = UpperStrMap::new();
        for (k, v) in value {
            maps.insert(UpperKey::from(k), SecValueType::nor_from(v));
        }
        Self {
            useage: DictUse::Global,
            maps,
        }
    }
}
impl From<IndexMap<UpperKey, SecValueType>> for VarDict {
    fn from(value: IndexMap<UpperKey, SecValueType>) -> Self {
        Self {
            useage: DictUse::Global,
            maps: value,
        }
    }
}

impl From<ValueDict> for VarDict {
    fn from(data: ValueDict) -> Self {
        let mut dict = Self::default();
        for (k, var_def) in data.dict().clone() {
            dict.set(k.as_str(), SecValueType::nor_from(var_def));
        }
        dict
    }
}

impl Display for VarDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let keys: Vec<_> = self.maps().keys().collect();
        //keys.sort(); // 对键进行排序
        for k in keys {
            if let Some(v) = self.maps.get(k) {
                writeln!(f, "{:30}: {v}", k.as_str())?;
            }
        }
        Ok(())
    }
}

impl From<VarDict> for EnvDict {
    fn from(value: VarDict) -> Self {
        let mut dict = EnvDict::new();
        for (k, v) in value.maps {
            dict.insert(k.clone(), v.no_sec());
        }
        dict
    }
}
impl VarDict {
    pub fn global_new() -> Self {
        VarDict {
            useage: DictUse::Global,
            maps: IndexMap::new(),
        }
    }
    pub fn new<S: Into<String>>(name: S) -> Self {
        VarDict {
            useage: DictUse::Named(name.into()),
            maps: IndexMap::new(),
        }
    }

    pub fn export(&self) -> IndexMap<UpperKey, ValueType> {
        let mut map: IndexMap<UpperKey, ValueType> = IndexMap::new();
        for (k, v) in &self.maps {
            map.insert(UpperKey::from(k.as_str()), v.clone().no_sec());
        }
        map
    }
    //todo sec  to nor
    pub fn export_str_map(&self) -> IndexMap<String, String> {
        let data = self.maps.clone();
        let mut map: IndexMap<String, String> = IndexMap::new();
        for (k, v) in data {
            map.insert(k.as_str().to_string(), v.to_string());
        }
        map
    }
    pub fn merge(&mut self, map: UpperStrMap<SecValueType>) {
        for (k, v) in map {
            self.maps.insert(k, v);
        }
    }
    pub fn merge_dict(&mut self, dict: Self) {
        for (k, v) in dict.maps {
            self.maps.insert(k, v);
        }
    }
    pub fn merge_item_obj(&mut self, key: &str, obj: SecValueObj) {
        if let Some(SecValueType::Obj(found)) = self.maps.get_mut(key) {
            for (k, v) in obj {
                found.insert(k, v);
            }
        } else {
            self.maps
                .insert(UpperKey::from(key), SecValueType::from(obj));
            //unreachable!("merge item_obj  miss or not obj");
        }
    }

    pub fn sec_set<S: Into<String>>(&mut self, key: S, val: ValueType) {
        self.maps
            .insert(UpperKey::from(key.into()), SecValueType::sec_from(val));
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.maps().contains_key(&UpperKey::from(key.to_string()))
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
impl Getter<&UpperKey, SecValueType> for VarDict {
    fn must_get(&self, key: &UpperKey) -> &SecValueType {
        self.maps
            .get(key)
            .unwrap_or_else(|| panic!("un get key {}", key.as_str()))
    }
    fn get_copy(&self, key: &UpperKey) -> Option<SecValueType> {
        self.maps.value_get(key.as_str())
    }
}
impl Getter<&str, SecValueType> for VarDict {
    fn must_get(&self, key: &str) -> &SecValueType {
        self.maps
            .get(&key.to_uppercase())
            .unwrap_or_else(|| panic!("un get key {key}"))
    }
    fn get_copy(&self, key: &str) -> Option<SecValueType> {
        self.maps.value_get(key)
    }
}

impl Setter<&String, String> for VarDict {
    fn set(&mut self, key: &String, val: String) {
        //self.maps.insert(key.clone(), val);
        self.maps
            .insert(UpperKey::from(key), SecValueType::nor_from(val));
    }
}

impl Setter<String, String> for VarDict {
    fn set(&mut self, key: String, val: String) {
        self.maps
            .insert(UpperKey::from(key), SecValueType::nor_from(val));
    }
}
impl Setter<&str, SecValueType> for VarDict {
    fn set(&mut self, key: &str, val: SecValueType) {
        self.maps.insert(UpperKey::from(key), val);
    }
}
impl Setter<&String, SecValueType> for VarDict {
    fn set(&mut self, key: &String, val: SecValueType) {
        self.maps.insert(UpperKey::from(key), val);
    }
}
impl Setter<String, SecValueType> for VarDict {
    fn set(&mut self, key: String, val: SecValueType) {
        self.maps.insert(UpperKey::from(key), val);
    }
}

impl Setter<&str, String> for VarDict {
    fn set(&mut self, key: &str, val: String) {
        self.maps
            .insert(UpperKey::from(key), SecValueType::nor_from(val));
    }
}
impl Setter<&str, bool> for VarDict {
    fn set(&mut self, key: &str, val: bool) {
        self.maps
            .insert(UpperKey::from(key), SecValueType::nor_from(val));
    }
}

impl Setter<&str, &str> for VarDict {
    fn set(&mut self, key: &str, val: &str) {
        self.maps
            .insert(UpperKey::from(key), SecValueType::nor_from(val.to_string()));
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
        let src = def.must_get("SrC");
        let dst = def.must_get("DST");
        assert_eq!(*src.to_string(), String::from("hello src"));
        assert_eq!(*dst.to_string(), String::from("hello dst"));
    }
}
