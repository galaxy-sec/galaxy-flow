use galaxy_flow::cli;
use galaxy_flow::err::report_gxl_error;

#[tokio::main]
async fn main() {
    use std::process;

    if let Err(e) = cli::run_from_env().await {
        report_gxl_error(e);
        process::exit(-1);
    }
}
