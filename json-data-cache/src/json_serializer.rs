use std::collections::HashMap;

use serde_json::Value;

/// A tool used to stringify a json Value, while collecting all keys and building slices
/// In memory, there will be a single String with as many references to it as there are nested keys
/// This is useful when using AhoCorasick to make mass replacements
#[derive(Debug)]
pub struct JsonSerializer {
}

#[derive(Debug)]
pub struct KeyValueRange {
    pub start: usize, // Including
    pub end: usize // Excluding
}

impl From<(usize, usize)> for KeyValueRange {
    fn from(value: (usize, usize)) -> Self {
        Self {
            start: value.0,
            end: value.1
        }
    }
}

pub struct SerializedWithKeys {
    pub data: Vec<u8>,
    pub key_values: HashMap<String, KeyValueRange>,
    pub length: usize,
}

impl std::fmt::Debug for SerializedWithKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerializedWithKeys")
            .field("data", &String::from_utf8(self.data.clone()).unwrap())
            .field("key_values",&self.key_values)
            .field("length", &self.length)
            .finish()
    }
}

/// Helps to distinguish between specifically the length of sting value in a JSON
/// outer part is accounting for the surrounding quotes, but the inner part does not
pub struct JsonLength {
    pub inner: usize,
    pub outer: usize,
}
impl From<usize> for JsonLength {
    fn from(value: usize) -> Self {
        Self {
            inner: value,
            outer: value,
        }
    }
}
impl From<(usize, usize)> for JsonLength {
    fn from((inner, outer): (usize, usize)) -> Self {
        Self {
            inner,
            outer,
        }
    }
}

impl JsonSerializer {
    /// Serialize a value and return it along with a list of all possible nested keys with the start & end indexes of their pointed value in the serialized result
    /// double_serialize, if set, will also provide a second doubly serialized string with its own set of value ranges - but without final double quotes!
    pub fn serialize(value: &Value, double_serialize: bool) -> (SerializedWithKeys, Option<SerializedWithKeys>) {
        let mut path = String::new();
        let mut serialized = SerializedWithKeys {
            data: Vec::new(),
            key_values: HashMap::new(),
            length: 0
        };
        let mut double_serialized = if double_serialize {
            Some(
                SerializedWithKeys {
                    data: Vec::new(),
                    key_values: HashMap::new(),
                    length: 0
                }
            )
        } else { None };

        Self::rec_serialize(
            value,
            &mut path,
            &mut serialized,
            &mut double_serialized,
            0,
            0 // Double serialized index starts at 1 because of the final double quotes
        );

        (serialized, double_serialized)
    }

