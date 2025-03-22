use std::sync::Arc;

pub trait New0 {
    type Ins;
    fn new() -> Self::Ins;
    fn arc_new() -> Arc<Self::Ins> {
        Arc::new(Self::new())
    }
}

pub trait DefaultCreator<T1, T2> {
    fn default_new(name: T1, args: T2) -> Self;
}

pub trait New1<T> {
    fn new(args: T) -> Self;
}

pub trait NewR1<T, E> {
    fn new(args: T) -> Result<Self, E>
    where
        Self: Sized;
}

pub trait ArcNew1<T> {}

pub trait New2<T1, T2> {
    fn new(a1: T1, a2: T2) -> Self;
}

pub trait New3<T1, T2, T3> {
    fn new(a1: T1, a2: T2, a3: T3) -> Self;
}

pub trait MultiNew2<T1, T2> {
    fn new2(a1: T1, a2: T2) -> Self;
}

pub trait MultiNew3<T1, T2, T3> {
    fn new3(a1: T1, a2: T2, a3: T3) -> Self;
}

pub trait Build1<T> {
    fn build(args: T) -> Self;
}
