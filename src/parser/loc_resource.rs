use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    io::BufRead,
};
use crate::{
    asset::Asset,
    id::Id,
    parser::ParseError,
    util,
};

pub fn parse_loc_resource(asset: &mut Asset, relative_to: Option<&PathBuf>) -> Result<Vec<Asset>, ParseError> {
    let path = match relative_to {
        Some(rel) => &rel.join(asset.path.as_ref().unwrap()),
        None => asset.path.as_ref().unwrap(),
    };

    let mut reader = match util::read_file_no_bom(path) {
        Ok(file) => file,
        Err(e) => return Err(ParseError {
            message: format!("Failed to read prefab file: {}", e),
        }),
    };

    parse_loc_resource_reader(&mut reader, asset)
}

fn parse_loc_resource_reader(reader: &mut dyn BufRead, asset: &mut Asset) -> Result<Vec<Asset>, ParseError> {
    let locstrings: HashMap<String, String> = match serde_json::from_reader(reader) {
        Ok(map) => map,
        Err(e) => return Err(ParseError {
            message: format!("Failed to parse loc resource JSON: {}", e),
        }),
    };

    // Use the parsed locstrings to create Asset instances
    let mut assets = vec![];
    for key in locstrings.keys() {
        let asset = Asset {
            id: Id::Loc(key.clone()),
            dependencies: HashSet::from([asset.id.clone()]),
            ..Default::default()
        };
        assets.push(asset);
    }

    Ok(assets)
}