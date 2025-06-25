use std::io;

use crate::ability::prelude::*;
use crate::components::GxlVars;

use orion_common::friendly::New2;

#[derive(Clone, Debug, PartialEq, Default, Builder)]
pub struct StdinDTO {
    pub name: String,
    pub prompt: String,
}

impl StdinDTO {
    pub fn execute(&self, mut ctx: ExecContext, mut vars_dict: VarSpace) -> VTResult {
        ctx.append("gx.read_ini");
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let msg = self.prompt.clone();
        let name = self.name.clone();
        let msg = exp.eval(&msg)?;
        println!("{}", msg);
        let mut buffer = String::new();
        let stdin = io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut buffer).owe_data()?;
        let mut vars = GxlVars::default();
        vars.append(GxlProp::new(name, buffer.trim().to_string()));
        vars.export_props(ctx, vars_dict.global_mut(), "")?;
        Ok((vars_dict, ExecOut::Ignore))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ability::{read::integra::ReadMode, *};

    #[ignore]
    #[tokio::test]
    async fn read_stdin_test() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/example/conf");
        let mut dto = StdinDTO::default();
        dto.prompt = String::from("please input you name");
        dto.name = String::from("name");
        let res = GxRead::from(ReadMode::from(dto));
        res.async_exec(context, def).await.unwrap();
    }
}
