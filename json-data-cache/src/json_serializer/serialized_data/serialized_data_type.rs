use indexmap::IndexMap;
use serde_json::Number;

use crate::json_serializer::serialized_data::serialized_data_node::SerializedDataNode;

pub enum SerializedDataType {
    Null,
    Bool,
    Number,
    String,
    Array(Vec<SerializedDataNode>),
    Object(IndexMap<String, SerializedDataNode>),
}