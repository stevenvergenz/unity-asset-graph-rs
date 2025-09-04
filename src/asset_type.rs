use std::{
    convert::From,
    fmt::{Display, Formatter, Result},
    path::PathBuf,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, PartialEq, Eq, Default)]
pub enum AssetType {
    #[default]
    Unknown,
    Directory,
    Prefab,
    Scene,
    Asset,
    Texture,
    Model,
    Audio,
    Script,
    LocResource,
    LocString,
    BrokenRef,
}

impl AssetType {
    pub fn is_unity(&self) -> bool {
        match self {
            Self::Prefab | Self::Scene | Self::Asset => true,
            _ => false,
        }
    }
}

impl From<&PathBuf> for AssetType {
    fn from(value: &PathBuf) -> Self {
        match value.extension().and_then(|s| s.to_str()) {
            Some("prefab") => AssetType::Prefab,
            Some("unity") | Some("scene") => AssetType::Scene,
            Some("asset") => AssetType::Asset,
            Some("png") | Some("jpg") | Some("jpeg") => AssetType::Texture,
            Some("fbx") | Some("obj") => AssetType::Model,
            Some("wav") | Some("mp3") => AssetType::Audio,
            Some("cs") => AssetType::Script,
            _ => AssetType::Unknown,
        }
    }
}

impl Display for AssetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            AssetType::Unknown => write!(f, "Unknown"),
            AssetType::Directory => write!(f, "Directory"),
            AssetType::Prefab => write!(f, "Prefab"),
            AssetType::Scene => write!(f, "Scene"),
            AssetType::Texture => write!(f, "Texture"),
            AssetType::Model => write!(f, "Model"),
            AssetType::Audio => write!(f, "Audio"),
            AssetType::Script => write!(f, "Script"),
            AssetType::LocResource => write!(f, "Localization Resource"),
            AssetType::LocString => write!(f, "Localized String"),
            AssetType::BrokenRef => write!(f, "Broken Reference"),
            AssetType::Asset => write!(f, "Asset"),
        }
    }
}
