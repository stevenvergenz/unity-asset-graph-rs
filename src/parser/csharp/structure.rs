use std::{
    collections::{HashMap, HashSet}, fmt::{Display, Formatter, Result as FResult}, str::Utf8Error, sync::LazyLock
};
use tree_sitter::{Tree, Query, QueryCursor, QueryError, QueryMatch, Node, StreamingIterator};
use crate::parser::csharp::qualified_name::{self, QualifiedNameRef};

use super::{
    CS_LANG,
    queries::QUERY_ALL,
};

#[derive(Debug)]
pub enum Error<'a> {
    Query(QueryError),
    FieldName(&'a str),
    FieldId(u32),
    Utf8(Utf8Error),
    BadStaticUsing(&'a str),
    BadName(qualified_name::Error<'a>),
    Unknown(&'a str),
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        match self {
            Self::Query(q) => write!(f, "{q}"),
            Self::FieldName(e) => write!(f, "No such field '{e}'"),
            Self::FieldId(id) => write!(f, "No such field {id}"),
            Self::Utf8(e) => write!(f, "Failed to convert buffer to UTF-8: {e}"),
            Self::BadStaticUsing(s) => write!(f, "Failed to parse static using '{s}'"),
            Self::BadName(e) => write!(f, "Failed to parse name: {e}"),
            Self::Unknown(s) => write!(f, "Unknown error: {s}"),
        }
    }
}

impl<'a> std::error::Error for Error<'a> {}

static QUERY: LazyLock<Query> = LazyLock::new(|| {
    Query::new(&CS_LANG, QUERY_ALL).expect("Failed to compile query")
});

static F_NS_DECL: LazyLock<u32> = LazyLock::new(|| QUERY.capture_index_for_name("ns_decl").expect("Failed to get field ns_decl"));
static F_NS_USE: LazyLock<u32> = LazyLock::new(|| QUERY.capture_index_for_name("ns_use").expect("Failed to get field ns_use"));
static F_TYPE_DECL: LazyLock<u32> = LazyLock::new(|| QUERY.capture_index_for_name("type_decl").expect("Failed to get field type_decl"));
static F_TYPE_USE: LazyLock<u32> = LazyLock::new(|| QUERY.capture_index_for_name("type_use").expect("Failed to get field type_use"));
static F_VAR_DECL: LazyLock<u32> = LazyLock::new(|| QUERY.capture_index_for_name("var_decl").expect("Failed to get field var_decl"));
static F_VAR_USE: LazyLock<u32> = LazyLock::new(|| QUERY.capture_index_for_name("var_use").expect("Failed to get field var_use"));
static F_ID: LazyLock<u32> = LazyLock::new(|| QUERY.capture_index_for_name("id").expect("Failed to get field id"));
static F_ALIAS: LazyLock<u32> = LazyLock::new(|| QUERY.capture_index_for_name("alias").expect("Failed to get field alias"));

static K_USING: LazyLock<u16> = LazyLock::new(|| CS_LANG.id_for_node_kind("using_directive", true));
static K_STATIC: LazyLock<u16> = LazyLock::new(|| CS_LANG.id_for_node_kind("static", false));

#[derive(Default)]
pub struct StructureInfo<'buffer, 'tree> {
    /// A map of scope nodes to alias/original names
    pub aliases: HashMap<Node<'tree>, HashMap<QualifiedNameRef<'buffer>, QualifiedNameRef<'buffer>>>,

    /// A map of scope nodes to declared namespace names
    pub ns_decl: HashMap<Node<'tree>, HashSet<QualifiedNameRef<'buffer>>>,

    /// A map of scope nodes to used namespace names
    pub ns_usages: HashMap<Node<'tree>, HashSet<QualifiedNameRef<'buffer>>>,

    /// A map of scope nodes to declared type names
    pub type_decl: HashMap<Node<'tree>, HashSet<QualifiedNameRef<'buffer>>>,

    /// A map of usage nodes to the used type name
    pub type_usages: HashMap<Node<'tree>, QualifiedNameRef<'buffer>>,

    /// A map of scope nodes to declared variable names
    pub var_decl: HashMap<Node<'tree>, HashSet<QualifiedNameRef<'buffer>>>,

    /// A map of usage nodes to the used variable name
    pub var_usages: HashMap<Node<'tree>, QualifiedNameRef<'buffer>>,
}

impl<'buffer, 'tree> StructureInfo<'buffer, 'tree> {
    pub fn resolve_type_decl_name(&self, id_node: Node<'tree>) -> QualifiedNameRef<'buffer> {
        todo!();

        let ns_kind = CS_LANG.id_for_node_kind("namespace_declaration", true);
        let fsns_kind = CS_LANG.id_for_node_kind("file_scoped_namespace_declaration", true);
        // // find full namespace of the declared type

        // // walk up ancestor nodes, prepending any namespace declarations we come across
        // let mut i = scope_node;
        // while let Some(ancestor) = i.parent() {
        //     if ancestor.kind_id() == ns_kind
        //     && let Some(ns) = ancestor.child_by_field_name("name") {
        //         let ns = QualifiedNameRef::try_from(ns, buffer).map_err(|e| Error::BadName(e))?;
        //         name = QualifiedNameRef::concat(ns, name);
        //     }

        //     i = ancestor;
        // }

        // // if there is a file-scoped namespace declaration, add it as well
        // let root = i;
        // let mut cursor = root.walk();
        // if let Some(fsns) = root.named_children(&mut cursor)
        //     .filter(|c| c.kind_id() == fsns_kind)
        //     .next()
        // && let Some(ns) = fsns.child_by_field_name("name")
        // && let Ok(ns) = QualifiedNameRef::try_from(ns, buffer) {
        //     name = QualifiedNameRef::concat(ns, name);
        // }

    }
}

pub fn evaluate_structure<'t, 'b>(tree: &'t Tree, buffer: &'b [u8]) -> Result<StructureInfo<'b, 't>, Error<'b>> {
    let mut results = StructureInfo { ..Default::default() };
    let mut cursor = QueryCursor::new();
    let mut iter = cursor.matches(&QUERY, tree.root_node(), buffer);

    while let Some(m) = iter.next() {
        for c in m.captures {
            if c.index == *F_NS_DECL {
                evaluate_ns_decl(c.node, m, buffer, &mut results)?;
            } else if c.index == *F_NS_USE {
                evaluate_ns_usage(c.node, m, buffer, &mut results)?;
            } else if c.index == *F_TYPE_DECL {
                evaluate_type_decl(c.node, m, buffer, &mut results)?;
            } else if c.index == *F_TYPE_USE {
                evaluate_type_usage(c.node, m, buffer, &mut results)?;
            } else if c.index == *F_VAR_DECL {
                evaluate_var_decl(c.node, m, buffer, &mut results)?;
            } else if c.index == *F_VAR_USE {
                evaluate_var_usage(c.node, m, buffer, &mut results)?;
            } else if c.index != *F_ID && c.index != *F_ALIAS {
                return Err(Error::FieldId(c.index));
            }
        }
    }

    Ok(results)
}

fn get_root(node: Node) -> Node {
    let mut root = node;
    while let Some(parent) = root.parent() {
        root = parent;
    }
    root
}

fn evaluate_ns_decl<'t, 'b>(
    node: Node<'t>, qmatch: &QueryMatch<'_, 't>, buffer: &'b [u8], result: &mut StructureInfo<'b, 't>,
) -> Result<(), Error<'b>> {
    let id = match qmatch.nodes_for_capture_index(*F_ID).next() {
        Some(id) => QualifiedNameRef::try_from(id, buffer).map_err(|e| Error::BadName(e))?,
        None => return Err(Error::FieldName("id")),
    };

    result.ns_decl.entry(node).or_insert(HashSet::new()).insert(id);
    Ok(())
}

fn evaluate_ns_usage<'t, 'b>(
    scope_node: Node<'t>, qmatch: &QueryMatch<'_, 't>, buffer: &'b [u8], result: &mut StructureInfo<'b, 't>,
) -> Result<(), Error<'b>> {
    let id_node = match qmatch.nodes_for_capture_index(*F_ID).next() {
        Some(id) => id,
        None => return Err(Error::FieldName("id")),
    };
    let id = QualifiedNameRef::try_from(id_node, buffer).map_err(|e| Error::BadName(e))?;

    let alias = match qmatch.nodes_for_capture_index(*F_ALIAS).next() {
        Some(n) => Some(QualifiedNameRef::try_from(n, buffer).map_err(|e| Error::BadName(e))?),
        None => None,
    };

    let decl_node = match id_node.parent() {
        Some(p) => p,
        None => return Err(Error::BadStaticUsing(id_node.utf8_text(buffer).map_err(|e| Error::Utf8(e))?)),
    };
    let mut cursor = decl_node.walk();
    let is_static = decl_node.children(&mut cursor).any(|c| c.kind_id() == *K_STATIC);

    if let Some(alias) = alias {
        result.aliases.entry(scope_node).or_insert(HashMap::new())
            .insert(alias, id);
    } else if is_static {
        // `using static N.S.Type.Field;`
        // `N.S.Type`: the type actually being used when field is used
        // `Field`: the variable that refers to the type
        let mut qualtype = id;
        let field = qualtype.split_off(qualtype.len() - 1);
        result.aliases.entry(scope_node).or_insert(HashMap::new())
            .insert(field, qualtype);
    } else {
        result.ns_usages.entry(scope_node).or_insert(HashSet::new())
            .insert(id);
    }

    Ok(())
}

