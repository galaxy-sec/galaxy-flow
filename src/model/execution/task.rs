use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Getters, PartialEq)]
pub struct Task {
    name: String,
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
}

impl From<String> for Task {
    fn from(name: String) -> Self {
        Self {
            name,
            begin: SystemTime::now(),
            result: Err("unknow".into()),
        }
    }
}
impl From<&String> for Task {
    fn from(name: &String) -> Self {
        Self {
            name: name.clone(),
            begin: SystemTime::now(),
            result: Err("unknow".into()),
        }
    }
}

impl From<&str> for Task {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
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
