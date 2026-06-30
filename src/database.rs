use std::{
    cell::RefCell, collections::{HashMap, HashSet }, fmt::Display, fs, path::PathBuf,
};
use serde::{Deserialize, Serialize};
use regex::{Regex, RegexBuilder};
use crate::{
    BoundAsset, QualifiedName, QualifiedNameOwned, asset::Asset, asset_type::AssetType, id::Id, parser::{ParseError, TypeBroker}
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
                let (rel_id, rel_asset) = match self.assets.remove_entry(relation.id()) {
                    Some((rel_id, mut rel_asset)) => {
                        rel_asset.back_relations.insert(asset.invert_relation(relation));
                        (rel_id, rel_asset)
                    },
                    None => {
                        let a = Asset::new(
                            relation.id().clone(), 
                            AssetType::BrokenRef, 
                            None, 
                            [asset.invert_relation(relation)],
                        );
                        (relation.id().clone(), a)
                    },
                };
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

    pub fn asset<'a>(&'a self, id: &Id) -> Option<BoundAsset<'a>> {
        self.assets.get(id).map(|a| a.bind(self))
    }

    pub fn asset_mut(&mut self, id: &Id) -> Option<&mut Asset> {
        self.assets.get_mut(id)
    }

    pub fn assets<'a>(&'a self) -> impl Iterator<Item = BoundAsset<'a>> {
        self.assets.values().map(|a| a.bind(self))
    }

    pub fn find_assets_by_path<'a>(&'a self, filter: &AssetFilter) -> impl ExactSizeIterator<Item = BoundAsset<'a>> {
        let mut out = vec![];
        for a in self.assets.values() {
            if a.bind(self).path_matches(filter) {
                out.push(a.bind(self));
            }
        }
        out.into_iter()
    }

    pub fn find_assets_by_id<'a>(&'a self, filter: &AssetFilter) -> impl ExactSizeIterator<Item = BoundAsset<'a>> {
        let mut out = vec![];
        for asset in self.assets.values() {
            if asset.id_matches(filter) {
                out.push(asset.bind(self));
            }
        }

        out.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct AssetFilter {
    re: Regex,
    invert: bool,
}

impl AssetFilter {
    pub fn new(re: Regex, invert: bool) -> Self {
        Self { re, invert }
    }

    pub fn matches(&self, a: &str) -> bool {
        self.invert ^ self.re.is_match(a)
    }
}

impl TryFrom<&str> for AssetFilter {
    type Error = regex::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (invert, pat) = if value.starts_with('~') {
            (true, value.split_at(1).1)
        } else {
            (false, value)
        };
        Ok(Self {
            invert,
            re: RegexBuilder::new(pat).unicode(false).build()?,
        })
    }
}

impl TryFrom<&String> for AssetFilter {
    type Error = regex::Error;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<String> for AssetFilter {
    type Error = regex::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl Display for AssetFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}",
            if self.invert { "~" } else { "" },
            self.re,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn asset_filter() {
        let filter = AssetFilter::try_from("abcd").unwrap();
        assert!(filter.matches("abcdefg"));
        assert!(!filter.matches("cdefg"));
        
        let filter = AssetFilter::try_from("~abcd").unwrap();
        assert!(!filter.matches("abcdefg"));
        assert!(filter.matches("cdefg"));
    }
}