    /// Recursively serializes a Value while building a map of keys with indices to their (byte) positions in the final serialized string
    fn rec_serialize(
        value: &Value,
        path: &mut String, // Pointing to the current parent, for example list.0
        serialized: &mut SerializedWithKeys,
        double_serialized: &mut Option<SerializedWithKeys>,
        serialized_index: usize,
        double_serialized_index: usize,
    ) -> (JsonLength, JsonLength) { // Return value is the length of the newly serialized element, for serialized and double_serialized
        match value {
            Value::Null => {
                let ret = "null";
                serialized.data.extend(ret.as_bytes());
                let len = ret.as_bytes().len();
                if let Some(double_serialized) = double_serialized {
                    double_serialized.data.extend(ret.as_bytes());
                }
                (len.into(), len.into()) // Same for double serialized
            },
            Value::Bool(b) => {
                let ret = b.to_string();
                serialized.data.extend(ret.as_bytes());
                let len = ret.as_bytes().len();
                if let Some(double_serialized) = double_serialized {
                    double_serialized.data.extend(ret.as_bytes());
                }
                (len.into(), len.into()) // Same for double serialized
            },
            Value::Number(number) => {
                let ret = number.to_string();
                serialized.data.extend(ret.as_bytes());
                let len = ret.as_bytes().len();
                if let Some(double_serialized) = double_serialized {
                    double_serialized.data.extend(ret.as_bytes());
                }
                (len.into(), len.into()) // Same for double serialized
            },
            Value::String(string) => {
                let ret = Value::String(string.to_string()).to_string(); // Including potential escapes and surrounding quotes
                serialized.data.extend(ret.as_bytes());
                let len = ret.as_bytes().len();
                let double_serialized_len = if let Some(double_serialized) = double_serialized {
                    // Here we stringify an additional time (and remove the surrouding quotes)
                    let mut double_serialized_data = Value::String(ret).to_string();
                    double_serialized_data.remove(0);
                    double_serialized_data.remove(double_serialized_data.len()-1);
                    double_serialized.data.extend(double_serialized_data.as_bytes());
                    double_serialized_data.as_bytes().len()
                } else { 0 };
                ((len-2, len).into(), (double_serialized_len-2, double_serialized_len).into())
            },
            Value::Object(map) => {
                serialized.data.push(b'{');
                if let Some(double_serialized) = double_serialized {
                    double_serialized.data.push(b'{');
                }
                let mut serialized_current_map_length = 1usize;
                let mut double_serialized_current_map_length = 1usize;
                let original_path_len = path.len();
                for (idx, (key, val)) in map.iter().enumerate() {
                    if path != "" {
                        path.push('.');
                    }
                    path.push_str(key);
                    let key_serialized = Value::String(key.to_string()).to_string(); // Including potential escapes and surrounding quotes
                    if idx > 0 {
                        serialized.data.push(b',');
                        serialized_current_map_length += 1;
                    }
                    let key_serialized_bytes = key_serialized.as_bytes();
                    serialized.data.extend(key_serialized_bytes);
                    serialized.data.push(b':');
                    serialized_current_map_length += key_serialized_bytes.len() + 1;

                    if let Some(double_serialized) = double_serialized {
                        // Double serialization of key & computing its own serialized value and indices separately
                        let mut key_double_serialized = Value::String(key_serialized).to_string();
                        key_double_serialized.remove(0);
                        key_double_serialized.remove(key_double_serialized.len()-1);
                        if idx > 0 {
                            double_serialized.data.push(b',');
                            double_serialized_current_map_length += 1;
                        }
                        let key_double_serialized_bytes = key_double_serialized.as_bytes();
                        double_serialized.data.extend(key_double_serialized_bytes);
                        double_serialized.data.push(b':');
                        double_serialized_current_map_length += key_double_serialized.len() + 1;
                    }

                    let child_length = Self::rec_serialize(
                        val,
                        path,
                        serialized,
                        double_serialized,
                        serialized_index + serialized_current_map_length,
                        double_serialized_index + double_serialized_current_map_length,
                    );

                    let starting_position = serialized_current_map_length;
                    serialized_current_map_length += child_length.0.outer;

                    let (child_start, child_end) = if child_length.0.inner != child_length.0.outer {
                        // For child strings, the actual pointed value is the inner part between the quotes, not the whole thing
                        (
                            serialized_index + starting_position + 1,
                            serialized_index + serialized_current_map_length - 1
                        )
                    } else {
                        (
                            serialized_index + starting_position,
                            serialized_index + serialized_current_map_length
                        )
                    };
                    let serialized_child_range: KeyValueRange = (child_start, child_end).into();

                    serialized.key_values.insert(path.to_string(), serialized_child_range);

                    if let Some(double_serialized) = double_serialized {
                        // Double serialization handling
                        let starting_position = double_serialized_current_map_length;
                        double_serialized_current_map_length += child_length.1.outer;

                        let (child_start, child_end) = if child_length.1.inner != child_length.1.outer {
                            // For child strings, the actual pointed value is the inner part between the quotes, not the whole thing
                            (
                                double_serialized_index + starting_position + 2, // 2 characters because quotes are preceeded with backslashes
                                double_serialized_index + double_serialized_current_map_length - 2
                            )
                        } else {
                            (
                                double_serialized_index + starting_position,
                                double_serialized_index + double_serialized_current_map_length
                            )
                        };
                        println!("DOUBLE CHILD FOR {path} : {}", String::from_utf8((&double_serialized.data[child_start..child_end]).to_vec()).unwrap());
                        let double_serialized_child_range: KeyValueRange = (child_start, child_end).into();

                        double_serialized.key_values.insert(path.to_string(), double_serialized_child_range);
                    }

                    // Post key
                    path.drain(original_path_len..); // Remove the key suffix that has been temporarily added to path
                }
                serialized.data.push(b'}');
                if let Some(double_serialized) = double_serialized {
                    double_serialized.data.push(b'}');
                }
                serialized_current_map_length += 1;
                double_serialized_current_map_length += 1;
                (serialized_current_map_length.into(), double_serialized_current_map_length.into())
            },
            Value::Array(values) => {
                serialized.data.push(b'[');
                if let Some(double_serialized) = double_serialized {
                    double_serialized.data.push(b'[');
                }
                let mut serialized_current_array_length = 1usize;
                let mut double_serialized_current_array_length = 1usize;
                let original_path_len = path.len();

                for (idx, val) in values.iter().enumerate() {
                    if path != "" {
                        path.push('.');
                    }
                    let idx_str = idx.to_string();
                    path.push_str(&idx_str);

                    if idx > 0 {
                        serialized.data.push(b',');
                        serialized_current_array_length += 1;

                        if let Some(double_serialized) = double_serialized {
                            double_serialized.data.push(b',');
                            double_serialized_current_array_length += 1;
                        }
                    }

                    let child_length = Self::rec_serialize(
                        val,
                        path,
                        serialized,
                        double_serialized,
                        serialized_index + serialized_current_array_length,
                        double_serialized_index + double_serialized_current_array_length
                    );

                    let starting_position = serialized_current_array_length;
                    serialized_current_array_length += child_length.0.outer; // child_range.end here is equivalent to the length of stringified child

                    let (child_start, child_end) = if child_length.0.inner != child_length.0.outer {
                        // For child strings, the actual pointed value is the inner part between the quotes, not the whole thing
                        (
                            serialized_index + starting_position + 1,
                            serialized_index + serialized_current_array_length - 1
                        )
                    } else {
                        (
                            serialized_index + starting_position,
                            serialized_index + serialized_current_array_length
                        )
                    };

                    let serialized_child_range: KeyValueRange = (child_start, child_end).into();

                    serialized.key_values.insert(path.to_string(), serialized_child_range);

                    if let Some(double_serialized) = double_serialized {
                        let starting_position = double_serialized_current_array_length;
                        double_serialized_current_array_length += child_length.1.outer;

                        let (child_start, child_end) = if child_length.1.inner != child_length.1.outer {
                            // For child strings, the actual pointed value is the inner part between the quotes, not the whole thing
                            (
                                double_serialized_index + starting_position + 2, // 2 characters because quotes are preceeded with backslashes
                                double_serialized_index + double_serialized_current_array_length - 2
                            )
                        } else {
                            (
                                double_serialized_index + starting_position,
                                double_serialized_index + double_serialized_current_array_length
                            )
                        };
                        println!("DOUBLE CHILD FOR {path} : {}", String::from_utf8((&double_serialized.data[child_start..child_end]).to_vec()).unwrap());

                        let double_serialized_child_range: KeyValueRange = (child_start, child_end).into();

                        double_serialized.key_values.insert(path.to_string(), double_serialized_child_range);
                    }

                    // Post key
                    path.drain(original_path_len..); // Remove the key suffix that has been temporarily added to path
                }
                serialized.data.push(b']');
                serialized_current_array_length += 1;
                if let Some(double_serialized) = double_serialized {
                    double_serialized.data.push(b']');
                    double_serialized_current_array_length += 1;
                }
                (serialized_current_array_length.into(), double_serialized_current_array_length.into())
            },
        }
    }
}