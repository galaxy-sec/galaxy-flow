use std::path::PathBuf;

use derive_more::From;
use dirs::home_dir;

use crate::{
    sec::{ObjGetter, SecValueType},
    var::VarDict,
    ExecResult,
};

use super::global::{load_secfile, setup_gxlrun_vars, setup_start_vars};

#[derive(Debug, Clone, Default, PartialEq, From, Getters)]
pub struct VarSpace {
    inherited: VarDict,
    global: VarDict,
    //nameds: HashMap<String, VarDict>,
}

pub fn sec_value_default_path() -> PathBuf {
    galaxy_dot_path().join("sec_value.yml")
}
pub fn galaxy_dot_path() -> PathBuf {
    home_dir()
        .map(|x| x.join(".galaxy"))
        .unwrap_or(PathBuf::from("./"))
}

impl VarSpace {
    pub fn sys_init() -> ExecResult<VarSpace> {
        let mut var_space = VarSpace::default();
        load_secfile(&mut var_space.inherited)?;
        setup_start_vars(&mut var_space.inherited)?;
        setup_gxlrun_vars(&mut var_space.inherited)?;
        var_space.global = var_space.inherited.clone();
        Ok(var_space)
    }
    pub fn inherit_init(mut origin: VarSpace, isolate: bool) -> ExecResult<VarSpace> {
        if isolate {
            let mut ins = Self {
                inherited: origin.inherited.clone(),
                global: origin.inherited,
            };
            setup_gxlrun_vars(&mut ins.inherited)?;
            Ok(ins)
        } else {
            setup_gxlrun_vars(&mut origin.inherited)?;
            setup_gxlrun_vars(&mut origin.global)?;
            Ok(origin)
        }
    }
    pub fn global_mut(&mut self) -> &mut VarDict {
        &mut self.global
    }
    pub fn get(&self, path: &str) -> Option<&SecValueType> {
        self.global().maps().obj_get(path)
    }

    /*
    pub fn nameds_mut(&mut self) -> &mut HashMap<String, VarDict> {
        &mut self.nameds
    }
    */
}
#[derive(Debug, Clone, Default, PartialEq, From)]
pub enum DictUse {
    #[default]
    Global,
    Named(String),
}

#[cfg(test)]
mod tests {
    use crate::{
        execution::{dict::sec_value_default_path, global::load_secfile},
        sec::{SecFrom, ToUniCase},
        traits::{Getter, Setter},
        var::{UniCaseMap, VarDict},
    };

    use orion_error::TestAssertWithMsg;
    use std::{env::temp_dir, fs::File, io::Write};

    #[test]
    fn test_load_secfile_with_values() {
        // 创建临时目录和文件
        //let dir = PathBuf::from("./temp");
        let dir = temp_dir();
        let file_path = dir.join("sec_value.yml");
        if file_path.exists() {
            std::fs::remove_file(&file_path).assert("remove file");
        }

        // 写入测试内容
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "key1: value1\nkey2: value2").unwrap();

        // 创建 VarSpace 实例并加载文件
        let mut var_dict = VarDict::new("test");

        // 临时修改路径指向我们的测试文件
        let original_path = sec_value_default_path();
        std::env::set_var("GAL_SEC_FILE_PATH", file_path.to_str().unwrap());

        load_secfile(&mut var_dict).assert("load secfile");

