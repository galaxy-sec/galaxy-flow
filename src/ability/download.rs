use crate::ability::prelude::*;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct GxDownLoad {
    value: String,
}
impl GxDownLoad {
    pub fn set(&mut self, val: &str) {
        self.value = val.to_string();
    }
}

// impl DefaultDTO for RgEcho {}

#[async_trait]
impl AsyncRunnableTrait for GxDownLoad {
    async fn async_exec(&self, ctx: ExecContext, def: &mut VarsDict) -> EOResult {
        //let local = LocalAddr::from("./test/data/sys-1");
        //local.update_rename(&path, "sys-2").await?;
        //local.update_local(&path).await?;

        let ex = EnvExpress::from_env_mix(def.clone());
        let out = ex.eval(&self.value)?;
        info!(target: ctx.path(), "{} :{}", &self.value, out);
        println!("{}", out);
        Ok(ExecOut::Ignore)
    }
}

impl ComponentRunnable for GxDownLoad {
    fn meta(&self) -> RgoMeta {
        RgoMeta::build_ability("gx.echo")
    }
}

#[cfg(test)]
mod tests {

    use crate::traits::Setter;

    use super::*;

    #[tokio::test]
    async fn echo_test() {
        let mut watcher = GxDownLoad::default();
        watcher.set("${HOME}");
        let ctx = ExecContext::default();
        let mut def = VarsDict::default();
        watcher.async_exec(ctx.clone(), &mut def).await.unwrap();
        def.set("HOME", "/root");
        watcher.async_exec(ctx.clone(), &mut def).await.unwrap();
    }
}
