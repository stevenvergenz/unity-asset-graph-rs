use serde::{Deserialize, Serialize};
use super::QualifiedName;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartiallyQualifiedName {
    parts: Vec<String>,
}

impl PartiallyQualifiedName {
    pub fn new(parts: Vec<String>) -> Self {
       Self { parts }
    }
}

impl QualifiedName for PartiallyQualifiedName {
    fn is_fully_qualified(&self) -> bool {
        false
    }
    fn parts(&self) -> &[String] {
        &self.parts[..]
    }
}

impl<'a> FromIterator<&'a str> for PartiallyQualifiedName {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self::new(iter.into_iter().map(|s| String::from(s)).collect())
    }
}

impl From<&str> for PartiallyQualifiedName {
    fn from(value: &str) -> Self {
        Self::from_iter(value.split('.'))
    }
}

