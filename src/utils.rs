use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Deserializer};

/// Custom deserialize function to convert mongodb ObjectId to String
/// Field level attribute `deserialize_with` to be provided as below
///```
/// #[derive(serde::Deserialize)]
/// struct MyStruct {
///    #[serde(deserialize_with = "deserialize_objectid")]
///    _id: String,
///    // other fields
/// }
///```
///   
pub fn deserialize_objectid<'de, D>(val: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let oid = ObjectId::deserialize(val)?;
    Ok(oid.to_hex())
}
