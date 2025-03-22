use std::sync::{Arc, RwLock};

use crate::friendly::new::Build1;
pub use append::AppendAble;
pub use new::{ArcNew1, DefaultCreator, MultiNew2, MultiNew3, New1, New2, New3, NewR1};

pub mod append;
pub mod conv;
pub mod new;

pub type SafeH<T> = Arc<RwLock<T>>;

impl<T> Build1<T> for SafeH<T> {
    fn build(args: T) -> Self {
        Arc::new(RwLock::new(args))
    }
}

pub trait Abstract {
    fn abstract_info(&self) -> String;
}
