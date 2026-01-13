use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter, Result as FResult},
    sync::LazyLock
};
use tree_sitter::{Tree, Query, QueryCursor, QueryError, Node, StreamingIterator};
use super::{
    CS_LANG,
    queries::QUERY_ALL,
};

#[derive(Debug)]
pub enum Error<'a> {
    Query(QueryError),
    FieldName(&'a str),
    FieldId(u32),
    Utf8,
}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        match self {
            Self::Query(q) => write!(f, "{q}"),
            Self::FieldName(e) => write!(f, "No such field '{e}'"),
            Self::FieldId(id) => write!(f, "No such field {id}"),
            Self::Utf8 => write!(f, "Failed to convert buffer to UTF-8"),
        }
    }
}

impl<'a> std::error::Error for Error<'a> {}

static QUERY: LazyLock<Query> = LazyLock::new(|| {
    Query::new(&CS_LANG, QUERY_ALL).expect("Failed to compile query")
});

#[derive(Default)]
pub struct ScopeResults<'a> {
    pub namespaces: HashSet<&'a str>,
    pub type_declarations: HashSet<Node<'a>>,
    pub id_scopes: HashMap<Node<'a>, HashSet<&'a str>>,
    pub id_uses: HashSet<Node<'a>>,
}

pub fn evaluate_scopes<'t, 'b>(tree: &'t Tree, buffer: &'b [u8]) -> Result<ScopeResults<'b>, Error<'b>> {
    let mut results = ScopeResults { ..Default::default() };

    let mut cursor = QueryCursor::new();
    let mut iter = cursor.matches(&QUERY, tree.root_node(), buffer);

    while let Some(m) = iter.next() {
        
    }

    Ok(results)
}