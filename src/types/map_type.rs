use std::borrow::Cow;

use super::{CommonType, TypeId, StructType, FieldType, WireType};

#[derive(Clone, Debug, Deserialize)]
pub struct MapType {
    pub common: CommonType,
    #[serde(rename = "Key")]
    pub key: TypeId,
    #[serde(rename = "Elem")]
    pub elem: TypeId
}

pub static MAP_TYPE_DEF: WireType = {
    WireType::Struct(StructType {
        common: CommonType { name: Cow::Borrowed("MapType"), id: TypeId::MAP_TYPE },
        fields: Cow::Borrowed(&[
            FieldType { name: Cow::Borrowed("common"), id: TypeId::COMMON_TYPE },
            FieldType { name: Cow::Borrowed("Key"), id: TypeId::INT },
            FieldType { name: Cow::Borrowed("Elem"), id: TypeId::INT }
        ])
    })
};
