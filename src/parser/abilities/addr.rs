use orion_parse::atom::take_var_name;
use winnow::combinator::{delimited, separated};

use crate::{components::gxl_extend::{ModAddr, ModGitAddr, ModLocAddr, ModRef}, parser::atom::{take_filename, take_filename_body, take_host}};

use super::prelude::*;

pub fn parse_git_addr(input: &mut &str) -> Result<ModAddr> {
    ("{", multispace0, "git", multispace0, "=", multispace0)
        .context(wn_desc("{ git = "))
        .parse_next(input)?;
    let remote = delimited("\"", take_while(1.., |c: char| c != '"'), "\"")
        .context(wn_desc("git path"))
        .parse_next(input)?;
    (
        multispace0,
        ";",
        multispace0,
        //"channel",
        alt(("channel", "tag", "branch")),
        multispace0,
        "=",
        multispace0,
    )
        .context(wn_desc(", channel = "))
        .parse_next(input)?;

    let channel = delimited("\"", take_while(1.., |c: char| c != '"'), "\"").parse_next(input)?;
    (multispace0, ";", multispace0, "}").parse_next(input)?;
    Ok(ModAddr::Git(ModGitAddr::new(remote, channel)))
}

// 解析本地路径模式
fn parse_local_addr(input: &mut &str) -> Result<ModAddr> {
    ("{", multispace0, "path", multispace0, "=", multispace0).parse_next(input)?;

    let path = delimited("\"", take_while(1.., |c: char| c != '"'), "\"").parse_next(input)?;
    (multispace0, ";", multispace0, "}").parse_next(input)?;
    Ok(ModAddr::Loc(ModLocAddr::new(path)))
}

pub fn gal_git_path(input: &mut &str) -> Result<(String, String)> {
    let _ = alt(("git@", "https://")).parse_next(input)?;
    let host = take_host.parse_next(input)?;
    let _ = alt((":", "/")).parse_next(input)?;
    let path = (take_filename, "/");
    let _: Vec<(String, &str)> = repeat(0.., path).parse_next(input)?;
    let name = take_filename_body.parse_next(input)?;
    let _ = opt(".git").parse_next(input)?;
    Ok((host, name))
}
pub fn parse_mod_addr(input: &mut &str) -> Result<ModAddr> {
    if starts_with(
        ("{", multispace0, "git", multispace0, "=", multispace0),
        input,
    ) {
        parse_git_addr(input)
    } else {
        parse_local_addr(input)
    }
}


// 解析 extern mod
pub fn gal_extern_mod(input: &mut &str) -> Result<ModRef> {
    // 解析 "extern mod"
    let _ = (multispace0, "extern", multispace0, "mod", multispace0)
        .context(wn_desc("extern mod "))
        .parse_next(input)?;

    // 解析模块名列表
    let mods = separated(1.., take_var_name, ",")
        .context(wn_desc("mod names"))
        .parse_next(input)?;

    // 解析地址部分
    let addr = delimited(multispace0, parse_mod_addr, multispace0)
        .context(wn_desc("<mod-addr>"))
        .parse_next(input)?;

    Ok(ModRef::new(mods, addr))
}



#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use crate::{parser::{ abilities::define::gal_var_assign, domain::take_version, inner::run_gxl}, primitive::GxlObject};

    use super::*;

    #[test]
    fn test_parse_git_addr() {
        // 测试 Git 仓库模式
        let mut input = r#"{ git = "git@galaxy-sec.org:/gxl-lab.git"; channel = "0.1.0"; }"#;
        let result = parse_git_addr(&mut input).unwrap();
        match result {
            ModAddr::Git(addr) => {
                assert_eq!(addr.remote(), "git@galaxy-sec.org:/gxl-lab.git");
                assert_eq!(addr.channel(), "0.1.0");
            }
            _ => panic!("Expected Git address"),
        }
    }

    #[test]
    fn test_parse_git_addr_invalid() {
        // 测试无效的 Git 仓库模式（缺少 channel）
        let mut input = r#"{ git = "git@galaxy-sec.org:free/gxl-lab.git"; }"#;
        assert!(parse_git_addr(&mut input).is_err());

        // 测试无效的 Git 仓库模式（缺少 git）
        let mut input = r#"{ channel = "0.1.0"; }"#;
        assert!(parse_git_addr(&mut input).is_err());
    }

    #[test]
    fn test_parse_local_addr() {
        // 测试本地路径模式
        let mut input = r#"{ path = "./extern/gxl-lab-0.2.8/mods"; }"#;
        let result = parse_local_addr(&mut input).unwrap();
        match result {
            ModAddr::Loc(addr) => {
                assert_eq!(addr.path(), "./extern/gxl-lab-0.2.8/mods");
            }
            _ => panic!("Expected local path"),
        }
    }

    #[test]
    fn test_parse_local_addr_invalid() {
        // 测试无效的本地路径模式（缺少 path）
        let mut input = r#"{ invalid = "./extern/gxl-lab-0.2.8/mods"; }"#;
        assert!(parse_local_addr(&mut input).is_err());

        // 测试无效的本地路径模式（缺少路径值）
        let mut input = r#"{ path = ; }"#;
        assert!(parse_local_addr(&mut input).is_err());
    }
    #[test]
    fn test_gal_extern_mod_git() {
        let mut input =
            r#"extern mod os { git = "git@galaxy-sec.org:free/gxl-lab.git"; channel = "0.1.0"; }"#;
        let result = gal_extern_mod(&mut input).unwrap();
        match result.addr() {
            ModAddr::Git(addr) => {
                assert_eq!(addr.remote(), "git@galaxy-sec.org:free/gxl-lab.git");
                assert_eq!(addr.channel(), "0.1.0");
            }
            _ => panic!("Expected Git address"),
        }
        assert_eq!(result.mods(), &vec!["os"]);
    }

    #[test]
    fn test_gal_extern_mod_loc() {
        let mut input = r#"extern mod os,ssh,af_biz { path = "./extern/gxl-lab-0.2.8/mods"; }"#;
        let result = gal_extern_mod(&mut input).unwrap();
        match result.addr() {
            ModAddr::Loc(addr) => {
                assert_eq!(addr.path(), "./extern/gxl-lab-0.2.8/mods");
            }
            _ => panic!("Expected local path"),
        }
        assert_eq!(result.mods(), &vec!["os", "ssh", "af_biz"]);
    }

    #[test]
    fn test_gal_extern_mod_invalid() {
        let mut input = r#"extern mod os { invalid = "value"; }"#;
        assert!(gal_extern_mod(&mut input).is_err());
    }


    #[test]
    fn test_git() {
        let mut data = "git@galaxy-sec.org:free/gxl-lab.git";
        let (host, name) = gal_git_path(&mut data).assert();
        assert_eq!(host, "galaxy-sec.org");
        assert_eq!(name, "gxl-lab");

        let mut data = "https://galaxy-sec.org/free/gxl-lab.git";
        let (host, name) = gal_git_path(&mut data).assert();
        assert_eq!(host, "galaxy-sec.org");
        assert_eq!(name, "gxl-lab");

        let mut data = "git@galaxy-sec.org:free/x/gxl-lab";
        let (host, name) = gal_git_path(&mut data).assert();
        assert_eq!(host, "galaxy-sec.org");
        assert_eq!(name, "gxl-lab");

        //https://galaxy-sec.org/free/gxl-lab.git
    }


}
