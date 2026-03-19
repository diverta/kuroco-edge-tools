use core::{fmt, str};
use std::{collections::HashMap, io, rc::Rc};

use aho_corasick::AhoCorasick;
use regex::Regex;
use serde_json::{Value, json};

use crate::{error::JsonDataCacheError, json_serializer::{JsonSerializer, serialized_data::SerializedDataLegacy}};

pub mod error;
pub mod json_serializer;

#[derive(Debug)]
pub struct DataCache {
    pub root: Value,
    options: DataCacheOptions,
    serialized_data: DataCacheSerializedData // Cache for AC & replacements, updated on each insert
}

#[derive(Debug, Default)]
pub struct DataCacheSerializedData {
    is_built: bool,
    ac: Option<AhoCorasick>,
    serialized: Option<SerializedDataLegacy>, // In memory serialized data cache tree
    double_serialized: Option<SerializedDataLegacy>, // In memory doubly serialized data cache tree
    replacements: Vec<Rc<[u8]>>
}

#[derive(Debug, Default)]
pub struct DataCacheOptions {
    pub reserved_cache_top_level_names: Vec<String>
}

impl fmt::Display for DataCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}",
            serde_json::to_string(&self.as_string_values_map()).unwrap()
        ))
    }
}

impl DataCache {
    pub fn new(options: DataCacheOptions) -> Self {
        let new_data_cache = Self {
            root: json!({}),
            options,
            serialized_data: DataCacheSerializedData::default()
        };
        new_data_cache
    }

    fn insert_rec(parent: &mut Value, path: &str, mut value: Value) {
        let two_parts: Vec<&str> = path.splitn(2, '.').collect(); // Can only have length 1 or 2

        if two_parts.len() == 1 {
            let current_key = two_parts.get(0).unwrap();
            match parent {
                Value::Array(p) => {
                    if current_key.len() == 0 {
                        p.push(value);
                    }
                },
                Value::Object(parent_object) => {
                    if current_key.len() > 0 {
                        let new_current = parent_object
                            .entry(*current_key)
                            .or_insert(json!({}));
                        Self::merge_rec(new_current, value);
                    }
                },
                _ => {
                    // Can't handle other cases. Object case should've been handled in the previous iteration
                },
            }
        } else {
            match parent {
                Value::Object(parent_object) => {
                    // There is something else to insert
                    let current_key = two_parts.get(0).unwrap();
                    let remaining_path = two_parts.get(1).unwrap();
                    if remaining_path.contains('.') {
                        parent_object
                            .entry(*current_key)
                            .and_modify(|new_parent| {
                                Self::insert_rec(new_parent, remaining_path, value.clone());
                            })
                            .or_insert_with(|| {
                                let mut new_parent = json!({});
                                Self::insert_rec(&mut new_parent, remaining_path, value);
                                new_parent
                            });
                    } else {
                        // No more nesting
                        if remaining_path == &"" {
                            // Build array (path ended with a single '.')
                            let new_array = parent_object
                                .entry(*current_key)
                                .and_modify(|existing: &mut Value| {
                                    if !existing.is_array() {
                                        // Force conversion to array
                                        *existing = Value::Array(Vec::new());
                                    }
                                })
                                .or_insert(Value::Array(Vec::new()));
                            new_array.as_array_mut().unwrap().push(value);
                        } else {
                            if parent_object.get(*current_key).map(|found| found.is_array()).unwrap_or(false) {
                                // Special case : parent object is an array and we set a key => we want to set the give key & value for each object item
                                let arr = parent_object.get_mut(*current_key).unwrap().as_array_mut().unwrap();
                                if value.is_array() {
                                    // Prepare for special case of special case, and reverse value array to efficiently consume it during iterating
                                    value.as_array_mut().unwrap().reverse();
                                }
                                for item in arr.iter_mut() {
                                    let value_to_insert = if value.is_array() {
                                        // Even more special case : if the value is an array, distribute it
                                        let value_arr = value.as_array_mut().unwrap();
                                        if value_arr.len() > 0 {
                                            value_arr.pop().unwrap()
                                        } else {
                                            // Value array was shorter than parent, nothing left to distribute
                                            break;
                                        }
                                    } else {
                                        value.clone()
                                    };
                                    if item.is_object() {
                                        let previous_value = item.as_object_mut().unwrap()
                                            .entry(remaining_path.to_string())
                                            .or_insert(Value::Object(serde_json::Map::new()));
                                        if previous_value.is_object() && value_to_insert.is_object() {
                                            // Both are objects : merge is possible
                                            Self::merge_rec(previous_value, value_to_insert);
                                        } else {
                                            // Replace the existing value by new one
                                            item.as_object_mut().unwrap().insert(remaining_path.to_string(), value_to_insert);
                                        }
                                    } else {
                                        // Not an object - ignore
                                    }
                                }
                            } else {
                                // Parent is not an object (not array special case), so we force its conversion to object
                                let new_object = parent_object
                                    .entry(*current_key)
                                    .and_modify(|existing| {
                                        if !existing.is_object() {
                                            // Force conversion to object
                                            *existing = Value::Object(serde_json::Map::new());
                                        }
                                    })
                                    .or_insert(Value::Object(serde_json::Map::new()));
                                let previous_value = new_object.as_object_mut().unwrap()
                                    .entry(remaining_path.to_string())
                                    .or_insert(Value::Object(serde_json::Map::new()));
                                if previous_value.is_object() && value.is_object() {
                                    // Both are objects : merge is possible
                                    Self::merge_rec(previous_value, value);
                                } else {
                                    // Replace the existing value by new one
                                    new_object.as_object_mut().unwrap().insert(remaining_path.to_string(), value);
                                }
                            }
                        }
                    }
                },
                _ => {
                    // Unable to process
                }
            }
        }
    }

