use std::sync::LazyLock;
use tree_sitter::{Query, QueryCursor, StreamingIterator, Tree};
use crate::{Asset, AssetType, Id, parser::ParseError, Relation};

/// Query to find class, struct, enum, and interface declarations.
/// Syntax tree identifiers come from https://github.com/tree-sitter/tree-sitter-c-sharp/blob/master/src/node-types.json
static CSOBJ_QUERY: LazyLock<Query> = LazyLock::new(|| {
    Query::new(&super::CS_LANG, r#"
[(class_declaration
    name: (identifier) @name
)
(struct_declaration
    name: (identifier) @name
)
(enum_declaration
    name: (identifier) @name
)
(interface_declaration
    name: (identifier) @name
)]"#)
        .expect("Failed to compile class query")
});

pub fn find_types(tree: &Tree, buffer: &[u8], asset: &mut Asset, def_assets: &mut Vec<Asset>) -> Result<(), ParseError> {
    // loop over all type declarations
    let mut q = QueryCursor::new();
    let mut iter = q.matches(&CSOBJ_QUERY, tree.root_node(), buffer);
    while let Some(m) = iter.next() {
        // get the name of the type
        let node = m.captures[0].node;
        let text = &buffer[node.start_byte()..node.end_byte()];
        let type_name = match std::str::from_utf8(text) {
            Ok(name) => name,
            Err(_) => continue,
        };

        // find the namespace, if any
        let mut parent = node.parent();
        let mut namespace = None;
        while let Some(p) = parent {
            if p.kind() == "namespace_declaration" {
                if let Some(name_node) = p.child_by_field_name("name") {
                    let name_text = &buffer[name_node.start_byte()..name_node.end_byte()];
                    match std::str::from_utf8(name_text) {
                        Ok(ns) => {
                            namespace = Some(ns);
                            break;
                        },
                        Err(_) => break,
                    }
                } else {
                    break;
                }
            }
            parent = p.parent();
        }

        // create a new asset for this type
        let mut def = Asset {
            id: Id::CsType { name: type_name.into(), namespace: namespace.map(|s| s.into()) },
            path: None,
            asset_type: AssetType::CsType,
            ..Default::default()
        };
        def.relations.insert(Relation::ContainedBy(asset.id.clone()));

        def_assets.push(def);
    }
    Ok(())
}