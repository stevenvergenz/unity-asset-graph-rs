use std::{
    io::BufRead,
    path::PathBuf,
    sync::LazyLock,
};
use regex::Regex;
use uuid::Uuid;
use crate::{
    asset::Asset,
    id::Id,
    parser::{
        localized_text::LocStringParser,
        ParseError,
    },
};

static ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b([0-9a-f]{32})\b").expect("Failed to compile ID regex")
});

pub fn parse_unity(asset: &mut Asset, relative_to: Option<&PathBuf>) -> Result<Vec<Asset>, ParseError> {
    let path = match relative_to {
        Some(rel) => &rel.join(&asset.path),
        None => &asset.path,
    };

    let mut reader = match crate::util::read_file_no_bom(path) {
        Ok(file) => file,
        Err(e) => return Err(ParseError {
            message: format!("Failed to read prefab file: {}", e),
            inner: None,
        }),
    };

    let mut loc_parser = LocStringParser::Start;
    let mut line = String::new();
    while let Ok(bytes) = reader.read_line(&mut line) && bytes > 0 {
        loc_parser = loc_parser.update(&line);
        if let LocStringParser::LocStringKey(id) = loc_parser {
            asset.dependencies.insert(id);
            loc_parser = LocStringParser::Start;
        }

        if let Some(captures) = ID_REGEX.captures(&line)
            && let Some(id_str) = captures.get(1)
            && let Ok(uuid) = Uuid::parse_str(id_str.as_str())
        {
            asset.dependencies.insert(Id::Guid(uuid));
        }

        line.clear();
    }

    Ok(vec![])
}