fn evaluate_type_decl<'t, 'b>(
    scope_node: Node<'t>, qmatch: &QueryMatch<'_, 't>, buffer: &'b [u8], result: &mut StructureInfo<'b, 't>,
) -> Result<(), Error<'b>> {
    let name = match qmatch.nodes_for_capture_index(*F_ID).next() {
        Some(id) => {
            QualifiedNameRef::try_from(id, buffer).map_err(|e| Error::BadName(e))?
        },
        None => {
            return Err(Error::FieldName("id"));
        },
    };

    result.type_decl.entry(scope_node).or_insert(HashSet::new())
        .insert(name);

    Ok(())
}

fn evaluate_type_usage<'t, 'b>(
    node: Node<'t>, _qmatch: &QueryMatch<'_, 't>, buffer: &'b [u8], result: &mut StructureInfo<'b, 't>,
) -> Result<(), Error<'b>> {
    // skip "using x = typename", included in ns_usage
    if let Some(user) = node.parent() && user.kind_id() == *K_USING {
        return Ok(());
    }

    let name = QualifiedNameRef::try_from(node, buffer).map_err(|e| Error::BadName(e))?;
    result.type_usages.insert(node, name);
    Ok(())
}

fn evaluate_var_decl<'t, 'b>(
    node: Node<'t>, qmatch: &QueryMatch<'_, 't>, buffer: &'b [u8], result: &mut StructureInfo<'b, 't>,
) -> Result<(), Error<'b>> {
    let id_node = match qmatch.nodes_for_capture_index(*F_ID).next() {
        Some(id) => id,
        None => return Err(Error::FieldName("id")),
    };
    let id = QualifiedNameRef::try_from(id_node, buffer).map_err(|e| Error::BadName(e))?;

    result.var_decl.entry(node)
        .or_insert(HashSet::new())
        .insert(id);
    Ok(())
}


