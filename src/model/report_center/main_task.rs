use crate::util::http_handle::{get_main_task_create_url, send_http_request};
use serde::Serialize;
use std::env;
use time::{format_description, OffsetDateTime};

#[derive(Debug, Serialize, Clone)]
pub struct MainTask {
    pub maintask_name: String,
    pub worker_name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub id: i64,
}

pub async fn create_main_task(task_name: String) {
    // 创建主任务
    let datetime = OffsetDateTime::now_utc();
    let format: Result<
        Vec<format_description::BorrowedFormatItem<'_>>,
        time::error::InvalidFormatDescription,
    > = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]");
    let mut now = String::new();
    match format {
        Ok(fmt) => now = datetime.format(&fmt).unwrap_or_default(),
        Err(e) => println!("create main task time format error: {}", e),
    }
    let parent_id = datetime.unix_timestamp();
    let main_task = MainTask {
        id: parent_id,
        maintask_name: format!("{} {}", task_name, now),
        worker_name: String::new(),
        description: Some(task_name.clone()),
        task_type: task_name,
    };
    // 设置环境变量中的父id
    std::env::set_var("task_id", parent_id.to_string());
    // 创建主任务
    if let Some(url) = get_main_task_create_url() {
        match send_http_request(main_task, &url).await {
            Ok(response) => {
                if response.status().is_success() {
                    println!("create maintask success");
                } else {
                    println!("create maintask error: {:?}", response.text().await);
                }
            }
            Err(e) => {
                println!("create maintask error: {}", e);
            }
        }
    }
}

// 获取当前任务的父id
pub fn get_task_parent_id() -> Option<String> {
    env::var("task_id").ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_task_parent_id() {
        let parent_id = 123;
        env::set_var("task_id", parent_id.to_string());

        let retrieved_id = get_task_parent_id().unwrap();
        assert_eq!(retrieved_id, parent_id.to_string());
    }

    #[tokio::test]
    async fn test_create_main_task() {
        let task_name = String::from("Test Task");
        create_main_task(task_name.clone()).await;

        let parent_id = env::var("task_id").unwrap();
        assert!(parent_id.parse::<i64>().is_ok());
    }
}

