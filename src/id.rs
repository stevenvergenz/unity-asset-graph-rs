use uuid::Uuid;
use serde::Serialize;

// #[derive(PartialEq, Eq, Hash, Debug, Clone)]
// pub enum Id {
//     Guid(Uuid),
//     Loc(String),
// }

// impl Serialize for Id {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         match self {
//             Id::Guid(uuid) => serializer.serialize_bytes(uuid.as_bytes()),
//             Id::Loc(key) => serializer.serialize_str(Uuid::new_v5(Uuid::NAMESPACE_URL, name)),
//         }
//     }
// }

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize)]
pub struct Id(Uuid);

impl Id {
    pub fn new_uuid(id: Uuid) -> Self {
        Id(id)
    }

    pub fn new_loc(name: &str) -> Self {
        Id(Uuid::new_v5(&Uuid::NAMESPACE_URL, format!("loc:{name}").as_bytes()))
    }
}