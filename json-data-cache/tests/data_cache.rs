use std::io::BufWriter;

use json_data_cache::{DataCache, DataCacheOptions};
use serde_json::json;

#[test]
fn data_cache() {
    let mut data_cache = DataCache::new(DataCacheOptions::default());
    
    data_cache.insert("basic_key", json!("basic_value"));
    assert_eq!(data_cache.root, json!({"basic_key": "basic_value"}));
    assert_eq!(data_cache.get("basic_key"), Some(&json!("basic_value")));

    data_cache.insert("a.b.c", json!("my_c_value"));
    data_cache.insert("a.b", json!({"d": "my_d_value"}));
    assert_eq!(data_cache.root, json!({"basic_key": "basic_value", "a": {"b": {"c": "my_c_value", "d": "my_d_value"}}}));
    assert_eq!(data_cache.get("a"), Some(&json!({"b": {"c": "my_c_value", "d": "my_d_value"}})));
    assert_eq!(data_cache.get("a.b"), Some(&json!({"c": "my_c_value", "d": "my_d_value"})));
    assert_eq!(data_cache.get("a.b.c"), Some(&json!("my_c_value")));
    assert_eq!(data_cache.get("a.b.d"), Some(&json!("my_d_value")));
    assert_eq!(data_cache.get("a.b.e"), None);

    data_cache.insert("a.b", json!("overwrite_string"));
    assert_eq!(data_cache.root, json!({"basic_key": "basic_value", "a": {"b": "overwrite_string"}}));
    assert_eq!(data_cache.get("a.b"), Some(&json!("overwrite_string")));

    data_cache.insert("basic_key", json!({"nested_key": "nested_value"}));
    assert_eq!(data_cache.root, json!({"basic_key": {"nested_key": "nested_value"}, "a": {"b": "overwrite_string"}}));
    assert_eq!(data_cache.get("basic_key"), Some(&json!({"nested_key": "nested_value"})));
    assert_eq!(data_cache.get("basic_key.nested_key"), Some(&json!("nested_value")));

    data_cache.insert("a.my_arr.", json!("first_el"));
    data_cache.insert("a.my_arr.", json!("second_el"));
    assert_eq!(data_cache.root, json!({"basic_key": {"nested_key": "nested_value"}, "a": {"b": "overwrite_string", "my_arr": ["first_el", "second_el"]}}));
    assert_eq!(data_cache.get("a"), Some(&json!({"b": "overwrite_string", "my_arr": ["first_el", "second_el"]})));
    assert_eq!(data_cache.get("a.my_arr"), Some(&json!(["first_el", "second_el"])));
    assert_eq!(data_cache.get("a.my_arr.0"), Some(&json!("first_el")));
    assert_eq!(data_cache.get("a.my_arr.1"), Some(&json!("second_el")));
    assert_eq!(data_cache.get("a.my_arr.2"), None);

    data_cache.insert("a.b.c", json!("my_c_value"));
    data_cache.insert("a.b", json!({"d": "my_d_value"}));
    assert_eq!(data_cache.root, json!({"basic_key": {"nested_key": "nested_value"}, "a": {"b": {"c": "my_c_value", "d": "my_d_value"}, "my_arr": ["first_el", "second_el"]}}));

    data_cache.merge(json!({"a": {"b": {"e": "my_e_value"}}}));
    assert_eq!(data_cache.root, json!(
        {
            "basic_key": {"nested_key": "nested_value"},
            "a": {
                "b": {"c": "my_c_value", "d": "my_d_value", "e": "my_e_value"},
                "my_arr": ["first_el", "second_el"]
            }
        }
    ));

    assert_eq!(data_cache.as_string_values_map().len(), 10);

    // All nodes must have string representation, not only string leafs
    assert_eq!(
        data_cache.as_string_values_map().get("a"),
        Some(&String::from(r#"{"b":{"c":"my_c_value","d":"my_d_value","e":"my_e_value"},"my_arr":["first_el","second_el"]}"#))
    );
    assert_eq!(
        data_cache.as_string_values_map().get("a.b"),
        Some(&String::from(r#"{"c":"my_c_value","d":"my_d_value","e":"my_e_value"}"#))
    );
    assert_eq!(
        data_cache.as_string_values_map().get("a.my_arr"),
        Some(&String::from(r#"["first_el","second_el"]"#))
    );

    // Test replacements
    for (input, replacement) in [
        ("{$basic_key.nested_key}", "nested_value"),
        ("{$a.b}", r#"{"c":"my_c_value","d":"my_d_value","e":"my_e_value"}"#),
        ("{$$basic_key.nested_key}", r#"nested_value"#),
        ("{$$a.b}", r#"{\"c\":\"my_c_value\",\"d\":\"my_d_value\",\"e\":\"my_e_value\"}"#)
    ] {
        let reader = String::from(input);
        let mut writer = BufWriter::new(Vec::new());
        assert!(data_cache.replace_with_data_cache(reader.as_bytes(), &mut writer).is_ok());
        let writer_string = String::from_utf8(writer.buffer().to_vec()).unwrap();
        assert_eq!(&writer_string, replacement);
    }
}
