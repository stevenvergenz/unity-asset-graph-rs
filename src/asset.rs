use std::{
    collections::HashSet,
    path::PathBuf,
};
use serde::{Deserialize, Serialize};
use crate::{
    asset_type::AssetType,
    id::Id,
    database::Database,
};

#[derive(Deserialize, Serialize, Default)]
pub struct Asset {
    pub id: Id,
    pub asset_type: AssetType,
    pub path: Option<PathBuf>,
    pub dependencies: HashSet<Id>,

    #[serde(skip)]
    pub dependents: HashSet<Id>,
}

impl Asset {
    pub fn bind<'a, 'b>(&'a self, db: &'b Database) -> BoundAsset<'a, 'b> {
        BoundAsset {
            asset: self,
            db,
            indent: 0,
        }
    }
}

pub struct BoundAsset<'a, 'b> {
    pub asset: &'a Asset,
    pub db: &'b Database,
    indent: usize,
}

impl<'a, 'b> BoundAsset<'a, 'b> {
    pub fn indent(self) -> Self {
        Self {
            asset: self.asset,
            db: self.db,
            indent: self.indent + 1,
        }
    }

    pub fn unindent(self) -> Self{
        Self {
            asset: self.asset,
            db: self.db,
            indent: self.indent.saturating_sub(1),
        }
    }
}

impl<'a, 'b> std::fmt::Display for BoundAsset<'a, 'b> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let first_indent = format!("{}- ", "  ".repeat(self.indent));
        let indent_str = "  ".repeat(self.indent + 1);
        writeln!(f, "{first_indent}Asset ID: {}", self.asset.id)?;
        writeln!(f, "{indent_str}Type: {}", self.asset.asset_type)?;
        if let Some(path) = &self.asset.path {
            writeln!(f, "{indent_str}Path: {}", path.display())?;
        }

        let mut deps = vec![];
        for dep_id in self.asset.dependents.iter() {
            if let Some(dep_asset) = self.db.asset(dep_id) {
                if let Some(path) = &dep_asset.path {
                    deps.push(path.display().to_string());
                }
                else {
                    deps.push(dep_asset.id.to_string());
                }
            }
            else {
                deps.push(dep_id.to_string());
            }
        }
        deps.sort();

        writeln!(f, "{indent_str}Dependents ({}):", deps.len())?;
        for dep in &deps {
            writeln!(f, "{indent_str}- {}", dep)?;
        }

        let mut deps = vec![];
        for dep_id in self.asset.dependencies.iter() {
            if let Some(dep_asset) = self.db.asset(dep_id) {
                if let Some(path) = &dep_asset.path {
                    deps.push(path.display().to_string());
                }
                else {
                    deps.push(dep_asset.id.to_string());
                }
            }
            else {
                deps.push(dep_id.to_string());
            }
        }
        deps.sort();

        writeln!(f, "{indent_str}Dependencies ({}):", deps.len())?;
        for dep in deps {
            writeln!(f, "{indent_str}- {}", dep)?;
        }
        Ok(())
    }
}