pub trait LocalInto<T> {
    fn into(self) -> T;
}

impl<const N: usize> LocalInto<Vec<String>> for [&str; N] {
    fn into(self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}

impl LocalInto<Vec<String>> for Vec<&str> {
    fn into(self) -> Vec<String> {
        self.iter().map(|s| s.to_string()).collect()
    }
}
