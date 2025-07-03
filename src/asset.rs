use std::{
    path::PathBuf,
    collections::HashSet,
};
use serde::{Deserialize, Serialize};
use crate::{
    id::Id,
    parsers::{unity::UnityObject, ParseError, Parser},
};

#[derive(Deserialize, Serialize)]
pub enum AssetType {
    Prefab,
    Scene,
    Texture,
    Model,
    Audio,
    Script,
    Unknown,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetType::Prefab => write!(f, "Prefab"),
            AssetType::Scene => write!(f, "Scene"),
            AssetType::Texture => write!(f, "Texture"),
            AssetType::Model => write!(f, "Model"),
            AssetType::Audio => write!(f, "Audio"),
            AssetType::Script => write!(f, "Script"),
            AssetType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Asset {
    pub id: Id,
    pub asset_type: AssetType,
    pub path: PathBuf,
    pub loc_roots: HashSet<PathBuf>,
    pub dependencies: HashSet<Id>,
}

impl Asset {
    pub fn new(id: Id, path: PathBuf) -> Self {
        let asset_type = match path.extension().and_then(|s| s.to_str()) {
            Some("prefab") => AssetType::Prefab,
            Some("unity") | Some("scene") => AssetType::Scene,
            Some("png") | Some("jpg") | Some("jpeg") => AssetType::Texture,
            Some("fbx") | Some("obj") => AssetType::Model,
            Some("wav") | Some("mp3") => AssetType::Audio,
            Some("cs") | Some("js") => AssetType::Script,
            _ => AssetType::Unknown,
        };

        Self {
            id,
            asset_type,
            path,
            loc_roots: HashSet::new(),
            dependencies: HashSet::new(),
        }
    }

    pub fn read_contents(&mut self, relative_to: Option<&PathBuf>) -> Result<(), ParseError> {
        match self.asset_type {
            AssetType::Prefab => UnityObject::parse(self, relative_to),
            _ => { Ok(()) },
        }
    }
}

impl std::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Asset ID: {}", self.id)?;
        writeln!(f, "Type: {}", self.asset_type)?;
        writeln!(f, "Path: {}", self.path.display())?;
        writeln!(f, "Dependencies:")?;
        for dep in &self.dependencies {
            writeln!(f, " - {}", dep)?;
        }
        Ok(())
    }
}