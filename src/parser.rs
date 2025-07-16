use std::path::PathBuf;
use crate::{
    asset::{Asset, AssetType},
};

pub mod manifest_json;
pub mod package_json;
mod unity;
mod loc_text;
mod loc_resource;
mod loc_override;
mod csharp;

#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}

pub fn parse(asset: &mut Asset, relative_to: Option<&PathBuf>) -> Result<Vec<Asset>, ParseError> {
    let path = match asset.path {
        Some(ref p) => p,
        None => return Ok(vec![]),
    };

    match path.extension().and_then(|s| s.to_str()) {
        Some("prefab") | Some("unity") | Some("scene") | Some("asset") => {
            unity::parse_unity(asset, relative_to)
        },
        Some("cs") => {
            csharp::parse_csharp(asset, relative_to)
        },
        _ => {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap();
            if name.ends_with("Resource.en-us.json") {
                asset.asset_type = AssetType::LocResource;
                loc_resource::parse_loc_resource(asset, relative_to)
            }
            else {
                Ok(vec![]) // Not a known file type
            }
        }
    }
}
