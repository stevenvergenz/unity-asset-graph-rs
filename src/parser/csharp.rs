use std::{
    path::PathBuf,
    io::BufRead,
    sync::LazyLock,
};
use regex::Regex;
use crate::{
    asset::Asset,
    id::Id,
    parser::ParseError,
};

static LOC_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"LocStringCache.Get\("([^"]+)""#).expect("Failed to compile locString regex")
});

pub fn parse(asset: &mut Asset, relative_to: Option<&PathBuf>) -> Result<Vec<Asset>, ParseError> {
    let path = match relative_to {
        Some(rel) => &rel.join(asset.path.as_ref().unwrap()),
        None => asset.path.as_ref().unwrap(),
    };

    let mut reader = match crate::util::read_file_no_bom(path) {
        Ok(file) => file,
        Err(e) => return Err(ParseError::new(path, format!("Failed to read prefab file: {}", e))),
    };

    parse_reader(&mut reader, asset, relative_to)
}

fn parse_reader(
    reader: &mut dyn BufRead, 
    asset: &mut Asset, 
    relative_to: Option<&PathBuf>,
) -> Result<Vec<Asset>, ParseError> {
    let path = match relative_to {
        Some(rel) => &rel.join(asset.path.as_ref().unwrap()),
        None => asset.path.as_ref().unwrap(),
    };
    
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => return Err(ParseError::new(path, format!("Failed to read line: {}", e))),
        };

        for capture in LOC_REGEX.captures_iter(&line) {
            if let Some(loc) = capture.get(1) {
                let loc_id = loc.as_str().to_string();
                asset.dependencies.insert(Id::Loc(loc_id.clone()));
            }
        }
    }

    Ok(vec![])
}