use crate::json_serializer::{key_value_range::Range, serialized_data::serialized_data_type::SerializedDataType};

pub struct SerializedDataNode {
    range: Range, // Start and end index of this node in the serialized data
    double_range: Range, // Start and end index of this node in the double serialized data
    node_type: SerializedDataType
}