use std::{
    fs::File,
    io::Read,
    path::PathBuf,
    sync::LazyLock,
};
use ansi_term::Color::Yellow;
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator};
use tree_sitter_c_sharp as cs;
use crate::{
    Asset,
    AssetType,
    Id,
    parser::ParseError,
    Relation,
};

static CS_LANG: LazyLock<Language> = LazyLock::new(|| {
    cs::LANGUAGE.into()
});

/// Query to find class, struct, enum, and interface declarations.
/// Syntax tree identifiers come from https://github.com/tree-sitter/tree-sitter-c-sharp/blob/master/src/node-types.json
static CSOBJ_QUERY: LazyLock<Query> = LazyLock::new(|| {
    Query::new(&CS_LANG, r#"
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

static LOCSTR_QUERY: LazyLock<Query> = LazyLock::new(|| {
    match Query::new(&CS_LANG, r#"
(invocation_expression
    function: (member_access_expression
        expression: (
            (identifier) @obj-name
            (#eq? @obj-name "LocStringCache")
        )
        name: (
            (identifier) @fn-name
            (#eq? @fn-name "Get")
        )
    )
    arguments: (argument_list
        [
            (argument
                .
                (string_literal) @loc-str
            )
            (argument
                ((identifier) @arg-name (#eq? @arg-name "key"))
                .
                (string_literal) @loc-str
            )
        ]
    )
)"#) {
        Ok(q) => q,
        Err(e) => {
            eprintln!("Failed to compile locstring query: {e}");
            panic!();
        },
    }
});

pub fn parse(asset: &mut Asset, relative_to: Option<&PathBuf>) -> Result<Vec<Asset>, ParseError> {
    let path = match relative_to {
        Some(rel) => &rel.join(asset.path.as_ref().unwrap()),
        None => asset.path.as_ref().unwrap(),
    };

    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(e) => return Err(ParseError {
            path: path.clone(),
            message: format!("Failed to read C# file: {}", e),
        }),
    };

    let len = match file.metadata() {
        Ok(meta) => meta.len() as usize,
        Err(e) => return Err(ParseError {
            path: path.clone(),
            message: format!("Failed to read C# file metadata: {}", e),
        }),
    };

    let mut buf = Vec::with_capacity(len);
    if file.read_to_end(&mut buf).is_err() {
        return Err(ParseError {
            path: path.clone(),
            message: "Failed to read C# file".into(),
        });
    }

    parse_buffer(&buf, asset, &path.clone())
}

fn parse_buffer(
    buffer: &[u8], 
    asset: &mut Asset, 
    path: &PathBuf,
) -> Result<Vec<Asset>, ParseError> {
    let mut def_assets = vec![];
    
    // load syntax tree
    let mut parser = Parser::new();
    parser.set_language(&CS_LANG).expect("Error loading C# grammar");
    let tree = parser.parse(buffer, None);
    let tree = match tree {
        Some(t) => t,
        None => return Err(ParseError {
            path: path.clone(),
            message: "Failed to parse C# file".into(),
        }),
    };

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

        // combine namespace and type name to get FQN
        let fqn = if let Some(ns) = namespace {
            format!("{ns}.{type_name}")
        } else {
            type_name.to_string()
        };

        // create a new asset for this type
        let mut def = Asset {
            id: Id::CsType(fqn),
            path: None,
            asset_type: AssetType::CsType,
            ..Default::default()
        };
        def.relations.insert(Relation::ContainedBy(asset.id.clone()));

        def_assets.push(def);
    }

    // loop over all locstring cache gets
    let mut q = QueryCursor::new();
    let mut iter = q.matches(&LOCSTR_QUERY, tree.root_node(), buffer);

    while let Some(m) = iter.next() {
        let literal_match = m.captures.iter()
            .find(|c|
                c.index == LOCSTR_QUERY.capture_index_for_name("loc-str").unwrap()
            );
        let node = literal_match.unwrap().node;

        if node.kind() == "string_literal" {
            // trim open/close quotes
            let text = match std::str::from_utf8(&buffer[node.start_byte()+1..node.end_byte()-1]) {
                Ok(t) => t,
                Err(_) => {
                    eprintln!("\nFailed to read UTF-8 from {}", path.display());
                    continue;
                },
            };
            asset.relations.insert(Relation::Uses(Id::Loc(text.into())));
        }
        else {
            let pos = node.start_position();
            let text = match std::str::from_utf8(&buffer[node.start_byte()..node.end_byte()]) {
                Ok(t) => t,
                Err(_) => {
                    eprintln!("\nFailed to read UTF-8 from {}", path.display());
                    continue;
                },
            };
            eprintln!("\n{}: Failed to index non-literal localized string key '{text}' ({}) ({}, line {} col {})",
                Yellow.paint("Warning"),
                node.kind(),
                path.display(),
                pos.row + 1,
                pos.column + 1);
            continue;
        }
    }

    Ok(def_assets)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_parse_csharp() -> Result<(), ParseError> {
        let code = r#"
using System;
namespace MyNamespace {
    public class MyClass {
        private static LocalizedString locstringNormal = LocStringCache.Get("NormalKey");

        private static LocalizedString locstringPrefixed = LocStringCache.Get(
            key: "PrefixedKey",
            formatArgs: "Some other text");

        private static LocalizedString locstringBad = LocStringCache.Get(someKey);

        private static LocalizedString locstringBadPrefix = LocStringCache.Get(key: someKey);

        public int MyProperty { get; set; }
    }

    struct MyStruct {
        public int X;
        public int Y;
    }

    enum MyEnum {
        First,
        Second,
        Third
    }

    interface IMyInterface {
        void DoSomething();
    }
}"#;
        let mut asset = Asset {
            asset_type: AssetType::CsFile,
            ..Default::default()
        };
        let more_assets = parse_buffer(code.as_bytes(), &mut asset, &"no_path".into())?;

        assert_eq!(asset.relations, HashSet::from([
            Relation::Uses(Id::Loc("NormalKey".into())),
            Relation::Uses(Id::Loc("PrefixedKey".into()))
        ]));

        assert_eq!(more_assets.into_iter().map(|a| a.id).collect::<Vec<Id>>(), vec![
            Id::CsType("MyNamespace.MyClass".into()),
            Id::CsType("MyNamespace.MyStruct".into()),
            Id::CsType("MyNamespace.MyEnum".into()),
            Id::CsType("MyNamespace.IMyInterface".into()),
        ]);

        Ok(())
    }
}