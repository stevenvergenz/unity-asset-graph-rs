mod full;
mod partial;
pub use full::{FullyQualifiedName, /*FullyQualifiedNameRef*/};
pub use partial::{PartiallyQualifiedName, /* PartiallyQualifiedNameRef */};

use std::{
    fmt::{Display, Formatter, Result},
    borrow::Borrow,
    hash::Hash,
};
use serde::{Deserialize, Serialize};

/// A C# qualified name, represented as parts in reverse order (e.g. ["MyClass", "MyNamespace"])
pub trait QualifiedName {
    fn parts(&self) -> &[String];
    fn is_fully_qualified(&self) -> bool;

    fn trim_end(&self, other: &impl Self) -> impl Self;
}


impl PartialEq for dyn QualifiedName { 
    fn eq(&self, other: &Self) -> bool {
        self.is_fully_qualified() == other.is_fully_qualified() && self.parts() == other.parts()
    }
}

impl Eq for dyn QualifiedName {}

impl Display for dyn QualifiedName {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut iter = self.parts().iter();
        if let Some(p) = iter.next() {
            write!(f, "{p}")?;
        }
        for p in iter {
            write!(f, ".{p}")?;
        }
        Ok(())
    }
}

/*
impl QualifiedName {
    pub fn new(parts: Vec<String>) -> Self {
        if parts.is_empty() {
            panic!("QualifiedName must have at least one part");
        }
        Self::Partial(parts)
    }

    pub fn from_iter(parts: impl Iterator<Item = impl Into<String>>) -> Self {
        Self::new(parts.map(|s| s.into()).collect())
    }

    pub fn from_name(name: impl Into<String>, namespace: Self) -> Self {
        match namespace {
            Self::Partial(mut p) => {
                p.insert(0, name.into());
                Self::Partial(p)
            },
            Self::Full(mut p) => {
                p.insert(0, name.into());
                Self::Full(p)
            },
        }
    }

    pub fn resolve(self) -> Self {
        let parts = match self {
            Self::Partial(p) | Self::Full(p) => p,
        };
        Self::Full(parts)
    }

    pub fn concat(narrow: &Self, broad: &Self) -> Self {
        let new = Self::from_iter(narrow.iter().chain(broad.iter()));
        if let Self::Full(_) = broad {
            new.resolve()
        } else {
            new
        }
    }

    /// Whether another name is within this namespace
    pub fn contains(&self, other: &Self) -> bool {
        self.iter().rev().eq(other.iter().rev().take(self.len()))
    }

    /// The containing type or namespace of this name
    pub fn container(&self) -> Self {
        match self {
            Self::Partial(p) => Self::Partial(p.iter().skip(1).cloned().collect()),
            Self::Full(p) => Self::Full(p.iter().skip(1).cloned().collect()),
        }
    }

    /// Produce a less qualified name by removing the given namespace. Returns None if this name is not within the namespace.
    pub fn without_namespace(&self, ns: &Self) -> Option<Self> {
        let mut parts = match self {
            Self::Partial(p) | Self::Full(p) => p.clone(),
        };
        for ns_part in ns.iter().rev() {
            if parts.last() == Some(ns_part) {
                parts.pop();
            } else {
                return None;
            }
        }
        Some(Self::Partial(parts))
    }

    /// Produce a namespace by removing the given local name. Returns None if the local name is not within this namespace.
    pub fn without_local(&self, local: &Self) -> Option<Self> {
        if !self.iter().take(local.len()).eq(local.iter()) {
            None
        } else {
            let parts = self.iter().skip(local.len()).cloned().collect();
            match self {
                Self::Partial(_) => Some(Self::Partial(parts)),
                Self::Full(_) => Some(Self::Full(parts)),
            }
        }
    }

    pub fn local(&self) -> Self {
        Self::Partial(self.iter().take(1).cloned().collect())
    }

    pub fn namespace(&self) -> Self {
        match self {
            Self::Partial(p) => Self::Partial(p.iter().skip(1).cloned().collect()),
            Self::Full(p) => Self::Full(p.iter().skip(1).cloned().collect()),
        }
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &String> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Partial(p) | Self::Full(p) => p.len()
        }
    }
}
*/
