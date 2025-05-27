use clap::{Args, Subcommand};
use galaxy_flow::{infra::configure_flow_logging, types::AnyResult};
use log::info;
//use orion_vault::{utils::SecretSource, VaultBuilder, VaultKit};

pub fn vault_main(cmd: VaultCmd) -> AnyResult<()> {
    //log_init(&LogConf::alpha())?;
    match cmd {
        VaultCmd::Create(args) => {
            configure_flow_logging(args.log.clone(), 0);
            let factory = VaultBuilder::cust_init(args.root_path.as_str(), args.cust.as_str());
            factory.create_template()?;
            println!("create vault template success! in {}", factory.repo_path());
        }
        VaultCmd::Build(args) => {
            configure_flow_logging(args.log.clone(), 0);
            let factory = VaultBuilder::cust_init(args.root_path.as_str(), args.cust.as_str());
            factory.build()?;
            println!("build vault data success! in {}", factory.vault_path());
        }
        VaultCmd::ReBuild(args) => {
            configure_flow_logging(args.log.clone(), 0);
            let factory = VaultBuilder::cust_init(args.root_path.as_str(), args.cust.as_str());
            let sec_key =
                orion_vault::load_secret_key(&[SecretSource::file(factory.seckey_path())])?;
            factory.build_with_key(&sec_key)?;
            println!("rebuild vault data success! in {}", factory.vault_path());
        }
        VaultCmd::Export(args) => {
            configure_flow_logging(args.log.clone(), 0);
            if let Some(n_str) = &args.env_id {
                info!("env_nonce:{}", n_str);
            }
            let nonce = orion_vault::obtain_nonce(args.env_id);
            let sec_key = orion_vault::load_secret_key(&[
                SecretSource::env(args.env_sec_key),
                SecretSource::file(args.file_sec_key.as_str()),
            ])?;
            VaultKit::export(
                sec_key,
                args.dat_path.as_str(),
                args.dst_path.as_str(),
                nonce,
            )?;
            println!("export vault data success! to {}", args.dst_path.as_str());
        }
        VaultCmd::Rend(args) => {
            configure_flow_logging(args.log.clone(), 0);
            if let Some(n_str) = &args.env_id {
                info!("env_nonce:{}", n_str);
            }
            let nonce = orion_vault::obtain_nonce(args.env_id);
            let sec_key = orion_vault::load_secret_key(&[
                SecretSource::env(args.env_sec_key),
                SecretSource::file(args.file_sec_key.as_str()),
            ])?;
            VaultKit::render_tpl(
                sec_key,
                args.dat_path.as_str(),
                args.tpl_path.as_str(),
                args.out_path.as_str(),
                nonce,
            )?;
            println!("render template success! to {}", args.out_path.as_str());
        }
    }
    Ok(())
}

#[derive(Subcommand, Debug)]
#[command(name = "rgv")]
pub enum VaultCmd {
    /// create vault template in src<cust> folder
    #[command(name = "create")]
    Create(FactoryArgs),
    /// build vault data in dat<cust> folder
    Build(FactoryArgs),
    /// rebuild vault data in dat<cust> folder
    ReBuild(FactoryArgs),
    /// export vault data to dst folder
    Export(ExportArgs),
    /// render template to out folder
    Rend(RendArgs),
}

#[derive(Args, Debug, Default)]
pub struct FactoryArgs {
    #[clap(short, long)]
    pub cust: String,
    #[clap(long = "root", default_value = ".")]
    pub root_path: String,
    /// log config eg:  --log warn,parse=debug
    #[arg(long = "log")]
    pub log: Option<String>,
}

#[derive(Args, Debug, Default)]
pub struct VaultCmdArgs {
    #[clap(short, long, default_value = ".")]
    pub work_root: String,
    #[clap(short, long)]
    pub cust: String,
    #[clap(long = "src", default_value = "src")]
    pub src_path: String,
    #[clap(long = "dat", default_value = "dat")]
    pub dat_path: String,
    #[clap(long = "sec", default_value = "sec")]
    pub sec_path: String,
    #[clap(long = "dst", default_value = "dst")]
    pub dst_path: String,
}
#[derive(Args, Debug, Default)]
pub struct RendArgs {
    #[clap(short, long, default_value = ".")]
    pub work_root: String,
    #[clap(long = "dat", default_value = "./rgv.toml")]
    pub dat_path: String,
    /// get sec key from  file
    #[clap(long = "file_sec", default_value = "./sec.key")]
    pub file_sec_key: String,
    /// get sec key from env var
    #[clap(long = "env_sec")]
    pub env_sec_key: Option<String>,
    /// get sec id from env var
    #[clap(long = "env_id")]
    pub env_id: Option<String>,

    #[clap(short, long = "out")]
    pub out_path: String,
    /// tpl path
    #[clap(short, long = "tpl")]
    pub tpl_path: String,
    /// log config eg:  --log warn,parse=debug
    #[arg(long = "log")]
    pub log: Option<String>,
}

#[derive(Args, Debug, Default)]
pub struct ExportArgs {
    #[clap(short, long, default_value = ".")]
    pub work_root: String,
    #[clap(long = "dat", default_value = "./rgv.toml")]
    pub dat_path: String,
    /// get sec key from  file
    #[clap(long = "file_sec", default_value = "./sec.key")]
    pub file_sec_key: String,
    /// get sec key from env var
    #[clap(long = "env_sec")]
    pub env_sec_key: Option<String>,
    /// get sec  id from env var
    #[clap(long = "env_id")]
    pub env_id: Option<String>,
    /// dst path
    #[clap(long = "dst")]
    pub dst_path: String,
    /// log config eg:  --log warn,parse=debug
    #[arg(long = "log")]
    pub log: Option<String>,
}

#[derive(Subcommand, Debug)]
#[command(name = "sys")]
pub enum SysCmd {
    #[command(name = "info")]
    Info,
}
