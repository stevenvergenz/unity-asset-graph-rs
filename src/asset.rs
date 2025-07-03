use std::{
    path::PathBuf,
    collections::HashSet,
};
use serde::Serialize;
use crate::{
    id::Id,
    database::Database,
    parsers::{unity::UnityObject, ParseError, Parser},
};

#[derive(Serialize)]
pub enum AssetType {
    Prefab,
    Scene,
    Texture,
    Model,
    Audio,
    Script,
    Unknown,
}

#[derive(Serialize)]
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

    pub fn read_contents(&mut self) -> Result<(), ParseError> {
        match self.asset_type {
            AssetType::Prefab => UnityObject::parse(self),
            _ => { Ok(()) },
        }
    }
}