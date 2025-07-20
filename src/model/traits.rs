use std::sync::Arc;

use orion_error::UvsLogicFrom;

use crate::{
    evaluator::{EnvExpress, Parser},
    menu::GxMenu,
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
    fn get(&self, key: K) -> Option<&T>;
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
        let key_maker = UpperKeyMaker::new(prefix);
        debug!( target: ctx.path() ,"props export use prefix({prefix})" );
        let mut exp = EnvExpress::from_env_mix(dict.clone());
        for prop in self.fetch_props() {
            let key = key_maker.make(prop.key());
            match prop.val() {
                crate::primitive::GxlValue::VarRef(x) => {
                    if let Some(val) = dict.get(x.as_str()).cloned() {
                        dict.set(key.clone(), val.clone());
                        exp.insert_from(key, val);
                    }
                    return ExecError::from_logic(format!("nor var ref {x}")).err();
                }
                crate::primitive::GxlValue::Value(x) => {
                    match x {
                        crate::sec::SecValueType::String(v) => {
                            let val = exp.eval(v.value())?;
                            info!(target: ctx.path(),"{:10} = {}",key,val.cut_str(20));
                            dict.set(&key, val.clone());
                        }
                        _ => {
                            dict.set(&key, x.clone());
                        }
                    }
                    exp.insert_from(key, x.clone());
                }
            }
            //let val = exp.eval(prop.val())?;
        }
        Ok(())
    }
}
