use crate::ability::prelude::*;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct GxEcho {
    value: String,
}
impl GxEcho {
    pub fn set(&mut self, val: &str) {
        self.value = val.to_string();
    }
}

// impl DefaultDTO for RgEcho {}

impl RunnableTrait for GxEcho {
    fn exec(&self, ctx: ExecContext, def: &mut VarsDict) -> EOResult {
        let ex = EnvExpress::from_env_mix(def.clone());
        let out = ex.eval(&self.value)?;
        info!(target: ctx.path(), "{} :{}", &self.value, out);
        println!("{}", out);
        Ok(ExecOut::Ignore)
    }
}

impl ComponentRunnable for GxEcho {
    fn meta(&self) -> RgoMeta {
        RgoMeta::build_ability("gx.echo")
    }
}

#[cfg(test)]
mod tests {

    use crate::traits::Setter;

    use super::*;

    #[test]
    fn echo_test() {
        let mut watcher = GxEcho::default();
        watcher.set("${HOME}");
        let ctx = ExecContext::default();
        let mut def = VarsDict::default();
        watcher.exec(ctx.clone(), &mut def).unwrap();
        def.set("HOME", "/root");
        watcher.exec(ctx.clone(), &mut def).unwrap();
    }
}
