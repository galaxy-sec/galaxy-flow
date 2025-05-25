pub fn mod_obj_name(cur_mod: &str, obj_path: &str) -> (String, String) {
    let parts = obj_path.splitn(2, '.').collect::<Vec<&str>>();
    if parts.len() == 1 {
        (cur_mod.to_string(), obj_path.to_string())
    } else {
        (parts[0].to_string(), parts[1].to_string())
    }
}

/*
pub struct ExecPipe {
    keep_rin: PipeReceiver,
    keep_out: PipeSender,
}

impl ExecPipe {
    pub fn new(pipe: Pipe) -> Self {
        Self {
            keep_rin: pipe.0,
            keep_out: pipe.1,
        }
    }
    pub fn next_pipe(&mut self) -> (PipeReceiver, PipeSender) {
        let (next_out, mut next_in) = channel();
        swap(&mut self.keep_rin, &mut next_in);
        (next_in, next_out)
    }
}

impl Drop for ExecPipe {
    fn drop(&mut self) {
        channel_pass_data(&self.keep_rin, &self.keep_out).expect("pipe drop error");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_take_mod_obj() {
        // 基本功能测试
        assert_eq!(take_mod_obj("c", "a.b"), ("a".to_string(), "b".to_string()));
        assert_eq!(take_mod_obj("c", "b"), ("c".to_string(), "b".to_string()));

        // 空字符串情况
        assert_eq!(
            take_mod_obj("test", ""),
            ("test".to_string(), "".to_string())
        );

        // 多个点的路径
        assert_eq!(
            take_mod_obj("c", "a.b.c"),
            ("a".to_string(), "b.c".to_string())
        );

        // 特殊字符和空格
        assert_eq!(
            take_mod_obj("", "my.object"),
            ("my".to_string(), "object".to_string())
        );
        assert_eq!(
            take_mod_obj("test", "a b.c"),
            ("a b".to_string(), "c".to_string())
        );

        // 当前模块名包含点
        assert_eq!(
            take_mod_obj("my.module", "a.b.c"),
            ("a".to_string(), "b.c".to_string())
        );
    }
}

*/
