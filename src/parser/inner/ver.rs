use super::super::prelude::*;
use super::common::sentence_call_args;
use winnow::combinator::fail;

use crate::ability::echo::*;
use crate::ability::version::*;
use crate::parser::domain::gal_keyword_alt;

pub fn gal_echo(input: &mut &str) -> ModalResult<GxEcho> {
    let mut watcher = GxEcho::default();
    gal_keyword_alt("gx.echo", "rg.echo", input)?;
    let props = sentence_call_args.parse_next(input)?;
    for (k, v) in props {
        if k == "value" {
            watcher.set(v.as_str());
        }
    }
    Ok(watcher)
}

pub fn gal_version(input: &mut &str) -> ModalResult<RgVersion> {
    let mut builder = RgVersionBuilder::default();
    builder.verinc(VerInc::Build);
    builder.export("VERSION".into());
    gal_keyword_alt("gx.ver", "rg.ver", input)?;
    let props = sentence_call_args.parse_next(input)?;
    for (key, val) in props {
        if key == "file" {
            builder.file(val);
            continue;
        }
        if key == "export" {
            builder.file(val);
            continue;
        }
        if key == "inc" {
            debug!("version inc :{}", val);
            if val == "build" {
                builder.verinc(VerInc::Build);
            }
            if val == "bugfix" {
                builder.verinc(VerInc::Bugfix);
            }
            if val == "feature" {
                builder.verinc(VerInc::Feature);
            }
            if val == "main" {
                builder.verinc(VerInc::Main);
            }
            if val == "null" {
                builder.verinc(VerInc::Null);
            }
        }
    }
    if let Ok(ver) = builder.build() {
        Ok(ver)
    } else {
        fail.parse_next(input)
    }
}

#[cfg(test)]
mod tests {

    use crate::parser::inner::common::run_gxl;

    use super::*;

    #[test]
    fn echo_test() -> ModalResult<()> {
        let mut data = r#"
             gx.echo ( value : "${PRJ_ROOT}/test/main.py" ) ;"#;
        let found = run_gxl(gal_echo, &mut data)?;
        let mut expect = GxEcho::default();
        expect.set(r#"${PRJ_ROOT}/test/main.py"#);
        assert_eq!(found, expect);
        assert_eq!(data, "");
        Ok(())
    }
    #[test]
    fn ver_test() {
        let mut data = r#"
             gx.ver  ( file : "./tests/version.txt",  inc : "build"  ) ;"#;
        let found = gal_version(&mut data).unwrap();
        let expect = RgVersion::new(format!("./tests/version.txt"));
        assert_eq!(found, expect);
        assert_eq!(data, "");
    }
}
