use getset::{Getters, Setters, WithSetters};
#[derive(Clone, Debug, WithSetters, Setters, Default, Getters)]
pub struct ModGitAddr {
    #[getset(get = "pub", set = "pub")]
    remote: String,
    #[getset(get = "pub", set = "pub", set_with = "pub")]
    branch: Option<String>,
    #[getset(get = "pub", set = "pub", set_with = "pub")]
    tag: Option<String>,
}
impl ModGitAddr {
    pub fn new<S: Into<String>>(remote: S) -> Self {
        Self {
            remote: remote.into(),
            ..Default::default()
        }
    }
}
#[derive(Clone, Debug, Getters)]
pub struct ModLocAddr {
    #[getset(get = "pub")]
    path: String,
}
impl ModLocAddr {
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self { path: path.into() }
    }
}
#[derive(Clone, Debug)]
pub enum ModAddr {
    Git(ModGitAddr),
    Loc(ModLocAddr),
}
#[derive(Clone, Debug, Getters)]
pub struct ModRef {
    #[getset(get = "pub")]
    mods: Vec<String>,
    #[getset(get = "pub")]
    addr: ModAddr,
}
impl ModRef {
    pub fn new(mods: Vec<String>, addr: ModAddr) -> Self {
        Self { mods, addr }
    }
}
