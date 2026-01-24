use serde::{Deserialize, Serialize};
use tree_sitter::{Node};
use std::fmt::{Display, Formatter, Result as FResult};
use crate::parser::csharp::qualified_name::generic_args_count_from_str;

use super::{Error, GENERIC_NAMES};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NamePartRef<'a> {
    name: &'a str,
    generics: usize,
}

impl<'a> Display for NamePartRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "{}", self.name)?;
        if self.generics > 0 {
            write!(f, "{}", GENERIC_NAMES[self.generics - 1])?;
        }
        Ok(())
    }
}

impl<'a> From<&'a str> for NamePartRef<'a> {
    fn from(value: &'a str) -> Self {
        if let Some(open_index) = value.find('<') {
            let (n, g) = value.split_at(open_index);
            println!("Generic split parts: '{n}' and '{g}'");
            Self { name: n, generics: generic_args_count_from_str(g) }
        } else {
            Self { name: value, generics: 0 }
        }
    }
}

impl<'a> PartialEq<str> for NamePartRef<'a> {
    fn eq(&self, other: &str) -> bool {
        if other.len() < self.name.len() {
            return false;
        }

        let (n, g) = other.split_at(self.name.len());
        n == self.name && generic_args_count_from_str(g) == self.generics
    }
}

/// A C# qualified name, represented as parts in order (e.g. ["MyNamespace", "MyClass"])
#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QualifiedNameRef<'a> {
    parts: Vec<NamePartRef<'a>>,
    alias: Option<&'a str>,
}

impl<'a> QualifiedNameRef<'a> {
    pub fn try_concat(start: Self, end: Self) -> Result<Self, Error<'a>> {
        if let Some(alias) = end.alias {
            return Err(Error::BadAlias(alias));
        }

        Ok(Self {
            alias: start.alias,
            parts: start.parts.into_iter().chain(end.parts.into_iter()).collect(),
        })
    }

    pub fn try_from<'t, 'b>(node: Node<'t>, buffer: &'b [u8]) -> Result<Self, Error<'b>>
        where 'b: 'a {
        let mut name = Self { ..Default::default() };
        try_from(node, buffer, &mut name)?;
        Ok(name)
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn push(&mut self, part: &'a str) {
        self.parts.push(part.into());
    }

    pub fn pop(&mut self) -> Option<&'a str> {
        self.parts.pop().map(|p| p.name)
    }

    /// Splits the name into two at the given index. [0, index) is left here, [index, len) is in the returned name
    pub fn split_off(&mut self, index: usize) -> Self {
        Self { parts: self.parts.split_off(index), ..Default::default() }
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item=&NamePartRef<'a>> {
        self.parts.iter()
    }
}

impl<'a> FromIterator<NamePartRef<'a>> for QualifiedNameRef<'a> {
    fn from_iter<T: IntoIterator<Item = NamePartRef<'a>>>(iter: T) -> Self {
        Self { parts: iter.into_iter().collect(), ..Default::default() }
    }
}

impl<'a> FromIterator<&'a str> for QualifiedNameRef<'a> {
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        Self { parts: iter.into_iter().map(|s| NamePartRef::from(s)).collect(), ..Default::default() }
    }
}

impl<'a> IntoIterator for QualifiedNameRef<'a> {
    type IntoIter = std::vec::IntoIter<NamePartRef<'a>>;
    type Item = NamePartRef<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.parts.into_iter()
    }
}

impl<'a> From<&[&'a str]> for QualifiedNameRef<'a> {
    fn from(value: &[&'a str]) -> Self {
        Self::from_iter(value.into_iter().map(|s| *s))
    }
}

impl<'a> From<&'a str> for QualifiedNameRef<'a> {
    fn from(value: &'a str) -> Self {
        if let Some((alias, rest)) = value.split_once("::")
        && !alias.contains(|c: char| !c.is_alphanumeric()) {
            let mut new = Self::from_iter(rest.split('.'));
            new.alias = Some(alias);
            new
        } else {
            Self::from_iter(value.split('.'))
        }
    }
}

impl<'a> PartialEq<str> for QualifiedNameRef<'a> {
    fn eq(&self, other: &str) -> bool {
        let other = QualifiedNameRef::from(other);
        self == &other
    }
}

impl<'a> Display for QualifiedNameRef<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        if let Some(alias) = self.alias {
            write!(f, "{alias}::")?;
        }
        let mut iter = self.parts.iter();
        if let Some(p) = iter.next() {
            write!(f, "{}", p)?;
        }
        for part in iter {
            write!(f, ".{}", part)?;
        }
        Ok(())
    }
}

/// Extract a qualified name recursively from the source tree. Note: outputs parts in reverse order.
fn try_from<'t, 'b, 'n>(node: Node<'t>, buffer: &'b [u8], output: &mut QualifiedNameRef<'n>) -> Result<(), Error<'b>>
where 'b: 'n {
    match node.kind() {
        "identifier" => {
            let name = node.utf8_text(buffer)
                .map_err(|e| Error::Utf8(e))?;
            output.parts.push(NamePartRef { name, generics: 0 });
            Ok(())
        },
        "generic_name" => {
            // children: identifier, type_argument_list
            let mut name = NamePartRef { name: "", generics: 0 };
            let mut cursor = node.walk();
            for c in node.named_children(&mut cursor) {
                match c.kind() {
                    "identifier" => {
                        name.name = c.utf8_text(buffer).map_err(|e| Error::Utf8(e))?;
                    },
                    "type_argument_list" => {
                        name.generics = c.named_child_count();
                    },
                    _ => return Err(Error::BadGeneric(node.utf8_text(buffer).map_err(|e| Error::Utf8(e))?)),
                }
            }
            output.parts.push(name);
            Ok(())
        },
        "qualified_name" => {
            let (name, qualifier) = match (node.child_by_field_name("name"), node.child_by_field_name("qualifier")) {
                (Some(n), Some(q)) => (n, q),
                _ => return Err(Error::BadQualified(node.utf8_text(buffer).map_err(|e| Error::Utf8(e))?)),
            };
            try_from(qualifier, buffer, output)?;
            try_from(name, buffer, output)?;
            Ok(())
        },
        "alias_qualified_name" => {
            let (alias, name) = match (node.child_by_field_name("alias"), node.child_by_field_name("name")) {
                (Some(a), Some(n)) => (a, n),
                _ => return Err(Error::BadQualified(node.utf8_text(buffer).map_err(|e| Error::Utf8(e))?)),
            };
            output.alias = Some(alias.utf8_text(buffer).map_err(|e| Error::Utf8(e))?);
            try_from(name, buffer, output)
        },
        _ => Err(Error::BadKind(node.kind())),
    }
}