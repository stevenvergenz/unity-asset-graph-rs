use std::{
    collections::HashMap,
    fs,
    io::BufRead,
    path::PathBuf,
    sync::LazyLock,
};
use regex::Regex;
use uuid::Uuid;
use crate::{
    asset::Asset,
    id::Id,
    util::read_file_no_bom
};

use super::{Database, DatabaseError};

static META_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^guid: ([0-9a-f]{32})$").expect("Failed to compile meta id regex")
});

impl Database {
    pub fn find_assets(&mut self) -> Result<(), DatabaseError> {
        for root in self.roots.iter() {
            let abs_root = match self.relative_to.as_ref() {
                Some(rel) => rel.join(root),
                None => root.clone(),
            };
            if let Err(e) = Self::find_assets_in_dir(&abs_root, self.relative_to.as_ref(), &mut self.assets) {
                eprintln!("Error finding assets in '{}': {}", root.display(), e);
            }
        }
        Ok(())
    }

    pub fn resolve_assets(&mut self) -> () {
        for asset in self.assets.values_mut() {
            if let Err(e) = asset.read_contents(self.relative_to.as_ref()) {
                eprintln!("Error resolving dependencies for asset '{}': {}", asset.path.display(), e);
            }
        }
    }

    fn find_assets_in_dir(
        path: &PathBuf, 
        relative_to: Option<&PathBuf>, 
        assets: &mut HashMap<Id, Asset>,
    )-> Result<(), DatabaseError> {
        let dir = match fs::read_dir(path) {
            Ok(d) => d,
            Err(e) => {
                return Err(DatabaseError { message: format!("Error reading directory '{}': {}", path.display(), e), inner: None });
            },
        };
        for entry in dir {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Error reading entry in '{}': {}", path.display(), e);
                    continue;
                }
            };

            // skip non-meta files
            let meta_path = match entry.path().extension().and_then(|s| s.to_str()) {
                Some("meta") => entry.path(),
                _ => continue,
            };

            // read the meta file
            let meta_reader = match read_file_no_bom(&meta_path) {
                Ok(r) => r,
                Err(e) => return Err(DatabaseError {
                    message: format!("failed to read meta file '{}'", meta_path.display()),
                    inner: Some(Box::new(e)),
                }),
            };

            let mut asset_guid = None;
            for line in meta_reader.lines() {
                if let Ok(line) = line
                    && let Some(captures) = META_REGEX.captures(&line)
                    && let Some(m) = captures.get(1)
                    && let Ok(uuid) = Uuid::parse_str(m.as_str()){
                    // Extract the GUID from the meta file
                    asset_guid = Some(uuid);
                    break;
                }
            }
            let asset_guid = asset_guid.expect("Meta file must contain a valid GUID");

            // process the asset file
            let asset_path = meta_path.with_extension("");

            if asset_path.is_dir() {
                // Recursively find assets in subdirectories
                if let Err(e) = Self::find_assets_in_dir(&asset_path, relative_to, assets) {
                    eprintln!("Error finding assets in '{}': {}", asset_path.display(), e);
                }
            } else if asset_path.is_file() {
                let rel_path = if let Some(rel_to) = relative_to
                    && let Ok(rel) = asset_path.strip_prefix(rel_to) {
                    PathBuf::from(rel)
                }
                else {
                    asset_path
                };
                let asset = Asset::new(Id::new_uuid(asset_guid), rel_path);
                assets.insert(asset.id.clone(), asset);
            }
        }

        Ok(())
    }
}