fn evaluate_var_usage<'t, 'b>(
    node: Node<'t>, _qmatch: &QueryMatch<'_, 't>, buffer: &'b [u8], result: &mut StructureInfo<'b, 't>,
) -> Result<(), Error<'b>> {
    let name = QualifiedNameRef::try_from(node, buffer).map_err(|e| Error::BadName(e))?;
    result.var_usages.insert(node, name);
    Ok(())
}

#[cfg(test)]
mod test {
    use std::{
        collections::{HashMap, HashSet},
        cmp::Eq,
        hash::Hash,
        fmt::Debug,
    };
    use crate::parser::csharp::test::{NodeLike, NS_TEST_CODE, NS_TEST_TREE, TYPE_TEST_CODE, TYPE_TEST_TREE};
    use super::*;

    fn assert_map<'t, T, U, I>(
        actual: HashMap<Node<'t>, I>,
        expected: HashMap<T, I>,
    ) where T: From<Node<'t>> + PartialEq<Node<'t>> + Debug + Clone + Eq + Hash + Display,
        U: PartialEq<U> + Debug + Eq + Hash,
        I: IntoIterator<Item = U> + Clone {
        let mut matched = HashSet::new();
        let mut unexpected = HashSet::new();

        for (anode, aset) in actual.into_iter() {
            let eset = match expected.iter().find(|(nl, _)| **nl == anode) {
                Some((n, s)) => {
                    if !matched.insert(n.clone()) {
                        println!("Multiple matches for {n}");
                    }
                    s.clone()
                },
                None => {
                    unexpected.insert(T::from(anode));
                    continue;
                },
            };

            let aset = aset.into_iter().collect::<HashSet<U>>();
            let eset = eset.into_iter().collect::<HashSet<U>>();
            let unexpected: HashSet<&U> = aset.difference(&eset).collect();
            let missing: HashSet<&U> = eset.difference(&aset).collect();
            assert_eq!(unexpected, missing, "Mismatch between items under node {anode:?}");
        }

        let missing = expected.into_keys().filter(|n| !matched.contains(n)).collect::<HashSet<T>>();
        assert_eq!(unexpected, missing, "Mismatch between node lists");
    }

    #[test]
    fn evaluate_structure_ns() {
        let result = super::evaluate_structure(&NS_TEST_TREE, NS_TEST_CODE)
            .expect("Evaluation failed");

        assert_map(result.ns_decl, HashMap::from([
            (NodeLike::new("compilation_unit", 0, 0), HashSet::from([
                QualifiedNameRef::from("L1"),
            ])),
            (NodeLike::new("declaration_list", 6, 0), HashSet::from([
                QualifiedNameRef::from("L2"),
            ])),
        ]));

        assert_map(result.ns_usages, HashMap::from([
            (NodeLike::new("compilation_unit", 0, 0), HashSet::from([
                QualifiedNameRef::from("Ns0"),
            ])),
            (NodeLike::new("declaration_list", 6, 0), HashSet::from([
                QualifiedNameRef::from("Ns1"),
            ])),
            (NodeLike::new("declaration_list", 12, 4), HashSet::from([
                QualifiedNameRef::from("Ns2"),
            ])),
        ]));

        assert_map(result.aliases, HashMap::from([
            (NodeLike::new("compilation_unit", 0, 0), HashMap::from([
                (QualifiedNameRef::from("InnerType"), QualifiedNameRef::from("Ns0")),
                (QualifiedNameRef::from("ns0a"), QualifiedNameRef::from("Ns0.InnerNs")),
            ])),
            (NodeLike::new("declaration_list", 6, 0), HashMap::from([
                (QualifiedNameRef::from("InnerType"), QualifiedNameRef::from("Ns1")),
                (QualifiedNameRef::from("ns1a"), QualifiedNameRef::from("Ns1.InnerNs")),
            ])),
            (NodeLike::new("declaration_list", 12, 4), HashMap::from([
                (QualifiedNameRef::from("InnerType"), QualifiedNameRef::from("Ns2")),
                (QualifiedNameRef::from("ns2a"), QualifiedNameRef::from("Ns2.InnerNs")),
            ])),
        ]));

        assert_eq!(result.type_decl, HashMap::new());
        assert_eq!(result.type_usages, HashMap::new());
        assert_eq!(result.var_decl, HashMap::new());
        assert_eq!(result.var_usages, HashMap::new());
    }
}