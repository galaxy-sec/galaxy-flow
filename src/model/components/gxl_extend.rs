#[derive(Clone, Debug, Getters)]
pub struct ModGitAddr {
    remote: String,
    channel: String,
}
impl ModGitAddr {
    pub fn new<S: Into<String>>(remote: S, channel: S) -> Self {
        Self {
            remote: remote.into(),
            channel: channel.into(),
        }
    }
}
#[derive(Clone, Debug, Getters)]
pub struct ModLocAddr {
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
    mods: Vec<String>,
    addr: ModAddr,
}
impl ModRef {
    pub fn new(mods: Vec<String>, addr: ModAddr) -> Self {
        Self { mods, addr }
    }
}
