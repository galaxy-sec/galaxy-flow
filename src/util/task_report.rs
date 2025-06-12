use std::{
    fs::{create_dir_all, File},
    path::Path,
};

use orion_common::conf::export_toml;
use time::{format_description, OffsetDateTime};

use crate::ability::prelude::ExecOut;

// 任务直接结果本地化
pub fn task_local_report(out: ExecOut) {
    let datetime = OffsetDateTime::now_utc();
    let format = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]");
    let now = datetime.format(&format.unwrap()).unwrap();
    let dir_path = "_gal/.report";
    let path = Path::new(dir_path);

    // report目录不存在则创建
    if !path.exists() {
        if let Err(e) = create_dir_all(path) {
            println!("Failed to create directory '{}': {}", dir_path, e);
            return;
        }
    }

    let file_name = format!("{}/task_{}.toml", dir_path, now);

    // 创建task结果存储文件
    match File::create(&file_name) {
        Ok(_) => {
            if let Err(e) = export_toml(&out, &file_name) {
                println!("Failed to export toml: {}", e);
            }
        }
        Err(_) => {
            println!("Failed to create file: {}", file_name);
        }
    }
}
