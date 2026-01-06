use std::{
    fmt::{Display, Formatter, Result},
    hash::Hash,
};
use serde::{Deserialize, Serialize};

/// A C# qualified name, represented as parts in order (e.g. ["MyNamespace", "MyClass"])
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct QualifiedName(pub Vec<String>);

impl QualifiedName {
    pub fn new(parts: Vec<String>) -> Self {
        if parts.is_empty() {
            panic!("QualifiedName must have at least one part");
        }
        Self(parts)
    }

    pub fn starts_with(&self, other: &QualifiedName) -> bool {
        self.0[..other.0.len()] == other.0[..]
    }

    pub fn ends_with(&self, other: &QualifiedName) -> bool {
        self.0[self.0.len() - other.0.len()..] == other.0[..]
    }

    pub fn trim_start(&mut self, other: &QualifiedName) {
        if self.starts_with(other) {
            self.0 = self.0[other.0.len()..].to_vec();
        }
    }

    pub fn trim_end(&mut self, other: &QualifiedName) {
        if self.ends_with(other) {
            let new_len = self.0.len() - other.0.len();
            self.0.truncate(new_len);
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, part: String) {
        self.0.push(part);
    }

    pub fn pop(&mut self) -> Option<String> {
        self.0.pop()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.0.iter()
    }
}

impl<'a> FromIterator<&'a str> for QualifiedName {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        let parts: Vec<String> = iter.into_iter().map(|s| s.to_string()).collect();
        QualifiedName::new(parts)
    }
}

impl From<Vec<String>> for QualifiedName {
    fn from(value: Vec<String>) -> Self {
        Self::new(value)
    }
}

impl From<&[&str]> for QualifiedName {
    fn from(value: &[&str]) -> Self {
        Self::from_iter(value.iter().cloned())
    }
}

impl From<&str> for QualifiedName {
    fn from(value: &str) -> Self {
        Self::from_iter(value.split('.'))
    }
}

// todo: split in place instead of cloning slices
impl From<String> for QualifiedName {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl Display for QualifiedName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut iter = self.0.iter();
        if let Some(p) = iter.next() {
            write!(f, "{}", p)?;
        }
        for part in iter {
            write!(f, ".{}", part)?;
        }
        Ok(())
    }
}

impl PartialEq<&str> for QualifiedName {
    fn eq(&self, other: &&str) -> bool {
        other.split('.').eq(self.0.iter().map(String::as_str))
    }
}
