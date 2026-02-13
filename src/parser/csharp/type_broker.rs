use std::collections::HashSet;
use crate::{Database, Id, Relation};
use super::qualified_name::{QualifiedName, QualifiedNameOwned, QualifiedNamePart};

/// A reference to a type within the file being parsed. May be locally declared, fully qualified, or ambiguous and need brokering.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TypeRequest {
    /// The asset that uses the type
    pub requester: Id,
    /// The un- or partially-qualified name of the type being requested from the broker
    pub partial_name: QualifiedNameOwned,
    /// The namespaces in scope during the reference
    pub scoped_namespaces: Vec<QualifiedNameOwned>,
}

impl TypeRequest {
    /// Determines if the given type ID satisfies this type request.
    pub fn satisfied_by(&self, type_id: &Id) -> bool {
        if let Id::CsType(fqn) = type_id {
            if fqn.ends_with(&self.partial_name) {
                let fqn = fqn.as_ref().split_off(fqn.len() - self.partial_name.len());
                self.scoped_namespaces.iter().any(|ns| ns == &fqn)
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

    pub fn request(&mut self, requester: &Id, type_name: QualifiedNameOwned, scoped_namespaces: impl IntoIterator<Item=QualifiedNameOwned>) {
        self.requests.insert(TypeRequest {
            requester: requester.clone(),
            partial_name: type_name,
            scoped_namespaces: scoped_namespaces.into_iter().collect(),
        });
    }

    pub fn push(&mut self, request: TypeRequest) {
        self.requests.insert(request);
    }

    pub fn requests(&self) -> &HashSet<TypeRequest> {
        &self.requests
    }

    pub fn fulfill(&mut self, ids: impl IntoIterator<Item=Id>, database: &mut Database) {
        
    }
}