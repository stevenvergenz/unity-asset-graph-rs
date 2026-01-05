use serde::{Deserialize, Serialize};
use super::QualifiedName;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FullyQualifiedName {
    parts: Vec<String>,
}

impl FullyQualifiedName {
    pub fn new(parts: Vec<String>) -> Self {
       Self { parts }
    }
}

impl QualifiedName for FullyQualifiedName {
    fn is_fully_qualified(&self) -> bool {
        true
    }
    fn parts(&self) -> &[String] {
        &self.parts[..]
    }
}

impl<'a> FromIterator<&'a str> for FullyQualifiedName {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::new(iter.into_iter().map(|s| String::from(s)).collect())
    }
}

impl From<&str> for FullyQualifiedName {
    fn from(value: &str) -> Self {
        Self::from_iter(value.split('.'))
    }
}