        // 验证全局变量
        assert!(var_dict.contains_key("SEC_KEY1"));
        assert!(var_dict.contains_key("SEC_KEY2"));
        assert_eq!(
            format!("{}", var_dict.get("SEC_KEY1").unwrap()),
            "***".to_string()
        );
        // 清理
        std::env::set_var("GAL_SEC_FILE_PATH", original_path);
    }
    use super::*;
    use crate::sec::{SecString, SecValueType};

    #[test]
    fn test_get_top_level_key() {
        let mut var_space = VarSpace::default();
        var_space.global_mut().set(
            "name".to_string(),
            SecValueType::nor_from("Test User".to_string()),
        );

        let value = var_space.get("name");
        assert!(value.is_some());
        if let Some(SecValueType::String(s)) = value {
            assert_eq!(s.value(), "Test User");
        } else {
            panic!("Expected String variant");
        }
    }

    #[test]
    fn test_get_nested_key() {
        let mut var_space = VarSpace::default();

        // Create nested structure: user = { name: "Test", id: 42 }
        let mut user = UniCaseMap::new();
        user.insert(
            "name".to_unicase(),
            SecValueType::nor_from("Test User".to_string()),
        );
        user.insert("id".to_unicase(), SecValueType::nor_from(42u64));

        var_space.global_mut().set("user", SecValueType::Obj(user));

        // Test nested access
        let name = var_space.get("user.name");
        assert!(name.is_some());
        if let Some(SecValueType::String(s)) = name {
            assert_eq!(s.value(), "Test User");
        } else {
            panic!("Expected String variant");
        }

        let id = var_space.get("user.id");
        assert!(id.is_some());
        if let Some(SecValueType::Number(n)) = id {
            assert_eq!(*n.value(), 42);
        } else {
            panic!("Expected Number variant");
        }
    }

    #[test]
    fn test_get_multi_level_nested_key() {
        let mut var_space = VarSpace::default();

        // Create nested structure: app.user.profile.name
        let mut profile = UniCaseMap::new();
        profile.insert(
            "name".to_unicase(),
            SecValueType::nor_from("Test User".to_string()),
        );

        let mut user = UniCaseMap::new();
        user.insert("profile".to_unicase(), SecValueType::Obj(profile));

        var_space.global_mut().set("app", SecValueType::Obj(user));

        let name = var_space.get("app.profile.name");
        assert!(name.is_some());
        if let Some(SecValueType::String(s)) = name {
            assert_eq!(s.value(), "Test User");
        } else {
            panic!("Expected String variant");
        }
    }

    #[test]
    fn test_get_non_existent_key() {
        let mut var_space = VarSpace::default();
        var_space.global_mut().set("exists", "value");

        assert!(var_space.get("does_not_exist").is_none());
        assert!(var_space.get("exists.invalid").is_none());
    }

    #[test]
    fn test_get_key_in_middle_of_path() {
        let mut var_space = VarSpace::default();

        // Create structure: parent = { child: "value" }
        let mut parent = UniCaseMap::new();
        parent.insert(
            "child".to_unicase(),
            SecValueType::nor_from("value".to_string()),
        );

        var_space
            .global_mut()
            .set("parent", SecValueType::Obj(parent));

        // Try to get intermediate object
        let parent_obj = var_space.get("parent");
        assert!(parent_obj.is_some());
        if let Some(SecValueType::Obj(_)) = parent_obj {
            // This is expected
        } else {
            panic!("Expected Obj variant");
        }

        // Try to get non-existent child
        assert!(var_space.get("parent.invalid").is_none());
    }

    #[test]
    fn test_get_with_empty_path() {
        let mut var_space = VarSpace::default();
        var_space.global_mut().set("valid", "value");

        assert!(var_space.get("").is_none());
    }

    #[test]
    fn test_get_after_mutation() {
        let mut var_space = VarSpace::default();
        var_space.global_mut().set("counter", "1");

        // Initial value
        let initial = var_space.get("counter");
        assert!(initial.is_some());

        // Update value
        var_space.global_mut().set("counter", "2");

        // Get updated value
        let updated = var_space.get("counter");
        assert!(updated.is_some());
        if let Some(SecValueType::String(s)) = updated {
            assert_eq!(s.value(), "2");
        } else {
            panic!("Expected String variant");
        }
    }

    #[test]
    fn test_get_with_secret_values() {
        let mut var_space = VarSpace::default();

        // Create secret value
        let secret = SecString::sec_from("secret-value".to_string());

        var_space
            .global_mut()
            .set("api_key".to_string(), SecValueType::String(secret));

        let value = var_space.get("api_key");
        assert!(value.is_some());
        if let Some(SecValueType::String(s)) = value {
            assert!(s.is_secret());
            assert_eq!(s.value(), "secret-value");
        } else {
            panic!("Expected String variant");
        }
    }
}
