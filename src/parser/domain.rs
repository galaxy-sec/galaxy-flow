use super::prelude::*;
use orion_parse::{
    atom::{skip_spaces_block, starts_with, take_var_name},
    define::{gal_raw_str, take_string},
    symbol::{
        symbol_brace_beg, symbol_brace_end, symbol_bracket_beg, symbol_bracket_end, symbol_colon,
        symbol_semicolon, wn_desc,
    },
};
use winnow::combinator::{delimited, repeat, separated};

use crate::{
    components::gxl_extend::{ModAddr, ModGitAddr, ModLocAddr, ModRef},
    primitive::{GxlArg, GxlObject},
};

use super::atom::{take_filename, take_filename_body, take_host, take_var_ref_name};

pub fn parse_log(pair: (&str, &str)) -> log::Level {
    if pair.0 == "log" {
        if let Ok(level_int) = pair.1.parse::<u32>() {
            match level_int {
                1 => {
                    return log::Level::Info;
                }
                2 => {
                    return log::Level::Debug;
                }
                3 => {
                    return log::Level::Trace;
                }
                _ => {
                    return log::Level::Info;
                }
            }
        }
    }
    log::Level::Info
}

pub fn ext_meta_names(input: &mut &str) -> Result<String> {
    let fst = take_var_name(input)?;
    Ok(fst)
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

//take :  key, or ${key}
pub fn gal_mix_item(input: &mut &str) -> Result<String> {
    alt((
        take_var_name,
        take_var_ref_name.map(|x| format!("${{{x}}}")),
    ))
    .parse_next(input)
}
pub fn gal_mix_in(input: &mut &str) -> Result<Vec<String>> {
    (multispace0, symbol_colon).parse_next(input)?;
    let found: Vec<String> = separated(0.., gal_mix_item, ",").parse_next(input)?;
    Ok(found)
}

pub fn take_version(input: &mut &str) -> Result<(i32, i32, i32, Option<i32>)> {
    let (a, _, b, _, c) = (digit1, ".", digit1, ".", digit1).parse_next(input)?;
    let build = opt((".", digit1)).parse_next(input)?;
    let a = a.parse::<i32>().unwrap();
    let b = b.parse::<i32>().unwrap();
    let c = c.parse::<i32>().unwrap();
    let d = build.map(|x| x.1.parse::<i32>().unwrap());
    Ok((a, b, c, d))
}

pub fn gal_var_input(input: &mut &str) -> Result<(String, String)> {
    let _ = multispace0.parse_next(input)?;
    let key_opt = opt(
        take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.']))
            .context(wn_desc("<var-name>")),
    )
    .parse_next(input)?;
    if key_opt.is_some() {
        symbol_colon.parse_next(input)?;
    }
    let _ = multispace0.parse_next(input)?;
    let val = alt((take_string, gal_raw_str))
        .context(wn_desc("<var-val>"))
        .parse_next(input)?;
    multispace0(input)?;
    let key = key_opt.unwrap_or("default");
    //(multispace0, alt((symbol_comma, symbol_semicolon))).parse_next(input)?;
    Ok((key.to_string(), val.to_string()))
}

pub fn fun_arg(input: &mut &str) -> Result<GxlArg> {
    let _ = multispace0.parse_next(input)?;
    let key_opt = opt(
        take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.']))
            .context(wn_desc("<var-name>")),
    )
    .parse_next(input)?;
    if key_opt.is_some() {
        symbol_colon.parse_next(input)?;
    }
    let _ = multispace0.parse_next(input)?;
    let val = alt((
        take_var_ref_name.map(GxlObject::VarRef),
        take_string.map(GxlObject::from_val),
        //gal_raw_string,
    ))
    .context(wn_desc("<var-val>"))
    .parse_next(input)?;
    multispace0(input)?;
    Ok(GxlArg::new(key_opt.unwrap_or("default"), val))
}

pub fn gal_call_beg(input: &mut &str) -> Result<()> {
    skip_spaces_block
        .context(wn_desc("<space-line>"))
        .parse_next(input)?;
    symbol_bracket_beg
        .context(wn_desc("<call-beg>"))
        .parse_next(input)?;
    skip_spaces_block
        .context(wn_desc("<space-line>"))
        .parse_next(input)?;
    Ok(())
}
pub fn gal_call_end(input: &mut &str) -> Result<()> {
    skip_spaces_block.parse_next(input)?;
    symbol_bracket_end
        .context(wn_desc("<call-end>"))
        .parse_next(input)?;
    skip_spaces_block.parse_next(input)?;
    opt(symbol_semicolon).parse_next(input)?;
    skip_spaces_block.parse_next(input)?;
    Ok(())
}

pub fn gal_sentence_beg(input: &mut &str) -> Result<()> {
    skip_spaces_block
        .context(wn_desc("<space-line>"))
        .parse_next(input)?;
    symbol_brace_beg.parse_next(input)?;
    skip_spaces_block
        .context(wn_desc("<space-line>"))
        .parse_next(input)?;
    Ok(())
}
pub fn gal_sentence_end(input: &mut &str) -> Result<()> {
    skip_spaces_block.parse_next(input)?;
    symbol_brace_end.parse_next(input)?;
    skip_spaces_block.parse_next(input)?;
    opt(symbol_semicolon).parse_next(input)?;
    Ok(())
}

pub fn gal_block_beg(input: &mut &str) -> Result<()> {
    skip_spaces_block
        .context(wn_desc("<space-line>"))
        .parse_next(input)?;
    symbol_brace_beg.parse_next(input)?;
    skip_spaces_block
        .context(wn_desc("<space-line>"))
        .parse_next(input)?;
    Ok(())
}

pub fn gal_block_end(input: &mut &str) -> Result<()> {
    skip_spaces_block.parse_next(input)?;
    symbol_brace_end.parse_next(input)?;
    skip_spaces_block.parse_next(input)?;
    Ok(())
}

pub fn gal_keyword(keyword: &'static str, input: &mut &str) -> Result<()> {
    (multispace0, keyword)
        .context(wn_desc(keyword))
        .parse_next(input)?;
    Ok(())
}

pub fn gal_keyword_alt(
    keyword: &'static str,
    keyword2: &'static str,
    input: &mut &str,
) -> Result<()> {
    (multispace0, alt((keyword, keyword2)), multispace0)
        .context(wn_desc(keyword))
        .parse_next(input)?;
    Ok(())
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

fn parse_git_addr(input: &mut &str) -> Result<ModAddr> {
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

    use crate::parser::{define::gal_var_assign, inner::run_gxl};

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
    fn test_mix() {
        let mut data = r#": dev,base"#;
        let vals = gal_mix_in(&mut data).assert();
        assert_eq!(vals, vec!["dev", "base"]);

        let mut data = r#": dev,${base}"#;
        let vals = gal_mix_in(&mut data).assert();
        assert_eq!(vals, vec!["dev", "${base}"]);
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
    #[test]
    fn test_ver() {
        let mut data = "1.0.0.123";
        let (a, b, c, d) = take_version(&mut data).assert();
        assert_eq!(a, 1);
        assert_eq!(b, 0);
        assert_eq!(c, 0);
        assert_eq!(d, Some(123));
    }
    #[test]
    fn test_assign() {
        let mut data =
            "data= r#\"{\"branchs\" : [{ \"name\": \"develop\" }, { \"name\" : \"release/1\"}]}\"#;";
        let (key, val) = run_gxl(gal_var_assign, &mut data).assert();
        assert_eq!(key, "data".to_string());
        assert_eq!(
            val,
            GxlObject::from_val(
                r#"{"branchs" : [{ "name": "develop" }, { "name" : "release/1"}]}"#.to_string()
            )
        );
    }
}
