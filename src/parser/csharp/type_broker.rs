use std::collections::HashSet;
use crate::{Database, Id, Relation};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TypeRequest {
    requester: Id,
    type_name: String,
    scoped_namespaces: Vec<String>,
}

impl TypeRequest {
    pub fn new(requester: &Id, type_name: &str, scoped_namespaces: &[&str]) -> Self {
        Self {
            requester: requester.clone(),
            type_name: type_name.into(),
            scoped_namespaces: scoped_namespaces.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn satisfied_by(&self, type_id: &Id) -> bool {
        if let Id::CsType { name, namespace } = type_id {
            if &self.type_name != name {
                return false;
            }
            if let Some(ns) = namespace {
                if self.scoped_namespaces.iter().any(|s| s == ns) {
                    return true;
                }
            } else if self.scoped_namespaces.is_empty() {
                return true;
            }
        }
        false
    }
}

pub struct TypeBroker {
    requests: HashSet<TypeRequest>,
}

impl TypeBroker {
    pub fn new() -> Self {
        Self {
            requests: HashSet::new(),
        }
    }

    pub fn request(&mut self, requester: &Id, type_name: &str, scoped_namespaces: &[&str]) {
        self.requests.insert(TypeRequest::new(requester, type_name, scoped_namespaces));
    }

    pub fn fulfill(&mut self, id: &Id, database: &mut Database) {
        for req in self.requests.extract_if(|req| req.satisfied_by(id)) {
            if let Some(asset) = database.asset_mut(&req.requester) {
                asset.relations.insert(Relation::Uses(id.clone()));
            }
        }
    }

    pub fn requests(&self) -> &HashSet<TypeRequest> {
        &self.requests
    }
}