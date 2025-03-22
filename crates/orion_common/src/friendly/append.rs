pub trait AppendAble<T> {
    fn append(&mut self, now: T);
}
pub trait AppendAble2<T1, T2> {
    fn append(&mut self, first: T1, second: T2);
}
pub trait AppendAble3<T1, T2, T3> {
    fn append(&mut self, first: T1, second: T2, third: T3);
}

pub trait InsertAble<K, V> {
    fn insert(&mut self, key: K, value: V);
}
