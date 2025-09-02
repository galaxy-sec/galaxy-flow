use std::path::PathBuf;

pub fn first_parent_file(file_name: &str) -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().expect("Failed to get current directory");

    loop {
        let project_file = current_dir.join(file_name);
        if project_file.exists() {
            //let project_root = current_dir.clone();
            return Some(current_dir.join(file_name));
        }

        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => break, // 已到达根目录
        }
    }

    None
}
