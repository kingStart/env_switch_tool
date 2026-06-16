use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GroupKind {
    #[default]
    Env,
    Hosts,
}

impl fmt::Display for GroupKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Env => write!(f, "env"),
            Self::Hosts => write!(f, "hosts"),
        }
    }
}

impl GroupKind {
    pub fn parse(s: &str) -> Self {
        match s {
            "hosts" => Self::Hosts,
            _ => Self::Env,
        }
    }
}
