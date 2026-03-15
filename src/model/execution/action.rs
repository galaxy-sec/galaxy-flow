use crate::util::serialize_time_format::serialize_time_format;
use serde::Serialize;
use time::OffsetDateTime;

#[derive(Debug, Clone, Getters, PartialEq, Serialize)]
pub struct Action {
    name: String,
    target: Option<String>,
    #[serde(serialize_with = "serialize_time_format")]
    begin: OffsetDateTime,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    result: std::result::Result<RunningTime, String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RunningTime {
    running_time: String,
}

impl Action {
    pub fn finish(&mut self) {
        let end = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        let mut total_nanos = (end - self.begin).whole_nanoseconds();

        let units = [(1_000_000_000, "s"), (1_000_000, "ms")];

        let mut formate_time = String::new();
        for (unit, unit_name) in units {
            let value = total_nanos / unit;
            if value > 0 {
                formate_time.push_str(&format!("{value}{unit_name}"));
            }
            total_nanos %= unit;
        }
        self.result = Ok(RunningTime {
            running_time: formate_time,
        });
    }
    pub fn err(&mut self, msg: String) {
        self.result = Err(msg);
    }
    pub fn with_target<S: Into<String>>(mut self, target: S) -> Self {
        self.target = Some(target.into());
        self
    }

    pub fn set_command_output<S1: Into<String>, S2: Into<String>>(
        &mut self,
        exit_code: i32,
        stdout: S1,
        stderr: S2,
    ) {
        self.exit_code = Some(exit_code);
        self.stdout = stdout.into();
        self.stderr = stderr.into();
    }

    pub fn set_stdout<S: Into<String>>(&mut self, stdout: S) {
        self.stdout = stdout.into();
    }

    pub fn set_stderr<S: Into<String>>(&mut self, stderr: S) {
        self.stderr = stderr.into();
    }
}

impl From<String> for Action {
    fn from(name: String) -> Self {
        Self {
            name,
            target: None,
            begin: OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            result: Err("unknow".into()),
        }
    }
}
impl From<&String> for Action {
    fn from(name: &String) -> Self {
        Self {
            name: name.clone(),
            target: None,
            begin: OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            result: Err("unknow".into()),
        }
    }
}

impl From<&str> for Action {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            target: None,
            begin: OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            result: Err("unknow".into()),
        }
    }
}

#[test]
fn build_action() {
    let mut action1 = Action::from("test.name");
    action1.finish();
    let mut action2 = Action::from("test.name");
    action2.err("bad".into());
}
