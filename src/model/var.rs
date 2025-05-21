use std::collections::HashMap;
use std::fmt::{Debug, Display};

use super::traits::{Getter, Setter};

#[derive(Clone, PartialEq, Eq)]
pub enum VarMeta {
    Security,
    Normal,
}

#[derive(Clone, PartialEq)]
pub struct SecVar {
    meta: VarMeta,
    value: String,
}

#[derive(Debug, Clone, Default, Getters, PartialEq)]
pub struct VarsDict {
    maps: HashMap<String, SecVar>,
}

impl SecVar {
    pub fn new(meta: VarMeta, value: String) -> Self {
        SecVar { meta, value }
    }
    pub fn value(&self) -> &String {
        &self.value
    }
}
impl Display for SecVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.meta {
            VarMeta::Security => write!(f, "******"),
            VarMeta::Normal => write!(f, "{}", self.value),
        }
    }
}
impl Debug for SecVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.meta {
            VarMeta::Security => write!(f, "******"),
            VarMeta::Normal => write!(f, "{}", self.value),
        }
    }
}

impl Display for VarsDict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (k, v) in &self.maps {
            writeln!(f, "{:30}: {}", k, v)?;
        }
        Ok(())
    }
}

impl VarsDict {
    pub fn new() -> Self {
        VarsDict {
            maps: HashMap::new(),
        }
    }
    pub fn export(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (k, v) in &self.maps {
            map.insert(k.clone(), v.value.clone());
        }
        map
    }
    pub fn merge(&mut self, meta: VarMeta, map: HashMap<String, String>) {
        for (k, v) in map {
            self.maps.insert(
                k,
                SecVar {
                    meta: meta.clone(),
                    value: v,
                },
            );
        }
    }
    pub fn merge_dict(&mut self, dict: Self) {
        for (k, v) in dict.maps {
            self.maps.insert(k, v);
        }
    }
    pub fn sec_set<S: Into<String>>(&mut self, key: &str, val: S) {
        self.maps.insert(
            key.to_string(),
            SecVar {
                meta: VarMeta::Security,
                value: val.into(),
            },
        );
    }
}

impl From<HashMap<String, String>> for VarsDict {
    fn from(map: HashMap<String, String>) -> Self {
        let mut dict = VarsDict::new();
        for (k, v) in map {
            dict.set(&k, v);
        }
        dict
    }
}
impl Getter<&String, SecVar> for VarsDict {
    fn must_get(&self, key: &String) -> &SecVar {
        if let Some(val) = self.maps.get(key) {
            val
        } else {
            panic!("un get key {}", key);
        }
    }
    fn get(&self, key: &String) -> Option<&SecVar> {
        self.maps.get(key)
    }
}
impl Getter<&str, SecVar> for VarsDict {
    fn must_get(&self, key: &str) -> &SecVar {
        if let Some(val) = self.maps.get(key) {
            val
        } else {
            panic!("un get key {}", key);
        }
    }
    fn get(&self, key: &str) -> Option<&SecVar> {
        self.maps.get(key)
    }
}

impl Setter<&String, String> for VarsDict {
    fn set(&mut self, key: &String, val: String) {
        //self.maps.insert(key.clone(), val);
        self.maps.insert(
            key.to_string(),
            SecVar {
                meta: VarMeta::Normal,
                value: val,
            },
        );
    }
}

impl Setter<&str, String> for VarsDict {
    fn set(&mut self, key: &str, val: String) {
        self.maps.insert(
            key.to_string(),
            SecVar {
                meta: VarMeta::Normal,
                value: val,
            },
        );
    }
}

impl Setter<&str, &str> for VarsDict {
    fn set(&mut self, key: &str, val: &str) {
        self.maps.insert(
            key.to_string(),
            SecVar {
                meta: VarMeta::Normal,
                value: val.to_string(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn context_use() {
        let mut def = VarsDict::default();
        def.set("src", format!("hello src"));
        def.set("dst", "hello dst");
        let src = def.must_get("src");
        let dst = def.must_get("dst");
        assert_eq!(*src.value, String::from("hello src"));
        assert_eq!(*dst.value, String::from("hello dst"));
    }
}
