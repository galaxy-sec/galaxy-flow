use std::time::SystemTime;

use serde::ser::Serializer;
use serde::Serialize;
use time::OffsetDateTime;

#[derive(Debug, Clone, Getters, PartialEq, Serialize)]
pub struct Task {
    name: String,
    target: Option<String>,
    #[serde(serialize_with = "serialize_fmt")]
    begin: SystemTime,
    pub stdout: String,
    result: std::result::Result<RunningTime, String>,
}

#[derive(Debug, Clone, Getters, PartialEq, Serialize)]
pub struct Action {
    name: String,
    target: Option<String>,
    #[serde(serialize_with = "serialize_fmt")]
    begin: SystemTime,
    pub stdout: String,
    result: std::result::Result<RunningTime, String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RunningTime {
    running_time: u64,
}

// 序列化进行时间格式化
fn serialize_fmt<S>(value: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let datetime = OffsetDateTime::from(*value);

    datetime
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
        .serialize(serializer)
}

impl Task {
    pub fn finish(&mut self) {
        let end = SystemTime::now();
        let duration = end.duration_since(self.begin).unwrap();
        self.result = Ok(RunningTime {
            running_time: duration.as_micros() as u64,
        });
    }
    pub fn err(&mut self, msg: String) {
        self.result = Err(msg);
    }
    pub fn with_target<S: Into<String>>(mut self, target: S) -> Self {
        self.target = Some(target.into());
        self
    }
}

impl From<String> for Task {
    fn from(name: String) -> Self {
        Self {
            name,
            target: None,
            begin: SystemTime::now(),
            stdout: String::new(),
            result: Err("unknow".into()),
        }
    }
}
impl From<&String> for Task {
    fn from(name: &String) -> Self {
        Self {
            name: name.clone(),
            target: None,
            begin: SystemTime::now(),
            stdout: String::new(),
            result: Err("unknow".into()),
        }
    }
}

impl From<&str> for Task {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            target: None,
            begin: SystemTime::now(),
            stdout: String::new(),
            result: Err("unknow".into()),
        }
    }
}

#[test]
fn build_task() {
    let mut task1 = Task::from("test.name");
    task1.finish();
    let mut task2 = Task::from("test.name");
    task2.err("bad".into());
}
