use tree_sitter::{
    Node, Query, QueryCursor, QueryError, QueryMatch, StreamingIterator, Tree
};
use std::{
    collections::HashSet, 
};
use const_format::{formatcp, concatcp};

fn _debug_up(node: Node, buffer: &[u8]) {
    let mut n = Some(node);
    while let Some(node) = n {
        let text = node.utf8_text(buffer).unwrap().split('\n').next().unwrap();
        if text.len() < 100 {
            println!("{}: {}", node.kind(), text);
        }
        else {
            println!("{}: {}...<{} bytes>", node.kind(), &text[..100], node.end_byte() - node.start_byte() - 100);
        }
        n = node.parent();
    }
    println!();
}

fn _debug_down(node: Node, buffer: &[u8], max_depth: usize) {
    fn helper(node: Node, buffer: &[u8], depth: usize, max_depth: usize) {
        let indent = " ".repeat(depth);
        let kind = node.kind();
        let text = node.utf8_text(buffer).unwrap().split('\n').next().unwrap();
        if text.len() < 100 {
            println!("{indent}{kind}: {text}");
        }
        else {
            println!("{indent}{kind}: {}...<{} bytes>", &text[..100], node.end_byte() - node.start_byte() - 100);
        }

        if depth >= max_depth {
            return;
        }
        
        let mut cursor = node.walk();
        for c in node.children(&mut cursor) {
            helper(c, buffer, depth + 1, max_depth);
        }
    }
    helper(node, buffer, 0, max_depth);
}

/// Finds all the "normal" namespace import directives. Captures the directive "ns_use" and the namespace "id".
/// NOTE: Includes "using static" directives, which are not namespace imports. Must account for these manually.
const NS_USAGE: &str = r#"
    (using_directive
        [(qualified_name) (identifier)] @id
        !name
    ) @ns_use
"#;

/// Find all type declarations, and capture the type ID "id"
const TYPE_DECL_ID: &str = r#"
    (class_declaration
        name: (identifier) @id
    )
    (delegate_declaration
        name: (identifier) @id
    )
    (enum_declaration
        name: (identifier) @id
    )
    (interface_declaration
        name: (identifier) @id
    )
    (record_declaration
        name: (identifier) @id
    )
    (struct_declaration
        name: (identifier) @id
    )
"#;

