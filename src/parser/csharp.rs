mod queries;
mod find_types;
pub mod type_broker;
pub mod qualified_name;

#[cfg(feature = "locstring")]
mod find_locstrings;

use std::{
    fs::File,
    io::Read,
    path::PathBuf,
    sync::{Arc, Mutex, LazyLock},
};
use tree_sitter::{Language, Parser};
use tree_sitter_c_sharp as cs;
use crate::{Asset, parser::ParseError};
use type_broker::TypeBroker;

pub static CS_LANG: LazyLock<Language> = LazyLock::new(|| {
    cs::LANGUAGE.into()
});

pub fn parse(asset: &mut Asset, relative_to: Option<&PathBuf>, broker: &Arc<Mutex<TypeBroker>>) -> Result<Vec<Asset>, ParseError> {
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

    parse_buffer(&buf, asset, &path.clone(), broker)
}

fn parse_buffer(
    buffer: &[u8], 
    asset: &mut Asset, 
    path: &PathBuf,
    broker: &Arc<Mutex<TypeBroker>>
) -> Result<Vec<Asset>, ParseError> {
    println!("parse_buffer");
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

    find_types::find_types(&tree, buffer, asset, &mut def_assets, broker)?;

    #[cfg(feature = "locstring")]
    find_locstrings::find_locstrings(&tree, buffer, path, asset)?;

    Ok(def_assets)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;
    use pretty_assertions::assert_eq;
    use crate::{AssetType, Id, Relation, QualifiedName};

    const CODE: &[u8] = include_bytes!("./csharp_test.cs");

    #[test]
    fn test_parse_decls() -> Result<(), ParseError> {
        let mut file_asset = Asset {
            id: Id::Guid(uuid::Uuid::nil()),
            asset_type: AssetType::CsFile,
            ..Default::default()
        };
        let broker = Arc::new(Mutex::new(TypeBroker::new()));
        let type_assets = parse_buffer(CODE, &mut file_asset, &"no_path".into(), &broker)?;

        println!("Found assets: {:?}", type_assets.iter().map(|a| &a.id).collect::<Vec<&Id>>());
        assert_eq!(file_asset.relations, HashSet::from([
            Relation::Uses(Id::Loc("NormalKey".into())),
            Relation::Uses(Id::Loc("PrefixedKey".into()))
        ]));

        let class_t = Asset {
            id: Id::CsType(QualifiedName::from("My.Namespace.MyClass")),
            asset_type: AssetType::CsType,
            relations: HashSet::from([
                Relation::ContainedBy(file_asset.id.clone()),
            ]),
            ..Default::default()
        };
        assert_eq!(
            type_assets.iter().find(|a| a.id == class_t.id).expect("Missing class asset"),
            &class_t,
        );

        let delegate_t = Asset {
            id: Id::CsType(QualifiedName::from("My.Namespace.MyClass.MyDelegate")),
            asset_type: AssetType::CsType,
            relations: HashSet::from([
                Relation::ContainedBy(file_asset.id.clone()),
                //Relation::ContainedBy(class_t.id.clone()),
            ]),
            ..Default::default()
        };
        
        assert_eq!(
            type_assets.iter().find(|a| a.id == delegate_t.id).expect("Missing delegate asset"),
            &delegate_t,
        );

        let underclass_t = Asset {
            id: Id::CsType(QualifiedName::from("My.Namespace.MyClass.UnderClass")),
            asset_type: AssetType::CsType,
            relations: HashSet::from([
                Relation::ContainedBy(file_asset.id.clone()),
                //Relation::ContainedBy(class_t.id.clone()),
            ]),
            ..Default::default()
        };
        
        assert_eq!(
            type_assets.iter().find(|a| a.id == underclass_t.id).expect("Missing underclass asset"),
            &underclass_t,
        );

        let struct_t = Asset {
            id: Id::CsType(QualifiedName::from("My.Namespace.MyStruct")),
            asset_type: AssetType::CsType,
            relations: HashSet::from([
                Relation::ContainedBy(file_asset.id.clone()),
            ]),
            ..Default::default()
        };
        
        assert_eq!(
            type_assets.iter().find(|a| a.id == struct_t.id).expect("Missing struct asset"),
            &struct_t,
        );

        let enum_t = Asset {
            id: Id::CsType(QualifiedName::from("My.Namespace.MyEnum")),
            asset_type: AssetType::CsType,
            relations: HashSet::from([
                Relation::ContainedBy(file_asset.id.clone()),
            ]),
            ..Default::default()
        };
        
        assert_eq!(
            type_assets.iter().find(|a| a.id == enum_t.id).expect("Missing enum asset"),
            &enum_t,
        );

        let interface_t = Asset {
            id: Id::CsType(QualifiedName::from("My.Namespace.IMyInterface")),
            asset_type: AssetType::CsType,
            relations: HashSet::from([
                Relation::ContainedBy(file_asset.id.clone()),
            ]),
            ..Default::default()
        };
        
        assert_eq!(
            type_assets.iter().find(|a| a.id == interface_t.id).expect("Missing interface asset"),
            &interface_t,
        );

        let inner_t = Asset {
            id: Id::CsType(QualifiedName::from("My.Namespace.InnerNamespace.InnerClass")),
            asset_type: AssetType::CsType,
            relations: HashSet::from([
                Relation::ContainedBy(file_asset.id.clone()),
            ]),
            ..Default::default()
        };
        
        assert_eq!(
            type_assets.iter().find(|a| a.id == inner_t.id).expect("Missing inner class asset"),
            &inner_t,
        );

        assert_eq!(type_assets.len(), 7);

        Ok(())
    }

    #[test]
    fn test_parse_refs() -> Result<(), ParseError> {
        let mut file_asset = Asset {
            id: Id::Guid(uuid::Uuid::nil()),
            asset_type: AssetType::CsFile,
            ..Default::default()
        };
        let broker = Arc::new(Mutex::new(TypeBroker::new()));
        parse_buffer(CODE, &mut file_asset, &"no_path".into(), &broker)?;
        let broker = Arc::into_inner(broker).unwrap().into_inner().unwrap();

        println!("Type requests: {:#?}", broker.requests().iter().collect::<Vec<&type_broker::TypeRequest>>());
        
        let scoped_ns = vec![
            QualifiedName::from("My.DifferentNamespace"),
            QualifiedName::from("My"),
            QualifiedName::from("My.Namespace"),
        ];

        assert!(broker.requests().contains(&type_broker::TypeRequest::new(
            &Id::CsType(QualifiedName::from("My.Namespace.MyClass")),
            QualifiedName::from("My.OtherNamespace.LocalizedString"),
            &scoped_ns,
        )));

        assert!(broker.requests().contains(&type_broker::TypeRequest::new(
            &Id::CsType(QualifiedName::from("My.Namespace.MyClass")),
            QualifiedName::from("LocalizedString"),
            &scoped_ns,
        )));

        assert!(broker.requests().contains(&type_broker::TypeRequest::new(
            &Id::CsType(QualifiedName::from("My.Namespace.MyClass")),
            QualifiedName::from("LocStringCache"),
            &scoped_ns,
        )));
        
        assert_eq!(broker.requests().len(), 3);

        Ok(())
    }
}
