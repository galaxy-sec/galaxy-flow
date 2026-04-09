pub trait AppendAble<T> {
    fn append(&mut self, now: T);
}
