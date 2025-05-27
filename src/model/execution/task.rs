use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Getters, PartialEq)]
pub struct Task {
    name: String,
    target: Option<String>,
    begin: SystemTime,
    result: std::result::Result<Duration, String>,
}
impl Task {
    pub fn finish(&mut self) {
        self.result = Ok(self.begin.elapsed().unwrap());
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
