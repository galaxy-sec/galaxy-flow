use crate::ability::prelude::*;
use crate::components::RgVars;
use crate::expect::{LogicScope, ShellOption};

use orion_common::friendly::New2;

#[derive(Clone, Debug, PartialEq, Default, Builder)]
pub struct CmdDTO {
    pub name: String,
    pub cmd: String,
    pub expect: ShellOption,
}

impl CmdDTO {
    pub fn execute(&self, mut ctx: ExecContext, mut vars_dict: VarSpace) -> VTResult {
        ctx.append("gx.read_cmd");
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let cmd = self.cmd.clone();
        let name = self.name.clone();
        let cmd = exp.eval(&cmd)?;
        let (data, _) = gxl_sh!(LogicScope::Outer, ctx.path(), &cmd, &self.expect, &exp)?;
        let data_str = String::from_utf8(data)
            .map_err(|msg| ExecReason::Exp(format!("bad result {}", msg)))?;
        let mut vars = RgVars::default();
        vars.append(RgProp::new(name, data_str.trim().to_string()));
        vars.export_props(ctx, vars_dict.global_mut(), "")?;
        Ok((vars_dict, ExecOut::Ignore))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ability::{read::integra::ReadMode, *};

    #[tokio::test]
    async fn read_cmd_test() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/example/conf");
        let mut dto = CmdDTO::default();
        dto.name = format!("RG");
        dto.cmd = format!("echo galaxy-1.0");
        let res = GxRead::from(ReadMode::from(dto));
        res.async_exec(context, def).await.unwrap();
    }
}
