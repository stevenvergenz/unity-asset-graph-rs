use std::{
    collections::HashMap,
    fs,
    io::BufRead,
    path::PathBuf,
    sync::{LazyLock, Arc, Mutex},
};
use regex::Regex;
use uuid::Uuid;
use crate::{
    asset::Asset, database::assets, id::Id, parser, progress::ProgressIndicator, util::read_file_no_bom
};

use super::{Database, DatabaseError};

static META_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^guid: ([0-9a-f]{32})$").expect("Failed to compile meta id regex")
});

const THREADS: usize = 8;

impl Database {
    pub fn find_assets(&mut self) -> Result<(), DatabaseError> {
        let mut progress = ProgressIndicator::new("Finding assets", None);

        let mut paths = Vec::new();
        for root in &self.roots {
            paths.push(root.clone());
        }
        let paths = Arc::new(Mutex::new(paths));
        let assets = Arc::new(Mutex::new(HashMap::new()));
        let mut handles = vec![];

        for _ in 0..THREADS {
            let mut paths = Arc::clone(&paths);
            let mut assets = Arc::clone(&assets);
            let relative_to = self.relative_to.clone();
            handles.push(std::thread::spawn(move || {
                loop {
                    let path = match &paths.lock().unwrap().pop() {
                        Some(p) => p.clone(),
                        None => break,
                    };

                    Self::find_assets_job(&path, relative_to.as_ref(), &mut paths, &mut assets)
                        .unwrap_or_else(|e| eprintln!("Error finding assets in '{}': {}", path.display(), e));
                }
            }));
        }

        handles.into_iter().for_each(|h| {
            if let Err(e) = h.join() {
                eprintln!("Error joining thread: {:?}", e);
            }
        });

        // for root in self.roots.iter() {
        //     let abs_root = match self.relative_to.as_ref() {
        //         Some(rel) => rel.join(root),
        //         None => root.clone(),
        //     };
        //     if let Err(e) = Self::find_assets_in_dir(&abs_root, self.relative_to.as_ref(), &mut self.assets, &mut progress) {
        //         eprintln!("Error finding assets in '{}': {}", root.display(), e);
        //     }
        // }

        progress.finish(&format!("Database populated with {} assets from {} roots", self.assets.len(), self.roots.len()));
        Ok(())
    }

    pub fn resolve_assets(&mut self) -> () {
        let mut progress = ProgressIndicator::new(
            "Searching assets for dependencies", 
            Some(self.assets.len()),
        );

        let ids: Vec<Id> = self.assets.keys().cloned().collect();
        for id in ids {
            let asset = self.assets.get_mut(&id).expect("Asset should exist in database");
            let subassets = match parser::parse(asset, self.relative_to.as_ref()) {
                Ok(subs) => subs,
                Err(e) => {
                    eprintln!("Error resolving dependencies for asset '{}': {}", asset.path.display(), e);
                    continue;
                },
            };

            for sub in subassets {
                if !self.assets.contains_key(&sub.id) {
                    self.assets.insert(sub.id.clone(), sub);
                }
            }
            progress.advance();
        }

        progress.finish(&format!("Resolved dependencies for {} assets", self.assets.len()));
    }

    // fn find_assets_in_dir(
    //     path: &PathBuf, 
    //     relative_to: Option<&PathBuf>, 
    //     assets: &mut HashMap<Id, Asset>,
    //     progress: &mut ProgressIndicator,
    // ) -> Result<(), DatabaseError> {
    //     let dir = match fs::read_dir(path) {
    //         Ok(d) => d,
    //         Err(e) => {
    //             return Err(DatabaseError { message: format!("Error reading directory '{}': {}", path.display(), e), inner: None });
    //         },
    //     };
    //     for entry in dir {
    //         let entry = match entry {
    //             Ok(e) => e,
    //             Err(e) => {
    //                 eprintln!("Error reading entry in '{}': {}", path.display(), e);
    //                 continue;
    //             }
    //         };

