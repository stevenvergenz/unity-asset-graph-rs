use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Serialize, Deserialize)]
pub enum Id {
    None,
    Guid(Uuid),
    Loc(String),
    CsDeclaration(String),
}

impl Default for Id {
    fn default() -> Self {
        Id::Guid(Uuid::nil())
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Guid(uuid) => write!(f, "guid:{}", uuid),
            Self::Loc(name) => write!(f, "loc:{}", name),
            Self::CsDeclaration(name) => write!(f, "cs_decl:{}", name),
        }
    }
}
