use super::prelude::*;
use std::collections::HashMap;
use std::fmt::Display;

use orion_parse::atom::take_var_name;
use winnow::ascii::multispace0;
use winnow::combinator::preceded;
use winnow::combinator::separated;

use crate::annotation::AnnTypeEnum;
use crate::annotation::FlowAnnFunc;
use crate::annotation::FlowAnnotation;

use winnow::combinator::delimited;
use winnow::token::take_while;

use crate::model::annotation::{AnnEnum, EnvAnnFunc, EnvAnnotation, ModAnnFunc, ModAnnotation};

use super::atom::spaced_desc;
use super::atom::take_var_ref_fmt;

#[derive(Default, Debug, Clone)]
pub struct STCHeadDto {
    pub keyword: String,
    pub name: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
    pub args: Vec<(String, String)>,
}

#[derive(Default, Debug, Clone, Getters)]
pub struct EnvHeadDto {
    name: String,
    mix: Vec<String>,
}
impl EnvHeadDto {
    pub fn new(name: String, mix: Vec<String>) -> Self {
        Self { name, mix }
    }
}
#[derive(Default, Debug, Clone, Getters)]
pub struct ModDto {
    name: String,
    mix: Vec<String>,
}
impl ModDto {
    pub fn new(name: String, mix: Vec<String>) -> Self {
        Self { name, mix }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct FunDto {
    pub keyword: String,
    pub args: HashMap<String, String>,
}
impl Display for FunDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ann-fun : {}(", self.keyword)?;
        for (k, v) in &self.args {
            write!(f, "{}:{},", k, v)?;
        }
        write!(f, ")",)?;
        Ok(())
    }
}

impl FunDto {
    pub fn new(keyword: &str, args: Vec<(&str, &str)>) -> Self {
        Self {
            keyword: keyword.to_string(),
            args: args
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}
impl From<FunDto> for FlowAnnotation {
    fn from(dto: FunDto) -> FlowAnnotation {
        let name = FlowAnnFunc::from(dto.keyword.as_str());
        FlowAnnotation {
            name: dto.keyword.clone(),
            ann_type: AnnTypeEnum::Func,
            func: name,
            args: dto.args,
        }
    }
}
impl From<FunDto> for ModAnnotation {
    fn from(dto: FunDto) -> ModAnnotation {
        let name = ModAnnFunc::from(dto.keyword.as_str());
        ModAnnotation {
            name: dto.keyword.clone(),
            ann_type: AnnTypeEnum::Func,
            func: name,
            args: dto.args,
        }
    }
}

impl From<FunDto> for EnvAnnotation {
    fn from(dto: FunDto) -> EnvAnnotation {
        let name = EnvAnnFunc::from(dto.keyword.as_str());
        EnvAnnotation {
            name: dto.keyword.clone(),
            ann_type: AnnTypeEnum::Func,
            func: name,
            args: dto.args,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AnnDto {
    pub funs: Vec<FunDto>,
}
impl AnnDto {
    pub fn convert<T>(&self) -> Vec<AnnEnum>
    where
        T: From<FunDto>,
        AnnEnum: From<T>,
    {
        self.funs
            .iter()
            .map(|f| AnnEnum::from(T::from(f.clone())))
            .collect()
    }
}
#[derive(Default, Debug, Clone)]
pub struct HeadDto {
    pub keyword: String,
    pub name: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
}

/// Parse a key-value assignment (e.g., `x = 1`, `x= "A"`, `1`, `"A"`).
pub fn gal_fun_arg_item(input: &mut &str) -> ModalResult<(Option<String>, String)> {
    // Parse the key (optional)
    multispace0.parse_next(input)?;
    let key = opt((
        take_var_name,                   // Parse the key (alphanumeric)
        (multispace0, "=", multispace0), // Skip optional whitespace and equals sign
    ))
    .parse_next(input)?;

    // Parse the value (either a number or a quoted string)
    let value = if input.starts_with('"') {
        // Parse a quoted string
        delimited('"', take_while(0.., |c| c != '"'), '"')
            .context(wn_desc("\"<arg>\""))
            .parse_next(input)?
    } else {
        // Parse a number or unquoted string
        take_while(1.., |c: char| c.is_alphanumeric() || c == '.' || c == '-')
            .context(wn_desc("<arg>"))
            .parse_next(input)?
    };

    // Extract the key if it exists
    let key = key.map(|(k, _)| k.to_string());
    Ok((key, value.to_string()))
}

/// Parse function arguments from a string and return a HashMap.
/// If a key is missing, it is replaced with a placeholder like `_1`, `_2`, etc.
pub fn gal_fun_args_map(input: &mut &str) -> ModalResult<HashMap<String, String>> {
    // Parse the arguments list enclosed in parentheses
    let args: Vec<(Option<String>, String)> = delimited(
        ('(', multispace0), // Start with '('
        separated(
            0..,
            gal_fun_arg_item,                // Parse each key-value pair
            (multispace0, ',', multispace0), // Separated by commas and optional whitespace
        ),
        (multispace0, ')'), // End with ')'
    )
    .context(wn_desc("<fun-args>"))
    .parse_next(input)?;

    // Convert Vec<(Option<String>, String)> to HashMap<String, String>
    let args_map = args
        .into_iter()
        .enumerate()
        .map(|(index, (key, value))| {
            let key = key.unwrap_or_else(|| format!("_{}", index + 1)); // Use _1, _2, etc. for missing keys
            (key, value)
        })
        .collect();

    Ok(args_map)
}

/// Parse a function call into a `FunDto`.
pub fn gal_fun(input: &mut &str) -> ModalResult<FunDto> {
    // Parse the function name
    let keyword = take_var_name.parse_next(input)?;
    //let keyword = ident1.parse_next(input)?;

    //let now: &str = input;
    let args = if starts_with("(", input) {
        // Parse the arguments
        gal_fun_args_map(input)?
    } else {
        HashMap::new()
    };

    Ok(FunDto {
        keyword: keyword.to_string(),
        args,
    })
}

pub fn gal_fun_vec(input: &mut &str) -> ModalResult<Vec<FunDto>> {
    separated(0.., gal_fun, (multispace0, ",", multispace0)).parse_next(input)
}

/// Parse an annotation into an `AnnDto`.
pub fn gal_ann(input: &mut &str) -> ModalResult<AnnDto> {
    // Parse the annotation prefix `#[`
    let _ = (multispace0, "#", "[").parse_next(input)?;

    // Parse the list of function calls
    let funs = gal_fun_vec.parse_next(input)?;

    // Parse the closing `]`
    let _ = "]".parse_next(input)?;

    // Return the parsed annotation
    funs.iter().for_each(|x| debug!(target: "parse","{}", x));
    Ok(AnnDto { funs })
}

pub fn gal_env_head(input: &mut &str) -> ModalResult<EnvHeadDto> {
    // Parse the keyword (e.g., "flow")
    spaced_desc("env", "<keyword:env>").parse_next(input)?;
    let first = take_var_name.parse_next(input)?;
    multispace0.parse_next(input)?;

    // Parse the before flows (e.g., "b-flow1,b-flow2")
    if starts_with(":", input) {
        let mix: Vec<String> = preceded(
            (multispace0, ':', multispace0),
            separated(
                0..,
                alt((take_var_path, take_var_ref_fmt)),
                (multispace0, ',', multispace0),
            ),
        )
        .parse_next(input)?;
        Ok(EnvHeadDto::new(first, mix))
    } else {
        Ok(EnvHeadDto::new(first, Vec::new()))
    }
}

pub fn gal_mod_head(input: &mut &str) -> ModalResult<ModDto> {
    // Parse the keyword (e.g., "flow")
    spaced_desc("mod", "<keyword:mod>").parse_next(input)?;
    let first = take_var_name(input)?;
    multispace0.parse_next(input)?;

    if starts_with(":", input) {
        let mix: Vec<String> = preceded(
            (multispace0, ':', multispace0),
            separated(0.., take_var_path, (multispace0, ',', multispace0)),
        )
        .parse_next(input)?;
        Ok(ModDto::new(first, mix))
    } else {
        Ok(ModDto::new(first, Vec::new()))
    }
}

pub fn gal_act_head(act: &'static str, input: &mut &str) -> ModalResult<String> {
    spaced_desc(act, act).parse_next(input)?;

    let key = take_var_name(input)?;
    multispace0.parse_next(input)?;
    Ok(key)
}

#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use crate::{
        annotation::{FST_ARG_TAG, SEC_ARG_TAG},
        parser::inner::run_gxl,
        str_map,
    };

    use super::*;

    #[test]
    fn test_take_evn_head() {
        // Set up a parse context and relevant variables to pass in to the function
        let main_key: &str = "dev";
        let mut input: &str = r#"env dev: env1,env2"#;

        // Pass variables into the function and capture the results
        let dto = run_gxl(gal_env_head, &mut input).assert();
        assert_eq!(dto.name(), main_key);
        assert_eq!(
            dto.mix(),
            &["env1".to_string(), "env2".to_string()].to_vec()
        );
        assert_eq!(input, "");

        let mut input: &str = r#"env dev: env1,env2;"#;

        // Pass variables into the function and capture the results
        let dto = run_gxl(gal_env_head, &mut input).assert();
        assert_eq!(dto.name(), main_key);
        assert_eq!(
            dto.mix(),
            &["env1".to_string(), "env2".to_string()].to_vec()
        );
        assert_eq!(input, ";");
        // Expected results
    }
    #[test]
    fn test_gal_fun_set() {
        let mut input = r#"fn_x(x="1",y="2")"#;
        let expected = FunDto {
            keyword: String::from("fn_x"),
            args: str_map!(
                String::from("x") => String::from("1"),
                String::from("y") => String::from("2"),
            ),
        };
        let output = run_gxl(gal_fun, &mut input).unwrap();

        assert_eq!(input, "");
        assert_eq!(output, expected);

        let mut input = r#"fn_x()"#;
        let expected = FunDto {
            keyword: String::from("fn_x"),
            args: str_map!(),
        };
        let output = gal_fun(&mut input).unwrap();

        assert_eq!(output, expected);
    }

    #[test]
    fn test_gal_fun_args_with_args() {
        let mut input = r#"(x="1",y="2")"#;
        let expected = str_map!(
            String::from("x") => String::from("1"),
            String::from("y") => String::from("2"),
        );
        let output = gal_fun_args_map(&mut input).unwrap();

        assert_eq!(input, "");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gal_fun_args_no_args() {
        let mut input = r#"()"#;
        let expected = str_map!();
        let output = gal_fun_args_map(&mut input).unwrap();

        assert_eq!(input, "");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gal_fun_args_invalid_format() {
        let mut input = r#"(x=1,y=2"#; // Missing closing parenthesis
        let result = gal_fun_args_map(&mut input);

        assert!(result.is_err());
    }

    #[test]
    fn test_gal_fun_args_whitespace() {
        let mut input = r#"( x = "1" , y = "2" )"#;
        let expected = str_map!(
            String::from("x") => String::from("1"),
            String::from("y") => String::from("2"),
        );
        let output = run_gxl(gal_fun_args_map, &mut input).unwrap();

        assert_eq!(input, "");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gal_fun_args_simple() {
        let mut input = r#"( "1" , "2" )"#;
        let expected = str_map!(
            String::from(FST_ARG_TAG) => String::from("1"),
            String::from(SEC_ARG_TAG) => String::from("2"),
        );
        let output = run_gxl(gal_fun_args_map, &mut input).unwrap();

        assert_eq!(input, "");
        assert_eq!(output, expected);
    }

    #[test]
    fn test_ext_extern_ann() {
        let mut input = r#"#[load(a,b)]"#;
        let expected = AnnDto {
            funs: [FunDto::new(
                "load",
                [(FST_ARG_TAG, "a"), (SEC_ARG_TAG, "b")].to_vec(),
            )]
            .to_vec(),
        };
        assert_ext_ann(&mut input, expected);

        let mut input = r#"#[load(a,b),unload()]"#;
        let expected = AnnDto {
            funs: [
                FunDto::new("load", [(FST_ARG_TAG, "a"), (SEC_ARG_TAG, "b")].to_vec()),
                FunDto::new("unload", [].to_vec()),
            ]
            .to_vec(),
        };
        assert_ext_ann(&mut input, expected);
    }

    fn assert_ext_ann(input: &mut &str, expected: AnnDto) {
        let output = gal_ann(input).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_gal_multi_fun() {
        let mut input = r#"load(a,b)"#;
        let expected = [FunDto::new(
            "load",
            [(FST_ARG_TAG, "a"), (SEC_ARG_TAG, "b")].to_vec(),
        )]
        .to_vec();
        assert_funs(&mut input, expected);
        let mut input = r#"load(a,b),unload(a,b)"#;
        let expected = [
            FunDto::new("load", [(FST_ARG_TAG, "a"), (SEC_ARG_TAG, "b")].to_vec()),
            FunDto::new("unload", [(FST_ARG_TAG, "a"), (SEC_ARG_TAG, "b")].to_vec()),
        ]
        .to_vec();
        assert_funs(&mut input, expected);
        let mut input = r#"load(a="1",b="2"),unload(a="3",b)"#;
        let expected = [
            FunDto::new("load", [("a", "1"), ("b", "2")].to_vec()),
            FunDto::new("unload", [("a", "3"), ("_2", "b")].to_vec()),
        ]
        .to_vec();
        assert_funs(&mut input, expected);
        let mut input = r#"load(a="1",b="2"),unload"#;
        let expected = [
            FunDto::new("load", [("a", "1"), ("b", "2")].to_vec()),
            FunDto::new("unload", [].to_vec()),
        ]
        .to_vec();
        assert_funs(&mut input, expected);
    }

    fn assert_funs(input: &mut &str, expected: Vec<FunDto>) {
        let output = gal_fun_vec(input).unwrap();
        assert_eq!(output, expected);
    }

    mod test_fun {
        use crate::parser::stc_base::gal_fun_arg_item;

        #[test]
        fn test_gal_fun_arg_item_with_key() {
            let mut input = r#"x = 1"#;
            let expected = (Some("x".to_string()), "1".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);

            let mut input = r#"x="A""#;
            let expected = (Some("x".to_string()), "A".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);
        }

        #[test]
        fn test_gal_fun_arg_item_without_key() {
            let mut input = r#"1"#;
            let expected = (None, "1".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);

            let mut input = r#""A""#;
            let expected = (None, "A".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);
        }

        #[test]
        fn test_gal_fun_arg_item_whitespace() {
            let mut input = r#"  x   =   123"#;
            let expected = (Some("x".to_string()), "123".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);

            let mut input = r#"  "A""#;
            let expected = (None, "A".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);
        }

        #[test]
        fn test_gal_fun_arg_item_invalid_format() {
            let mut input = r#"x="#; // Missing value
            let result = gal_fun_arg_item(&mut input);

            assert!(result.is_err());

            let mut input = r#""#; // Empty input
            let result = gal_fun_arg_item(&mut input);

            assert!(result.is_err());
        }

        #[test]
        fn test_gal_fun_arg_item_unquoted_string() {
            let mut input = r#"x=hello"#;
            let expected = (Some("x".to_string()), "hello".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);

            let mut input = r#"hello"#;
            let expected = (None, "hello".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);
        }

        #[test]
        fn test_gal_fun_arg_item_negative_number() {
            let mut input = r#"x=-123"#;
            let expected = (Some("x".to_string()), "-123".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);

            let mut input = r#"-123"#;
            let expected = (None, "-123".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);
        }

        #[test]
        fn test_gal_fun_arg_item_decimal_number() {
            let mut input = r#"x=3.14"#;
            let expected = (Some("x".to_string()), "3.14".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);

            let mut input = r#"3.14"#;
            let expected = (None, "3.14".to_string());
            let output = gal_fun_arg_item(&mut input).unwrap();

            assert_eq!(input, "");
            assert_eq!(output, expected);
        }
    }
}
