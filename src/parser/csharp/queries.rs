use tree_sitter::Query;
use std::sync::LazyLock;

pub static USING_QUERY: LazyLock<Query> = LazyLock::new(|| {
    Query::new(&super::CS_LANG, r#"
        [
            (using_directive
                name: (identifier) @alias
                (type) @type
            )
            (using_directive
                (qualified_name) @type
                !name
            )
        ]
    "#).expect("Failed to compile using query")
});

/// Query to find class, struct, enum, and interface declarations.
/// Syntax tree identifiers come from https://github.com/tree-sitter/tree-sitter-c-sharp/blob/master/src/node-types.json
pub static TYPE_DECL: LazyLock<Query> = LazyLock::new(|| {
    Query::new(&super::CS_LANG, r#"
        (type_declaration) @decl
    "#).expect("Failed to compile class query")
});

pub static TYPE_USAGE: LazyLock<Query> = LazyLock::new(|| {
    Query::new(&super::CS_LANG, r#"
        [
            (type/identifier) @type
            (type/generic_name) @type
            (type/qualified_name
                qualifier: [(identifier) (qualified_name) (generic_name)]
            ) @type
            (type/tuple_type
                (tuple_element
                    type: [(identifier) (qualified_name) (generic_name)] @type
                )
            )
            (type/scoped_type
                type: [(identifier) (qualified_name) (generic_name)] @type
            )
            (type/array_type 
                type: [(identifier) (qualified_name) (generic_name)] @type
            )
            (type/nullable_type 
                type: [(identifier) (qualified_name) (generic_name)] @type
            )
            (type/ref_type 
                type: [(identifier) (qualified_name) (generic_name)] @type
            )
        ]
    "#).expect("Failed to compile usage query")
});

pub static VAR_DECL: LazyLock<Query> = LazyLock::new(|| {
    Query::new(&super::CS_LANG, r#"
        [
            (block
                ([(local_declaration_statement) (fixed_statement) (using_statement)]
                    (variable_declaration
                        (variable_declarator
                            name: (identifier) @varname
                        )
                    )
                )
            )
            (for_statement
                initializer: (variable_declaration
                    (variable_declarator
                        name: (identifier) @varname
                    )
                )
            )
            (type_declaration
                name: (identifier) @varname
            )
            (type_declaration
                body: (declaration_list
                    ([(field_declaration) (event_field_declaration)]
                        (variable_declaration
                            (variable_declarator
                                name: (identifier) @varname
                            )
                        )
                    )
                )
            )
            (type_declaration
                body: (declaration_list
                    (declaration
                        name: (identifier) @varname
                    )
                )
            )
            ([(declaration) (lambda_expression) (anonymous_method_expression) (local_function_statement)]
                ([(parameter_list) (bracketed_parameter_list)]
                    (parameter
                        name: (identifier) @varname
                    )
                )
            )
        ] @scope
    "#).expect("Failed to compile variable usage query")
});

pub static VAR_USAGE: LazyLock<Query> = LazyLock::new(|| {
    Query::new(&super::CS_LANG, r#"
        (member_access_expression
            expression: [(identifier) (generic_name) (qualified_name)] @name
        )
    "#).expect("Failed to compile variable usage query")
});