    fn merge_rec(a: &mut Value, b: Value) {
        if let Value::Object(a) = a {
            if let Value::Object(b) = b {
                for (k, v) in b {
                    if v.is_null() {
                        a.remove(&k);
                    }
                    else {
                        Self::merge_rec(a.entry(k).or_insert(Value::Null), v);
                    }
                } 
    
                return;
            }
        }
    
        *a = b;
    }

    pub fn merge(&mut self, other: Value) {
        Self::merge_rec(&mut self.root, other);
    }

    /// Inserts the new value. Path containing dot '.' will build nested object.
    /// If the target object exists and is an array, the value will be appended
    pub fn insert(&mut self, path: &str, value: Value) {
        Self::insert_rec(&mut self.root, path, value);

        self.on_after_insert();
    }

    // A more efficient insert of many elements that only recalculates final state after all insertions instead of after each
    pub fn insert_bulk(&mut self, values: Vec<(String, Value)>) {
        for (path, value) in values {
            Self::insert_rec(&mut self.root, &path, value);
        }
        self.on_after_insert();
    }

    fn on_after_insert(&mut self) {
        // Reset (cached) serialized data
        self.serialized_data = DataCacheSerializedData::default()
    }

    fn as_string_values_map_rec(map: &mut HashMap<String, String>, parent: &Value, current_path: String) {
        let build_prefix = |path: &String| {
            if path.len() > 0 {
                format!("{}.", path)
            } else {
                String::new()
            }
        };
        match parent {
            Value::Array(a) => {
                for (idx, el) in a.iter().enumerate() {
                    Self::as_string_values_map_rec(map, el, format!("{}{}", build_prefix(&current_path), idx));
                }
                map.insert(current_path, serde_json::to_string(a).unwrap_or(String::from("[]")));
            },
            Value::Object(o) => {
                for (k, v) in o {
                    Self::as_string_values_map_rec(map, v, format!("{}{}", build_prefix(&current_path), k));
                }
                if current_path.len() > 0 {
                    map.insert(current_path, serde_json::to_string(o).unwrap_or(String::from("{}")));
                }
            },
            Value::String(v) => {
                map.insert(current_path, v.to_string());
            },
            Value::Number(v) => {
                map.insert(current_path, v.to_string());
            },
            Value::Bool(v) => {
                map.insert(current_path, v.to_string());
            },
            Value::Null => {
                map.insert(current_path, "null".to_string());
            },
        }
    }

