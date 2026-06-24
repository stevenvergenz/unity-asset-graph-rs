use std::{
    collections::{HashSet, HashMap },
    path::PathBuf,
    fs,
    cell::RefCell,
};
use serde::{Deserialize, Serialize};
use regex::RegexBuilder;
use crate::{
    QualifiedName, QualifiedNameOwned, asset::Asset, asset_type::AssetType, id::Id, parser::{ParseError, TypeBroker}
};

mod roots;
mod populate_pass1;
mod populate_pass2;
mod populate_pass3;

#[derive(Debug)]
pub enum DatabaseError {
    Parse(ParseError),
    Regex(regex::Error),
    BadPath(PathBuf),
}

impl DatabaseError {
    pub fn parse(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::Parse(ParseError::new(path, message))
    }
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(p) => write!(f, "Parse error: {p}"),
            Self::Regex(r) => write!(f, "Regex error: {r}"),
            Self::BadPath(p) => write!(f, "Bad path: {}", p.display()),
        }
    }
}

impl std::error::Error for DatabaseError {}

#[derive(Deserialize, Serialize)]
pub struct Database {
    relative_to: Option<PathBuf>,
    roots: HashSet<PathBuf>,
    loc_roots: HashSet<PathBuf>,
    assets: HashMap<Id, Asset>,
    #[serde(skip)]
    id_strs: RefCell<HashMap<Id, String>>,
}

impl Database {
    pub fn new(root: &str, relative_to: Option<&str>) -> Result<Self, DatabaseError> {
        let relative_to = if let Some(pathstr) = relative_to
            && let Ok(path) = fs::canonicalize(pathstr)
        {
            Some(path)
        }
        else {
            eprintln!("failed to canonicalize relative path '{relative_to:?}'");
            None
        };

        let mut db = Self {
            relative_to,
            roots: HashSet::new(),
            loc_roots: HashSet::new(),
            assets: HashMap::new(),
            id_strs: RefCell::new(HashMap::new()),
        };

        db.add_root_str(root).map(|_| db)
    }

    pub fn populate(&mut self) -> Result<(), DatabaseError> {
        self.populate_pass1_find()?;
        let broker = self.populate_pass2_resolve()?;
        self.populate_pass3_link(broker)?;
        self.populate_reverse_dependencies();
        Ok(())
    }

    pub fn populate_reverse_dependencies(&mut self) {
        // loop over a copy of the keys, and take the assets out of the map while we do this
        // so we can mutate them
        let keys: Vec<Id> = self.assets.keys().cloned().collect();
        for asset_id in keys.iter() {
            let (asset_id, asset) = match self.assets.remove_entry(asset_id) {
                Some(e) => e,
                None => continue,
            };

            // loop over the asset's (forward) relations
            for relation in asset.relations.iter() {
                // take the related asset out of the map too
                let (rel_id, mut rel_asset) = match self.assets.remove_entry(relation.id()) {
                    Some(e) => e,
                    None => {
                        let a = Asset {
                            id: relation.id().clone(),
                            asset_type: AssetType::BrokenRef,
                            ..Default::default()
                        };
                        (relation.id().clone(), a)
                    },
                };
                // add the back relation to the related asset
                rel_asset.back_relations.insert(asset.invert_relation(relation));
                self.assets.insert(rel_id, rel_asset);
            }
            self.assets.insert(asset_id, asset);
        }
    }

    pub fn roots(&self) -> &HashSet<PathBuf> {
        &self.roots
    }

    pub fn loc_roots(&self) -> impl Iterator<Item = &PathBuf> {
        self.loc_roots.iter()
    }

    pub fn asset(&self, id: &Id) -> Option<&Asset> {
        self.assets.get(id)
    }

    pub fn asset_mut(&mut self, id: &Id) -> Option<&mut Asset> {
        self.assets.get_mut(id)
    }

    pub fn assets(&self) -> impl Iterator<Item = &Asset> {
        self.assets.values()
    }

    pub fn find_assets_by_name(&self, regex: &str) -> Result<impl ExactSizeIterator<Item = &Asset>, DatabaseError> {
        let re = RegexBuilder::new(regex)
            .unicode(false)
            .build()
            .map_err(|e| DatabaseError::Regex(e))?;

        let mut out = vec![];
        for a in self.assets.values() {
            if let Some(haystack) = a.path.as_ref().and_then(|p| Some(p.to_string_lossy()))
                && re.is_match(&haystack) {
                out.push(a);
            }
        }
        Ok(out.into_iter())
    }

    pub fn find_assets_by_id(&self, regex: &str) -> Result<impl ExactSizeIterator<Item = &Asset>, DatabaseError> {
        let re = RegexBuilder::new(regex)
            .unicode(false)
            .build()
            .map_err(|e| DatabaseError::Regex(e))?;

        let mut strmap = self.id_strs.take();
        let mut out = vec![];
        for (id, asset) in self.assets.iter() {
            let idstr = if let Some(s) = strmap.get(id) {
                s
            } else {
                strmap.insert(id.clone(), id.to_string());
                strmap.get(id).unwrap()
            };

            if re.is_match(idstr) {
                out.push(asset);
            }
        }

        self.id_strs.replace(strmap);
        Ok(out.into_iter())
    }
}
