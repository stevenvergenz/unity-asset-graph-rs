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
    Asset, AssetType, Id, Relation, parser::{ParseError, TypeBroker}
};

use super::{
    qualified_name::{QualifiedNameOwned, QualifiedNameRef},
    structure::*,
    queries::kinds as k,
};

/// Find type declarations and usages in the given syntax tree, updating the provided asset and type broker accordingly.
pub fn find_types(
    tree: &Tree, 
    buffer: &[u8], 
    asset: &mut Asset, 
    def_assets: &mut Vec<Asset>, 
    broker: &Arc<Mutex<TypeBroker>>,
) -> Result<(), ParseError> {
    let info = evaluate_structure(tree, buffer)
        .map_err(|e| ParseError {
            path: asset.path.as_ref().unwrap().clone(),
            message: "Failed to analyze structure of C# file".to_string(),
            inner: Some(Box::new(e)),
        })?;

    // identify non-nested types, create assets for them

    'decls: for (node, name) in &info.type_decl_nodes {
        // walk up the node tree from the type decl
        let mut full_name = name.clone();
        while let Some(container) = node.parent() {
            // if we find a namespace declaration, add it to the fully-qualified type name
            if let Some(ns) = info.ns_decl_nodes.get(&container) {
                full_name = QualifiedNameRef::try_concat(ns.clone(), full_name)
                    .map_err(|e| ParseError {
                        path: asset.path.as_ref().unwrap().clone(),
                        message: "Failed to join qualified names".to_string(),
                        inner: Some(Box::new(e)),
                    })?;
            }

            // do not record assets for nested types
            else if let Some(_) = info.type_decl_nodes.get(&container) {
                continue 'decls;
            }
        }

        let type_asset = Asset {
            id: Id::CsType(full_name.to_owned()),
            asset_type: AssetType::CsType,
            path: None,
            ..Default::default()
        };
        def_assets.push(type_asset);
    }

    Ok(())
}
