use crate::{err::RunResult, execution::VarSpace};

pub async fn ai_diagnose(_var_space: &VarSpace) -> RunResult<()> {
    // orion-ai temporarily removed, ai_diagnose is a no-op
    println!("AI diagnose is currently disabled");
    Ok(())
}
