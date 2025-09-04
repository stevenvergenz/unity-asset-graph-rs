use std::path::PathBuf;
use crate::{
    asset::Asset,
    asset_type::AssetType,
};

pub mod manifest_json;
pub mod package_json;
mod unity;
mod loc_text;
mod loc_resource;
mod loc_override;
mod csharp;
mod directory;

#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl ParseError {
    pub fn new(message: String) -> Self {
        Self {
            message,
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}

pub fn parse(asset: &mut Asset, relative_to: Option<&PathBuf>) -> Result<Vec<Asset>, ParseError> {
    if asset.path.is_none() {
        return Ok(vec![]);
    }

    if asset.asset_type.is_unity() {
        unity::parse(asset, relative_to)
    }
    else if let AssetType::Script = asset.asset_type {
        csharp::parse(asset, relative_to)
    }
    else if let Some(filename) = asset.path.as_ref().unwrap().file_name().and_then(|f| f.to_str())
        && filename.ends_with("Resource.en-us.json") {
        asset.asset_type = AssetType::LocResource;
        loc_resource::parse(asset, relative_to)
    }
    else if let AssetType::Directory = asset.asset_type {
        directory::parse(asset, relative_to)
    }
    else {
        Ok(vec![])
    }
}
