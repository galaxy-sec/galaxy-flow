use crate::parser::atom::spaced_desc;
use crate::parser::atom::take_var_ref_fmt;
use crate::parser::prelude::*;

use orion_parse::atom::take_var_name;
use winnow::ascii::multispace0;
use winnow::combinator::preceded;
use winnow::combinator::separated;

#[derive(Default, Debug, Clone, Getters)]
pub struct FlowHeadDto {
    pub keyword: String,
    pub first: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
    pub args: Vec<(String, String)>,
}

pub fn galaxy_flow_head(input: &mut &str) -> ModalResult<FlowHeadDto> {
    // Parse the keyword (e.g., "flow")
    spaced_desc("flow", "<keyword:flow>").parse_next(input)?;
    let mut have_parse_syntax_new = true;
    // Parse the first part (e.g., "chatbot")

    let mut before: Vec<String> = separated(
        0..,
        alt((take_var_path, take_var_ref_fmt)),
        (multispace0, '|', multispace0),
    )
    .context(wn_desc("<pre-flow>"))
    .parse_next(input)?;
    multispace0(input)?;
    let (_, flow_name) = if starts_with("@", input) {
        spaced_desc(("@", take_var_name), "<flow-name>").parse_next(input)?
    } else if starts_with("|", input) {
        spaced_desc("|", "|").parse_next(input)?;
        spaced_desc(("@", take_var_name), "<flow-name>").parse_next(input)?
    } else if starts_with(":", input) {
        let name = if !before.is_empty() {
            before.remove(0)
        } else {
            String::new()
        };
        ("", name)
    } else {
        if !before.is_empty() {
            let flow_name = before.remove(0);
            return Ok(FlowHeadDto {
                keyword: "flow".to_string(),
                first: flow_name,
                before: Vec::new(),
                after: before,
                args: Vec::new(),
            });
        } else {
            fail.context(wn_desc("<flow-name>")).parse_next(input)?;
        }
        ("", String::new())
    };

    multispace0(input)?;
    let after: Vec<String> = if starts_with("|", input) {
        preceded(
            ('|', multispace0),
            // Parse the after flows (e.g., "a-flow1,a-flow2")
            separated(
                0..,
                //take_while(1.., |c: char| c.is_alphanumeric() || c == '_'),
                alt((take_var_path, take_var_ref_fmt)),
                (multispace0, '|', multispace0),
            ),
        )
        .context(wn_desc("<next-flow>"))
        .parse_next(input)?
    } else {
        Vec::new()
    };
    if before.is_empty() && after.is_empty() {
        have_parse_syntax_new = false;
    }
    if have_parse_syntax_new {
        return Ok(FlowHeadDto {
            keyword: "flow".to_string(),
            first: flow_name.to_string(),
            before,
            after,
            args: Vec::new(),
        });
    }

    //兼容 : <before>: <after>
    let before: Vec<String> = if starts_with(":", input) {
        // Parse the before flows (e.g., "b-flow1,b-flow2")
        preceded(
            (multispace0, ':', multispace0),
            separated(
                0..,
                alt((take_var_path, take_var_ref_fmt)),
                (multispace0, ',', multispace0),
            ),
            //(multispace0, alt((':', ';')), multispace0),
        )
        .context(wn_desc("<pre-flow>"))
        .parse_next(input)?
    } else {
        Vec::new()
    };

    multispace0(input)?;
    let after: Vec<String> = if starts_with(":", input) {
        preceded(
            (':', multispace0),
            // Parse the after flows (e.g., "a-flow1,a-flow2")
            separated(
                0..,
                //take_while(1.., |c: char| c.is_alphanumeric() || c == '_'),
                alt((take_var_path, take_var_ref_fmt)),
                (multispace0, ',', multispace0),
            ),
        )
        .context(wn_desc("<next-flow>"))
        .parse_next(input)?
    } else {
        Vec::new()
    };

    // Return the parsed flow head
    Ok(FlowHeadDto {
        keyword: "flow".to_string(),
        first: flow_name.to_string(),
        before,
        after,
        args: Vec::new(),
    })
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_take_flow_head_v1() {
        // Set up a parse context and relevant variables to pass in to the function
        let main_key: &str = "flow";
        let mut input: &str = r#"flow chatbot : b_flow1,b_flow2 : a_flow1,a_flow2"#;

        // Pass variables into the function and capture the results
        match galaxy_flow_head(&mut input) {
            Ok(dto) => {
                assert_eq!(dto.keyword, main_key);
                assert_eq!(dto.first, "chatbot");
                assert_eq!(dto.before, ["b_flow1", "b_flow2"]);
                assert_eq!(dto.after, ["a_flow1", "a_flow2"]);
            }
            Err(err) => {
                println!("Error: {:?}", err);
                assert!(false);
            }
        };

        // Expected results
    }

    #[test]
    fn test_take_flow_head_v2_0() {
        let main_key: &str = "flow";
        let mut input: &str = r#"flow b_flow1 | b_flow2 | @chatbot | a_flow1 | a_flow2"#;

        match galaxy_flow_head(&mut input) {
            Ok(dto) => {
                assert_eq!(dto.keyword, main_key);
                assert_eq!(dto.first, "chatbot");
                assert_eq!(dto.before, ["b_flow1", "b_flow2"]);
                assert_eq!(dto.after, ["a_flow1", "a_flow2"]);
            }
            Err(err) => {
                println!("Error: {:?}", err);
                assert!(false);
            }
        };

        // Expected results
    }

    #[test]
    fn test_take_flow_head_v2_1() {
        let main_key: &str = "flow";
        let mut input: &str = r#"flow chatbot |  a_flow1| a_flow2"#;

        match galaxy_flow_head(&mut input) {
            Ok(dto) => {
                assert_eq!(dto.keyword, main_key);
                assert_eq!(dto.first, "chatbot");
                assert!(dto.before.is_empty());
                assert_eq!(dto.after, ["a_flow1", "a_flow2"]);
            }
            Err(err) => {
                println!("Error: {:?}", err);
                assert!(false);
            }
        };
    }

    #[test]
    fn test_take_flow_head_v2_2() {
        let main_key: &str = "flow";
        let mut input: &str = r#"flow b_flow1| b_flow2 | @chatbot "#;

        match galaxy_flow_head(&mut input) {
            Ok(dto) => {
                assert_eq!(dto.keyword, main_key);
                assert_eq!(dto.first, "chatbot");
                assert_eq!(dto.before, ["b_flow1", "b_flow2"]);
                assert!(dto.after.is_empty());
            }
            Err(err) => {
                println!("Error: {:?}", err);
                assert!(false);
            }
        };
    }
}
