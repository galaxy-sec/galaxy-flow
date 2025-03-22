pub fn opt_zero(v: usize) -> Option<usize> {
    if v == 0 {
        None
    } else {
        Some(v)
    }
}

pub fn opt_or<T>(first: Option<T>, second: Option<T>) -> Option<T> {
    if first.is_some() {
        first
    } else {
        second
    }
}

pub fn val_or<T>(first: Option<T>, second: T) -> T {
    if let Some(value) = first {
        value
    } else {
        second
    }
}

pub trait OptionConv<T, E> {
    fn no_less(self, name: &str) -> Result<T, E>;
    fn no_empty(self) -> Result<T, E>;
}

pub trait OptionConvRef<'a, T, E> {
    fn no_less(&'a self, name: &str) -> Result<&'a T, E>;
    fn no_empty(&self) -> Result<&T, E>;
}

pub trait OptionError {
    fn empty() -> Self;
    fn less(msg: String) -> Self;
}
pub trait OptionConvTag {}

impl<T, E> OptionConv<T, E> for Option<T>
where
    E: OptionError,
    T: OptionConvTag,
{
    fn no_less(self, name: &str) -> Result<T, E> {
        self.ok_or(E::less(format!("{} less", name)))
    }

    fn no_empty(self) -> Result<T, E> {
        self.ok_or(E::empty())
    }
}

/*
impl<'a ,T, E> OptionConvRef<'a,T, E> for Option<T>
    where
        E: OptionError,
        T: OptionConvTag,
{
    fn no_less(&'a self, name: &str) -> Result<&'a T, E> {
        self.ok_or(E::less(format!("{} less", name)))
    }

    fn no_empty(& self) -> Result<&T, E> {
        self.ok_or(E::empty())
    }
}


 */
