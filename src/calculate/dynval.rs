use crate::evaluator::EnvExpress;
use crate::execution::VarSpace;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

pub trait VDSupport {
    fn name() -> &'static str;
}
#[derive(Clone)]
pub struct VarDef<T, E> {
    //assigns: VarsHold,
    _tag: PhantomData<E>,
    _val: PhantomData<T>,
    var: String,
}
impl<T, E> VarDef<T, E> {
    pub fn new(var: String) -> Self {
        Self {
            _tag: PhantomData,
            _val: PhantomData,
            var,
        }
    }
}

impl<T, E> Debug for VarDef<T, E>
where
    T: VDSupport,
    E: VDSupport,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VarDef")
            .field("name", &self.var)
            .field("type", &T::name())
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MocVarTag {}
lazy_static! {
    static ref ASSIGN_VARS: Arc<Mutex<HashMap<String, String>>> = {
        let map = HashMap::new();
        Arc::new(Mutex::new(map))
    };
}
impl VDSupport for MocVarTag {
    fn name() -> &'static str {
        "MocVarTag"
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct EnvVarTag {}
impl VDSupport for EnvVarTag {
    fn name() -> &'static str {
        "EnvVarTag"
    }
}
impl VDSupport for u32 {
    fn name() -> &'static str {
        "u32"
    }
}
impl VDSupport for String {
    fn name() -> &'static str {
        "String"
    }
}
pub enum VarDefEnum {
    Env,
    Moc,
}
pub type EVu32 = VarDef<u32, EnvVarTag>;
impl<T, E> From<String> for VarDef<T, E> {
    fn from(s: String) -> Self {
        Self {
            _tag: PhantomData,
            _val: PhantomData,
            var: s,
        }
    }
}
impl<T, E> From<&str> for VarDef<T, E> {
    fn from(s: &str) -> Self {
        Self {
            _tag: PhantomData,
            _val: PhantomData,
            var: s.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum EvalError {
    #[error("parse error")]
    ParseError,
    #[error("value error {0}")]
    ValueError(String),
    #[error("var miss '{0}'")]
    VarMiss(String),
    #[error("unthinking")]
    Unthinking,
}
pub type EvalResult<T> = Result<T, EvalError>;

#[automock]
pub trait ValueEval<R> {
    fn eval(&self, vars: &VarSpace) -> EvalResult<R>;
}

impl ValueEval<u32> for VarDef<u32, EnvVarTag> {
    fn eval(&self, vars: &VarSpace) -> EvalResult<u32> {
        let exp = EnvExpress::from_env_mix(vars.global().clone());
        let one = exp
            .eval_val(&self.var.to_uppercase())
            .ok_or(EvalError::VarMiss(self.var.clone()))?;
        Ok(one.parse::<u32>().unwrap())
    }
}
impl ValueEval<String> for VarDef<String, EnvVarTag> {
    fn eval(&self, vars: &VarSpace) -> EvalResult<String> {
        //let exp = EnvExpress::from_env_mix(VarDict::from(EnvVarTag::export()));
        let exp = EnvExpress::from_env_mix(vars.global().clone());
        let one = exp
            .eval_val(&self.var.to_uppercase())
            .ok_or(EvalError::VarMiss(self.var.clone()))?;
        Ok(one.clone())
    }
}

pub type MocU32 = VarDef<u32, MocVarTag>;
impl ValueEval<u32> for VarDef<u32, MocVarTag> {
    fn eval(&self, _vars: &VarSpace) -> EvalResult<u32> {
        let v = self.var.strip_prefix("moc_").ok_or(EvalError::Unthinking)?;
        v.parse::<u32>().map_err(|_| EvalError::ParseError)
    }
}
impl ValueEval<String> for VarDef<String, MocVarTag> {
    fn eval(&self, _vars: &VarSpace) -> EvalResult<String> {
        let v = self.var.strip_prefix("moc_").ok_or(EvalError::Unthinking)?;
        Ok(v.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug, Clone, PartialEq)]
    struct MocT(u32);
    type VarMoc2 = VarDef<MocT, MocVarTag>;
    impl ValueEval<MocT> for VarMoc2 {
        fn eval(&self, _vars: &VarSpace) -> EvalResult<MocT> {
            Ok(MocT(2))
        }
    }
    //test DynVal From
    #[test]
    fn test_var_from() {
        let val = VarMoc2::from("x");
        assert_eq!(val.eval(&VarSpace::default()), Ok(MocT(2)));
    }
}