    /// Returns a map with all String values of the data cache, using '.' for nested elements and numbers for array keys
    pub fn as_string_values_map(&self) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::new();
        Self::as_string_values_map_rec(&mut map, &self.root, String::new());
        map
    }

    fn target_to_pointer(target: &str)-> String {
        format!("/{}", target.replace(".", "/"))
    }

    /// Access a data node in the tree through a pointer path expression
    /// Example: get("root_object.some_array.0") => <first element of array>
    pub fn get<'b>(&'b self, target: &str) -> Option<&'b Value> {
        let target_pointer = DataCache::target_to_pointer(target);
        self.root.pointer(&target_pointer)
    }

    /// Get a list of references using a single wildcard * to collect specific data from a (nested) array
    /// Example: get_list("root_object.*.id") => `[1,2,3,...]` assuming every element of the array is an object having an id property
    pub fn get_list<'b>(&'b self, target: &str) -> Vec<&'b Value> {
        let wildcard_match_indices: Vec<_> = target.match_indices("*").collect();
        match wildcard_match_indices.len() {
            0 => match self.root.pointer(&DataCache::target_to_pointer(target)) {
                // Standard usage without wildcards => return a vector of 1 or 0 elements
                Some(found) => Vec::from([found]),
                None => Vec::new(),
            },
            1 => {
                let (wc_idx, _) = wildcard_match_indices.get(0).unwrap();
                let parent_array: Option<&Value> = if *wc_idx == 0 {
                    if self.root.is_array() {
                        // Actually impossible case, because with DataCache, root should never be an array.
                        // But just in case this changes in the future, we can easily support this case
                        Some(&self.root)
                    } else {
                        None
                    }
                } else {
                    if &target[*wc_idx-1..*wc_idx] != "." {
                        log::info!("[WARN] DataCache get_list : invalid target {}", target);
                        None // Invalid syntax : if * is not the first character, then it is expected to be after a dot
                    } else {
                        let parent_path = &target[..*wc_idx-1];
                        if let Some(parent_arr) = self.root.pointer(&DataCache::target_to_pointer(parent_path)) {
                            if parent_arr.is_array() {
                                Some(parent_arr)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                };
                let suffix = &target[*wc_idx+1..];
                match parent_array {
                    Some(arr) => {
                        // This unwrap is safe because parent_array is always confirmed to be an array to become Some during construction
                        let parent_arr: &'b Vec<Value> = arr.as_array().unwrap();

                        if suffix.len() == 0 {
                            // Wildcard is the end => the parent itself, owned
                            parent_arr.iter().collect::<Vec<&'b Value>>()
                        } else if suffix.len() == 1 || &target[*wc_idx+1..*wc_idx+2] != "." {
                            log::info!("[WARN] DataCache get_list : invalid target {}", target);
                            // If the character after the wildcard not a dot, or if there is nothing else after the dot : invalid
                            Vec::new()
                        } else {
                            let suffix = &suffix[1..]; // Without the following dot
                            parent_arr.iter().map(|el| {
                                // We should not filter_map to preserve element count property of the wildcard on the parent
                                el.pointer(&DataCache::target_to_pointer(suffix)).unwrap_or(&Value::Null)
                            }).collect::<Vec<&'b Value>>()
                        }
                    },
                    None => Vec::new(),
                }
            },
            _ => {
                // More than 1 wildcard pattern is not supported : spec not yet indefined
                log::info!("[WARN] DataCache get_list : multiple wildcards pattern is not supported");
                Vec::new()
            }
        }
    }

    /// Match a pattern while storing captured named capture groups in data_cache
    pub fn match_regex(&mut self, regex: &str, source: &str) -> Result<bool, JsonDataCacheError> {
        match Regex::new(regex) {
            Ok(re) => {
                match re.captures(source) {
                    Some(captures) => {
                        for name_opt in re.capture_names() {
                            if let Some(name) = name_opt {
                                if self.options.reserved_cache_top_level_names.iter().map(|s| s.as_str()).any(|i| i == name) {
                                    return Err(format!("Capturing into the reserved variable {name} is not allowed").into());
                                }
                                if let Some(matched) = captures.name(name) {
                                    // Named capture detected => insert into data_cache
                                    self.insert(name, Value::String(matched.as_str().to_owned()));
                                }
                            }
                        }
                        Ok(true) // Matched
                    }
                    None => Ok(false),
                }
            },
            Err(_) => Err(format!("Invalid regex {}", regex).into()),
        }
    }

    /// Performs replacements of {$key} into mapped values from data_cache if key exists
    /// It uses Aho-Corasick algorithm for efficient multi-replacement, and works on streams (Vec<u8> does work, too)
    pub fn replace_with_data_cache<R, W>(
        &mut self,
        reader: R,
        writer: W
    ) -> Result<(), JsonDataCacheError>
    where
        R: io::Read,
        W: io::Write,
    {
        if !self.serialized_data.is_built {
            // Rebuild serialized data
            let (serialized, double_serialized) = JsonSerializer::serialize(&self.root, true);

            self.serialized_data.serialized = Some(serialized);
            self.serialized_data.double_serialized = double_serialized;

            // Build AC
            let mut keys_count = self.serialized_data.serialized.as_ref().unwrap().key_values.len();
            if let Some(double_serialized) = self.serialized_data.double_serialized.as_ref() {
                keys_count += double_serialized.key_values.len();
            }
            let mut patterns: Vec<String> = Vec::with_capacity(keys_count);
            let mut replacements: Vec<Rc<[u8]>> = Vec::with_capacity(keys_count);

            for (key, range) in &self.serialized_data.serialized.as_ref().unwrap().key_values {
                let formatted_key = format!("{{${key}}}");
                patterns.push(formatted_key);

                let actual_value = &self.serialized_data.serialized.as_ref().unwrap().data[range.start..range.end];
                replacements.push(actual_value.into());
            }
            if let Some(double_serialized) = self.serialized_data.double_serialized.as_ref() {
                for (key, range) in &double_serialized.key_values {
                    let formatted_key = format!("{{$${key}}}");
                    patterns.push(formatted_key);

                    let actual_value = &double_serialized.data[range.start..range.end];
                    replacements.push(actual_value.into());
                }
            }

            self.serialized_data.ac = Some(AhoCorasick::new(patterns)?);
            self.serialized_data.replacements = replacements;
            self.serialized_data.is_built = true;
        }

        let ac = self.serialized_data.ac.as_ref().unwrap();

        ac.try_stream_replace_all(reader, writer, &self.serialized_data.replacements)?;
        Ok(())
    }
}
