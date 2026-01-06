use std::collections::HashSet;
use crate::{Database, Id, Relation, parser::QualifiedName};

/// A reference to a type within the file being parsed. May be locally declared, fully qualified, or ambiguous and need brokering.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TypeRequest {
    /// The asset that uses the type
    requester: Id,
    /// The un- or partially-qualified name of the type being requested from the broker
    partial_name: QualifiedName,
    /// The namespaces in scope during the reference
    scoped_namespaces: Vec<QualifiedName>,
}

impl TypeRequest {
    pub fn new(requester: &Id, name: QualifiedName, scoped_namespaces: &[QualifiedName]) -> Self {
        Self {
            requester: requester.clone(),
            partial_name: name.into(),
            scoped_namespaces: scoped_namespaces.to_vec(),
        }
    }

    /// Determines if the given type ID satisfies this type request.
    pub fn satisfied_by(&self, type_id: &Id) -> bool {
        if let Id::CsType(fqn) = type_id {
            if fqn.ends_with(&self.partial_name) {
                let mut fqn = fqn.clone();
                fqn.trim_end(&self.partial_name);
                self.scoped_namespaces.contains(&fqn)
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl Into<Id> for TypeRequest {
    fn into(self) -> Id {
        Id::CsType(self.partial_name)
    }
}

/// A broker for managing type references during parsing. Tracks which types are declared and which are referenced.
pub struct TypeBroker {
    requests: HashSet<TypeRequest>,
}

impl TypeBroker {
    pub fn new() -> Self {
        Self {
            requests: HashSet::new(),
        }
    }

    pub fn request(&mut self, requester: &Id, type_name: QualifiedName, scoped_namespaces: &[QualifiedName]) {
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