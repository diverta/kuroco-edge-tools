use std::io::BufWriter;

use json_data_cache::{DataCache, DataCacheOptions};
use serde_json::{Value, json};

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

    // Special case : setting a property to an array of objects will set it to each object
    data_cache.insert("array_of_objects", json!([
        {"k1":"v1"},
        {"k2":"v2"},
        {"nested": {"original_child": "original_child_value"}},
        "non_object"
    ]));
    data_cache.insert("array_of_objects.newkey", json!("newval"));
    assert_eq!(data_cache.get("array_of_objects"), Some(&json!([
        {"k1":"v1", "newkey":"newval"},
        {"k2":"v2", "newkey":"newval"},
        {"nested": {"original_child": "original_child_value"}, "newkey":"newval"},
        "non_object" // Unaffected
    ])));
    data_cache.insert("array_of_objects.nested", json!({"new_child": "new_child_value"}));
    assert_eq!(data_cache.get("array_of_objects"), Some(&json!([
        {"k1":"v1", "newkey":"newval", "nested": {"new_child": "new_child_value"}},
        {"k2":"v2", "newkey":"newval", "nested": {"new_child": "new_child_value"}},
        {"nested": {"original_child": "original_child_value", "new_child": "new_child_value"}, "newkey":"newval"},
        "non_object" // Unaffected
    ])));

    // Special case of a special case : setting a property to an array of objects with the value being another array
    // will distribute value array's contents over each object's key matching the property
    // If either parent or value array is too long, cuts off the longest array to match the size of the smallest
    data_cache.insert("array_of_objects", json!([
        {"k1":"v1"},
        {"k2":"v2"},
        {"nested": {"original_child": "original_child_value"}},
        "non_object"
    ]));
    data_cache.insert("array_of_objects.newkey", json!(["newval1", "newval2"]));
    assert_eq!(data_cache.get("array_of_objects"), Some(&json!([
        {"k1":"v1", "newkey":"newval1"},
        {"k2":"v2", "newkey":"newval2"},
        {"nested": {"original_child": "original_child_value"}},
        "non_object" // Unaffected
    ])));
    data_cache.insert("array_of_objects.newkey", json!(["newval1", "newval2", "newval3", "newval4", "newval5"]));
    assert_eq!(data_cache.get("array_of_objects"), Some(&json!([
        {"k1":"v1", "newkey":"newval1"},
        {"k2":"v2", "newkey":"newval2"},
        {"nested": {"original_child": "original_child_value"}, "newkey":"newval3"},
        "non_object" // Unaffected
    ])));
    data_cache.insert("array_of_objects.nested", json!([
        {"new_child": "new_child_value1"},
        {"new_child": "new_child_value2"},
        {"new_child": "new_child_value3"}
    ]));
    assert_eq!(data_cache.get("array_of_objects"), Some(&json!([
        {"k1":"v1", "newkey":"newval1", "nested": {"new_child": "new_child_value1"}},
        {"k2":"v2", "newkey":"newval2", "nested": {"new_child": "new_child_value2"}},
        {"nested": {"original_child": "original_child_value", "new_child": "new_child_value3"}, "newkey":"newval3"},
        "non_object" // Unaffected
    ])));

    // Test replacements
    for (input, replacement) in [
        ("{$basic_key.nested_key}", "nested_value"),
        ("{$a.b}", r#"{"c":"my_c_value","d":"my_d_value","e":"my_e_value"}"#),
        ("{$$basic_key.nested_key}", r#"nested_value"#),
        ("{$$a.b}", r#"{\"c\":\"my_c_value\",\"d\":\"my_d_value\",\"e\":\"my_e_value\"}"#)
    ] {
        let reader = String::from(input);
        let mut writer = BufWriter::new(Vec::new());
        let data_cache_ref = &mut data_cache;
        assert!(data_cache_ref.replace_with_data_cache(reader.as_bytes(), &mut writer).is_ok());
        let writer_string = String::from_utf8(writer.buffer().to_vec()).unwrap();
        assert_eq!(&writer_string, replacement);
    }
}

#[test]
fn data_cache_get_list_test() {
    let mut data_cache = DataCache::new(DataCacheOptions::default());
    
    data_cache.insert("list", json!([
        {
            "id": 1,
            "name": "first"
        },
        {
            "id": 2,
            "name": "second"
        },
        {
            "id": 3,
            "name": "third"
        },
    ]));

    // Wildcard allows to build arrays
    assert_eq!(data_cache.get_list("list.*"), Vec::from([
        &json!({
            "id": 1,
            "name": "first"
        }),
        &json!({
            "id": 2,
            "name": "second"
        }),
        &json!({
            "id": 3,
            "name": "third"
        }),
    ]));
    assert_eq!(data_cache.get_list("list.*"), data_cache.get("list")
        .map(|v| v.as_array().unwrap().iter().collect())
        .unwrap_or(Vec::new())); // Technically this is a shortcut
    
    assert_eq!(data_cache.get_list("list.*.id"), Vec::from([
        &json!(1),
        &json!(2),
        &json!(3)
    ]));
    assert_eq!(data_cache.get_list("list.*.name"), Vec::from([
        &json!("first"),
        &json!("second"),
        &json!("third"),
    ]));

    assert_eq!(data_cache.get_list("*"), Vec::<&Value>::new()); // With DataCache, the top level JSON structure is always an object

    // For now, only one wildcard is supported
    assert_eq!(data_cache.get_list("list.*.*"), Vec::<&Value>::new());
    assert_eq!(data_cache.get_list("list*"), Vec::<&Value>::new());
}