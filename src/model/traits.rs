use std::sync::Arc;

use orion_error::UvsLogicFrom;

use crate::{
    evaluator::{EnvExpress, VarParser},
    menu::GxMenu,
    sec::{SecFrom, SecValueObj, SecValueType, ToUniCase},
    util::str_utils::{StringCutter, UpperKeyMaker},
    ExecError,
};

use super::{
    components::gxl_var::GxlVar, context::ExecContext, error::AResult,
    execution::sequence::ExecSequence, var::VarDict, ExecResult,
};

pub trait DependTrait<T>: Sized {
    fn assemble(self, mod_name: &str, src: T) -> AResult<Self>;
}
pub trait MergeTrait {
    fn merge(&mut self, other: &Self);
}

pub trait Setter<K, T> {
    fn set(&mut self, key: K, val: T);
}
pub trait Getter<K, T> {
    fn must_get(&self, key: K) -> &T;
    fn get_copy(&self, key: K) -> Option<T>;
}

pub trait ExecLoadTrait {
    fn load_env(&self, ctx: ExecContext, sequ: &mut ExecSequence, env: &str) -> ExecResult<()>;
    fn load_flow(&self, ctx: ExecContext, sequ: &mut ExecSequence, flow: &str) -> ExecResult<()>;
    fn menu(&self) -> ExecResult<GxMenu>;
    fn of_name(&self) -> String;
}

pub type AssembleHold = Arc<dyn ExecLoadTrait + 'static + Send + Sync>;

pub trait PropsTrait {
    fn fetch_props(&self) -> Vec<GxlVar>;
    fn export_props(&self, ctx: ExecContext, dict: &mut VarDict, prefix: &str) -> ExecResult<()> {
        let mut obj = SecValueObj::new();
        let key_maker = UpperKeyMaker::new(prefix);
        let mut exp = EnvExpress::from_env_mix(dict.clone());
        for prop in self.fetch_props() {
            let old_ver_key = key_maker.make(prop.key());
            match prop.val() {
                crate::primitive::GxlObject::VarRef(x) => {
                    if let Some(val) = dict.get_copy(x.as_str()) {
                        dict.set(old_ver_key.clone(), val.clone());
                        exp.insert_from(old_ver_key.clone(), val.clone());
                        exp.insert_from(prefix.to_string(), obj.clone());
                        info!(target: ctx.path(),"{old_ver_key:10} = {val}",);
                        obj.insert(prop.key().to_unicase(), val.clone());
                    } else {
                        return ExecError::from_logic(format!("nor var ref {x}")).err();
                    }
                }
                crate::primitive::GxlObject::Value(x) => {
                    match x {
                        crate::sec::SecValueType::String(v) => {
                            let val = exp.eval(v.value())?;
                            info!(target: ctx.path(),"{old_ver_key:10} = {}",val.cut_str(20));
                            dict.set(&old_ver_key, val.clone());
                            obj.insert(prop.key().to_unicase(), SecValueType::nor_from(val));
                        }
                        _ => {
                            info!(target: ctx.path(),"{old_ver_key:10} = {x}");
                            dict.set(&old_ver_key, x.clone());
                            obj.insert(prop.key().to_unicase(), x.clone());
                        }
                    }
                    exp.insert_from(prefix.to_string(), obj.clone());
                    exp.insert_from(old_ver_key, x.clone());
                }
            }
            //let val = exp.eval(prop.val())?;
        }
        dict.merge_item_obj(prefix, obj);
        Ok(())
    }
}
