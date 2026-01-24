use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex, LazyLock},
};
use tree_sitter::{
    Node,
    Query,
    QueryCursor, 
    StreamingIterator, 
    Tree,
};
use crate::{
    Asset, AssetType, Id, Relation, parser::{ParseError, QualifiedName, TypeBroker}
};

use super::{
    structure::*,
};

/// Find type declarations and usages in the given syntax tree, updating the provided asset and type broker accordingly.
pub fn find_types(
    tree: &Tree, 
    buffer: &[u8], 
    asset: &mut Asset, 
    def_assets: &mut Vec<Asset>, 
    broker: &Arc<Mutex<TypeBroker>>,
) -> Result<(), ParseError> {
    let structure = evaluate_structure(tree, buffer)
        .map_err(|e| ParseError {
            path: asset.path.as_ref().unwrap().clone(),
            message: "Failed to analyze structure of C# file".to_string(),
            inner: Some(Box::new(e)),
        })?;
    Ok(())
}