    //         // skip non-meta files
    //         let meta_path = match entry.path().extension().and_then(|s| s.to_str()) {
    //             Some("meta") => entry.path(),
    //             _ => continue,
    //         };

    //         // read the meta file
    //         let meta_reader = match read_file_no_bom(&meta_path) {
    //             Ok(r) => r,
    //             Err(e) => return Err(DatabaseError {
    //                 message: format!("failed to read meta file '{}'", meta_path.display()),
    //                 inner: Some(Box::new(e)),
    //             }),
    //         };

    //         let mut asset_guid = None;
    //         for line in meta_reader.lines() {
    //             if let Ok(line) = line
    //                 && let Some(captures) = META_REGEX.captures(&line)
    //                 && let Some(m) = captures.get(1)
    //                 && let Ok(uuid) = Uuid::parse_str(m.as_str()){
    //                 // Extract the GUID from the meta file
    //                 asset_guid = Some(uuid);
    //                 break;
    //             }
    //         }
    //         let asset_guid = asset_guid.expect("Meta file must contain a valid GUID");

    //         // process the asset file
    //         let asset_path = meta_path.with_extension("");

    //         if asset_path.is_dir() {
    //             // Recursively find assets in subdirectories
    //             if let Err(e) = Self::find_assets_in_dir(&asset_path, relative_to, assets, progress) {
    //                 eprintln!("Error finding assets in '{}': {}", asset_path.display(), e);
    //             }
    //         } else if asset_path.is_file() {
    //             let rel_path = if let Some(rel_to) = relative_to
    //                 && let Ok(rel) = asset_path.strip_prefix(rel_to) {
    //                 PathBuf::from(rel)
    //             }
    //             else {
    //                 asset_path
    //             };
    //             let asset = Asset::new(Id::Guid(asset_guid), rel_path);
    //             assets.insert(asset.id.clone(), asset);
    //             progress.advance();
    //         }
    //     }

    //     Ok(())
    // }

    fn find_assets_job(
        path: &PathBuf, 
        relative_to: Option<&PathBuf>,
        paths: &mut Arc<Mutex<Vec<PathBuf>>>,
        assets: &mut Arc<Mutex<HashMap<Id, Asset>>>,
    ) -> Result<(), DatabaseError>{
        let path = match &relative_to {
            Some(rel) => &rel.join(path),
            None => path,
        };

        if !path.exists() {
            return Err(DatabaseError { message: format!("Asset path '{}' does not exist", path.display()), inner: None });
        }

        if path.is_dir() {
            let files = match path.read_dir() {
                Ok(files) => files,
                Err(e) => return Err(DatabaseError { message: format!("Failed to read directory '{}': {}", path.display(), e), inner: None }),
            };

            for f in files {
                match f {
                    Ok(f) => {
                        let path = f.path();

                        // skip meta files for now
                        if let Some(ext) = path.extension().and_then(|s| s.to_str())
                            && ext == "meta" {
                            continue;
                        }

                        match paths.lock() {
                            Ok(mut paths) => {
                                paths.push(path);
                            },
                            Err(e) => {
                                eprintln!("Error acquiring lock on paths: {e}");
                                continue;
                            },
                        }
                    },
                    Err(e) => {
                        eprintln!("Error reading file in '{}': {}", path.display(), e);
                        continue;
                    }
                }
            }
        }
        else {
            let meta_path = path.join(".meta");
            if !meta_path.exists() {
                return Err(DatabaseError { message: format!("Meta file '{}' does not exist", meta_path.display()), inner: None });
            }

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

            let rel_path = if let Some(rel_to) = relative_to.as_ref()
                && let Ok(rel) = path.strip_prefix(rel_to) {
                PathBuf::from(rel)
            }
            else {
                path.clone()
            };

            let asset = Asset::new(Id::Guid(asset_guid), rel_path);

            match assets.lock() {
                Ok(mut assets) => {
                    assets.insert(asset.id.clone(), asset);
                },
                Err(_) => {
                    return Err(DatabaseError { message: format!("Failed to acquire lock on assets"), inner: None });
                },
            }
        }

        Ok(())
    }
}