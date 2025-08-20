use std::{fs::read_to_string, path::PathBuf};

use colored::Colorize;
use orion_ai::{AiClient, AiClientTrait, AiConfig, AiRoleID};
use orion_error::{ErrorConv, ErrorOwe};
use orion_variate::vars::EnvDict;

use crate::{err::RunResult, execution::VarSpace, util::redirect::init_redirect_file};

pub async fn ai_diagnose(var_space: &VarSpace) -> RunResult<()> {
    let output = init_redirect_file().unwrap();
    let ai_config = AiConfig::galaxy_load(&EnvDict::from(var_space)).err_conv()?;
    let ai_client = AiClient::new(ai_config, None).err_conv()?;
    let mut message = read_to_string(output.as_path()).owe_data()?;
    let gxl = read_to_string(PathBuf::from("./.run.gxl")).owe_data()?;
    message.push_str("=========== run gxl file ============ \n");
    message.push_str(gxl.as_str());
    println!("{}", "Send AI Anaylse ....".yellow());
    let ai_response = ai_client
        .smart_role_request(&AiRoleID::new("galactiward"), message.as_str())
        .await
        .err_conv()?;
    let response_content = ai_response.content;
    let response_provider = ai_response.provider.to_string();
    println!("{}", "AI Response:".yellow());
    println!("{}", format!("Content: {response_content}").yellow());
    println!("{}", format!("Model: {response_provider}").yellow());
    println!("{}", "".yellow());
    Ok(())
}