/// Find all top-level type declarations, i.e. in a namespace or un-namespaced.
/// Captures the declaration node "type_decl" and the type identifier "id".
const TYPE_DECL: &str = formatcp!(r#"
    (namespace_declaration
        body: (declaration_list
            [{TYPE_DECL_ID}] @type_decl
        )
    )
    (compilation_unit
        [{TYPE_DECL_ID}] @type_decl
    )
"#);

/// Matches a variable declaration. Captures the var identifier "id"
const VAR_DECL_ID: &str = r#"
    (variable_declaration
        (variable_declarator
            name: (identifier) @id
        )
    )
"#;

/// Matches a function argument declaration. Captures the param identifier "id"
const PARAM_DECL_ID: &str = r#"
    (parameter_list
        (parameter
            name: (identifier) @id
        )
    )
"#;

const VAR_DECL_PARTS: [&str; 5] = [
    // "normal" variables in a code block
    formatcp!(r#"
        (block [
            (local_declaration_statement {VAR_DECL_ID})
            (fixed_statement {VAR_DECL_ID})
            (using_statement {VAR_DECL_ID})
        ])
    "#),

    // the iterator in a for statement
    formatcp!(r#"
        (for_statement initializer: {VAR_DECL_ID})
    "#),

    // un-namespaced type declarations
    formatcp!(r#"
        (compilation_unit [
            {TYPE_DECL_ID}
            (using_directive name: (identifier) @id)
            (using_directive
                "static"
                !name
                (qualified_name
                    name: (identifier) @id
                )
            )
        ])
    "#),

    // identifiers declared in a namespace or type body, i.e. type/field/property/method names
    formatcp!(r#"
        (declaration_list [
            {TYPE_DECL_ID}
            (field_declaration {VAR_DECL_ID})
            (event_field_declaration {VAR_DECL_ID})
            (property_declaration name: (identifier) @id)
            (method_declaration name: (identifier) @id)
        ])
    "#),

    // variables declared as function arguments
    formatcp!(r#"
        (constructor_declaration parameters: {PARAM_DECL_ID})
        (method_declaration parameters: {PARAM_DECL_ID})
        (operator_declaration parameters: {PARAM_DECL_ID})
        (lambda_expression parameters: {PARAM_DECL_ID})
        (anonymous_method_expression parameters: {PARAM_DECL_ID})
        (local_function_statement parameters: {PARAM_DECL_ID})
        (indexer_declaration
            parameters: (bracketed_parameter_list
                (parameter
                    name: (identifier) @id
                )
            )
        )
    "#),
];

/// Matches all declarations of all types of variables. Captures the variable identifier "id" and its scope "var_decl"
const VAR_DECL: &str = concatcp!(
    "[", VAR_DECL_PARTS[0], VAR_DECL_PARTS[1], VAR_DECL_PARTS[2], VAR_DECL_PARTS[3], VAR_DECL_PARTS[4], "] @var_decl"
);

/// Matches all usages of types. Captures the type name "type_use", including any generic args.
/// Note: Includes types in `using alias = T` directives. These must be excluded manually.
const TYPE_USAGE: &str = r#"
    (type/identifier) @type_use
    (type/qualified_name
        qualifier: [(identifier) (qualified_name) (generic_name)]
    ) @type_use
    (type/generic_name) @type_use
    (type/tuple_type
        (tuple_element
            type: [(identifier) (qualified_name) (generic_name)] @type_use
        )
    )
    (type/scoped_type
        type: [(identifier) (qualified_name) (generic_name)] @type_use
    )
    (type/array_type 
        type: [(identifier) (qualified_name) (generic_name)] @type_use
    )
    (type/nullable_type 
        type: [(identifier) (qualified_name) (generic_name)] @type_use
    )
    (type/ref_type 
        type: [(identifier) (qualified_name) (generic_name)] @type_use
    )
"#;

/// Matches all uses of variables, which will include some type references not caught by `TYPE_USAGE`. Filter against
/// `VAR_DECL` in the current scope to find them. Captures the variable/type name as "var_use".
const VAR_USAGE: &str = r#"
    (lvalue_expression/identifier) @var_use
    (lvalue_expression/generic_name) @var_use
    (lvalue_expression/member_access_expression
        expression: [(identifier) (qualified_name) (generic_name)] @var_use
    )
"#;

/// Matches everything we're looking for. Captures "ns_use", "type_decl", "var_decl", "id", "type_use", and "var_use".
pub const QUERY_ALL: &str = concatcp!(
    NS_USAGE,
    TYPE_DECL,
    VAR_DECL,
    TYPE_USAGE,
    VAR_USAGE,
);

#[cfg(test)]
mod test {
    use std::sync::LazyLock;
    use tree_sitter::{Parser};
    use crate::parser::csharp::CS_LANG;
    use super::*;

    const CODE: &[u8] = include_bytes!("../csharp_test.cs");
    static TREE: LazyLock<Tree> = LazyLock::new(|| {
        let mut parser = Parser::new();
        parser.set_language(&CS_LANG).expect("Failed to set language, bad lang version");
        parser.parse(CODE, None).expect("Failed to read code")
    });

    fn collect_set<'c, 't>(
        mut iter: impl StreamingIterator<Item = QueryMatch<'c, 't>>, 
        field: u32, 
        buffer: &'_ [u8],
    ) -> HashSet<&'_ str>
    where 't: 'c {
        let mut results = HashSet::new();
        while let Some(m) = iter.next() {
            let node = m.nodes_for_capture_index(field).next().expect("Failed to find captured field");
            let id = node.utf8_text(buffer).expect("Failed to decode UTF-8");
            results.insert(id);
        }
        results
    }

    #[test]
    fn use_ns() {
        let query = Query::new(&CS_LANG, NS_USAGE).expect("Failed to compile namespace query");
        let mut cursor = QueryCursor::new();
        let iter = cursor.matches(&query, TREE.root_node(), CODE);

        let field = query.capture_index_for_name("id").expect("Failed to find field 'id'");
        let namespaces = collect_set(iter, field, CODE);

        assert_eq!(namespaces, HashSet::from(["X", "X.Y.Z.Class.StaticField", "System.Text"]));
    }

    #[test]
    fn type_decl() {
        let query = Query::new(&CS_LANG, TYPE_DECL).expect("Failed to compile type declaration query");
        let mut cursor = QueryCursor::new();
        let iter = cursor.matches(&query, TREE.root_node(), CODE);

        let field = query.capture_index_for_name("id").expect("Failed to look up capture id");
        let types = collect_set(iter, field, CODE);

        assert_eq!(types, HashSet::from(["ClassB", "ClassC"]));
    }

    #[test]
    fn var_decl() {
        let query = Query::new(&CS_LANG, VAR_DECL).expect("Failed to compile variable decl query");
        let mut cursor = QueryCursor::new();
        let iter = cursor.matches(&query, TREE.root_node(), CODE);

        let field = query.capture_index_for_name("id").expect("Failed to look up capture id");
        let vars = collect_set(iter, field, CODE);

        assert_eq!(vars, HashSet::from([
            "XYC", "ClassB", "Delegate", "InnerClass", "A", "B", "x", "Ap", "Method", "a", "b", "c", "sb", "i", "ClassC",
            "poolobj", "StaticField",
        ]));
    }

    #[test]
    fn type_usage() {
        let query = Query::new(&CS_LANG, TYPE_USAGE).expect("Failed to compile type usage query");
        let mut cursor = QueryCursor::new();
        let iter = cursor.matches(&query, TREE.root_node(), CODE);

        let field = query.capture_index_for_name("type").expect("Failed to look up capture index for 'type'");
        let types = collect_set(iter, field, CODE);

        assert_eq!(types, HashSet::from([
            "X.Y.Class", "Delegate", "StringBuilder", "InnerClass",
        ]));
    }

    #[test]
    fn var_usage() {
        let query = Query::new(&CS_LANG, VAR_USAGE).expect("Failed to compile variable usage query");
        let mut cursor = QueryCursor::new();
        let iter = cursor.matches(&query, TREE.root_node(), CODE);

        let field = query.capture_index_for_name("var_use").expect("Failed to look up capture indoex for 'var_use'");
        let vars = collect_set(iter, field, CODE);

        assert_eq!(vars, HashSet::from([
            "StaticField", "A", "A", "x", "B", "XYC", "ObjectPool<InnerClass>", "StringBuilderCache",
            "i", "sb", "value"
        ]));
    }

    #[test]
    fn query_all() -> Result<(), QueryError> {
        Query::new(&CS_LANG, QUERY_ALL).map(|_| ())
    }
}
