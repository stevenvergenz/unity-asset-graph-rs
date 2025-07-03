use std::{
    path::PathBuf,
    collections::HashSet,
};
use crate::{
    asset::Asset, database::Database, id::Id
};

pub mod manifest_json;
pub mod package_json;
pub mod unity;
pub mod localized_text;

#[derive(Debug)]
pub struct ParseError {
    message: String,
    inner: Option<Box<dyn std::error::Error>>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(inner) = &self.inner {
            write!(f, "{}: {}", self.message, inner)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ParseError {}

pub trait Parser {
    fn parse(asset: &mut Asset) -> Result<(), ParseError>;
}