use once_cell::sync::OnceCell;
use orion_infra::logging::{configure_logging, LogConf};

pub trait DfxArgsGetter {
    fn debug_level(&self) -> usize;
    fn log_setting(&self) -> Option<String>;
}

pub fn configure_run_logging(_log_conf: Option<String>, debug: usize) {
    let setting = level_setting(debug);
    let conf = LogConf::new_console(setting);
    configure_logging(&conf).unwrap();
}

pub fn configure_dfx_logging(dfx: &impl DfxArgsGetter) {
    let setting = if let Some(log_setting) = dfx.log_setting() {
        log_setting
    } else {
        level_setting(dfx.debug_level()).to_string()
    };
    let conf = LogConf::new_console(&setting);
    configure_logging(&conf).unwrap();
}

fn level_setting(debug: usize) -> &'static str {
    if debug == 0 {
        return "error,exec=error,env=error,parse=error,sys=warn,stc=error";
    }
    if debug == 1 {
        return "error,exec=info";
    }
    if debug == 2 {
        return "warn,exec=info,load=info,assemble=info,parse=info,spec=info";
    }
    if debug == 3 {
        return "info,exec=debug,load=debug,assemble=debug,parse=debug,spec=debug";
    }
    if debug == 4 {
        return "debug";
    }
    if debug == 5 {
        return "debug,exec=trace,load=trace,assemble=trace,stc=trace";
    }
    if debug == 6 {
        return "trace";
    }
    "error"
}

#[allow(dead_code)]
pub fn init_env() {
    once_init_log();
}

struct TestIniter {}

pub fn once_init_log() {
    static INSTANCE: OnceCell<TestIniter> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        let conf = LogConf::new_console("debug");
        configure_logging(&conf).unwrap();
        TestIniter {}
    });
}
