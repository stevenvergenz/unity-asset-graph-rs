use std::{
    io::BufRead,
    sync::LazyLock,
    path::PathBuf,
};
use regex::Regex;
use uuid::Uuid;
use crate::{
    asset::Asset,
    id::Id,
    parsers::{
        localized_text::LocStringParser,
        ParseError,
        Parser,
    },
};

static ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b([0-9a-f]{32})\b").expect("Failed to compile ID regex")
});

pub struct UnityObject;

impl Parser for UnityObject {
    fn parse(asset: &mut Asset, relative_to: Option<&PathBuf>) -> Result<(), ParseError> {
        let path = if asset.path.is_relative() && let Some(root) = relative_to {
            root.join(&asset.path)
        } else {
            asset.path.clone()
        };

        let mut reader = match crate::util::read_file_no_bom(&path) {
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
                asset.dependencies.insert(Id::new_uuid(uuid));
            }

            line.clear();
        }

        Ok(())
    }
}