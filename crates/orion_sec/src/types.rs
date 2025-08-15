use indexmap::IndexMap;
use unicase::UniCase;

pub type UniString = UniCase<String>;
pub type UniCaseMap<T> = IndexMap<UniCase<String>, T>;
