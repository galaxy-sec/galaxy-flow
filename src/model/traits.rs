use std::sync::Arc;

use crate::{
    calculate::dynval::{EnvVarTag, VarCalcSupport},
    evaluator::{EnvExpress, Parser},
    menu::GxMenu,
    util::str_utils::{StringCutter, UpperKeyMaker},
};

use super::{
    components::gxl_var::RgProp, context::ExecContext, error::AResult,
    execution::sequence::Sequence, var::VarDict, ExecResult,
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
    fn load_env(&self, ctx: ExecContext, sequ: &mut Sequence, env: &str) -> ExecResult<()>;
    fn load_flow(&self, ctx: ExecContext, sequ: &mut Sequence, flow: &str) -> ExecResult<()>;
    fn menu(&self) -> ExecResult<GxMenu>;
    fn of_name(&self) -> String;
}

pub type AssembleHold = Arc<dyn ExecLoadTrait + 'static + Send + Sync>;

pub trait PropsTrait {
    fn fetch_props(&self) -> &Vec<RgProp>;
    fn export_props(&self, ctx: ExecContext, dict: &mut VarDict, prefix: &str) -> ExecResult<()> {
        EnvVarTag::import(&dict.export());
        let key_maker = UpperKeyMaker::new(prefix);
        debug!( target: ctx.path() ,"props export use prefix({})", prefix);
        let mut exp = EnvExpress::from_env_mix(dict.clone());
        for prop in self.fetch_props() {
            let key = key_maker.make(prop.key());
            let val = exp.eval(prop.val())?;
            info!(target: ctx.path(),"{:10} = {}",key,val.cut_str(20));
            dict.set(&key, val.clone());
            exp.insert(key, val);
        }
        Ok(())
    }
}
