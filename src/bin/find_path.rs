use std::{
    collections::HashMap, error::Error
};
use tree_sitter_c_sharp::NODE_TYPES;

#[derive(serde::Deserialize)]
struct Type {
    r#type: String,
    named: bool,
    #[serde(default = "Vec::new")]
    subtypes: Vec<TypeRef>,
    #[serde(default = "HashMap::new")]
    fields: HashMap<String, Field>,
    children: Field,
}

#[derive(serde::Deserialize)]
struct TypeRef {
    r#type: String,
    named: bool,
}

#[derive(serde::Deserialize)]
struct Field {
    multiple: bool,
    required: bool,
    #[serde(default = "Vec::new")]
    types: Vec<TypeRef>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let types: Vec<Type> = serde_json::from_str(NODE_TYPES)?;
    Ok(())
}