use crate::ability::prelude::*;

#[derive(Clone, Default, Builder, Debug, PartialEq, Getters)]
pub struct GxAssert {
    value: String,
    expect: String,
    result: bool,
    error: Option<String>,
}
impl GxAssert {
    pub fn expect_eq(&mut self, val: &str, expect: &str) {
        self.value = val.to_string();
        self.expect = expect.to_string();
        self.result = true;
    }
    pub fn expect_no_eq(&mut self, val: &str, expect: &str) {
        self.value = val.to_string();
        self.expect = expect.to_string();
        self.result = false;
    }
    pub fn from_diy_error<S: Into<String>>(err: S) -> Self {
        Self {
            error: Some(err.into()),
            ..Default::default()
        }
    }
}

//impl DefaultDTO for RgAssert {}

impl RunnableTrait for GxAssert {
    fn exec(&self, mut ctx: ExecContext, def: &mut VarsDict) -> EOResult {
        ctx.append("assert");
        let exp = EnvExpress::from_env_mix(def.clone());
        let value = exp.eval(&self.value)?;
        let expect = exp.eval(&self.expect)?;
        debug!(target: ctx.path(), "value  {} :{}", &self.value, value);
        debug!(target: ctx.path(), "expect {} :{}", &self.expect, expect);

        if (value == expect) != self.result {
            let mut err_msg = format!(
                "assert fail! result: [{}],\n expect: [{}],\n value:  [{}]",
                self.result, expect, value
            );
            if let Some(msg) = self.error.clone() {
                err_msg = exp.eval(&msg)?;
            }
            println!("{}", err_msg);
            return Err(ExecError::from_domain(ExecReason::Check(format!(
                "assert fail! [{}], expect: {},\n value {}",
                self.result, expect, value
            ))));
        }
        info!(target: ctx.path(), "value {} match exprect", value);
        Ok(ExecOut::Ignore)
    }
}
impl ComponentRunnable for GxAssert {
    fn meta(&self) -> RgoMeta {
        RgoMeta::build_ability("gx.assert")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assert_test() {
        let mut assert = GxAssert::default();
        let ctx = ExecContext::default();
        let mut def = VarsDict::default();
        assert.expect_eq("hello", "hello");
        assert.exec(ctx.clone(), &mut def).unwrap();
        assert.expect_eq("${HOME}", "${HOME}");
        assert.exec(ctx.clone(), &mut def).unwrap();
        assert.expect_no_eq("${HOME}", "xxxx");
        assert.exec(ctx.clone(), &mut def).unwrap();
    }
}
