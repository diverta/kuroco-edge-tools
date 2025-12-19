use json_data_cache::json_serializer::JsonSerializer;
use serde_json::json;

#[test]
fn serializer_test() {
    let value = json!({
        "parent": {
            "child_int": 12.5,
            "child_string": "toy",
            "child_bool": false,
            "child_map": {
                "one": 1,
                "two": "second"
            },
            "child_arr": [
                "one", 2, false, null
            ]
        },
        "uncle": "sam",
        "arr_aunt": [
            "sibling_string",
            null
        ]
    });
    let (serialized, double_serialized) = JsonSerializer::serialize(&value, true);

    // Check for serialized
    for (key, expected) in [
        ("parent", r#"{"child_int":12.5,"child_string":"toy","child_bool":false,"child_map":{"one":1,"two":"second"},"child_arr":["one",2,false,null]}"#),
        ("parent.child_int", r#"12.5"#),
        ("parent.child_string", r#"toy"#),
        ("parent.child_bool", r#"false"#),
        ("parent.child_map", r#"{"one":1,"two":"second"}"#),
        ("parent.child_map.one", r#"1"#),
        ("parent.child_map.two", r#"second"#),
        ("parent.child_arr", r#"["one",2,false,null]"#),
        ("parent.child_arr.0", r#"one"#),
        ("parent.child_arr.1", r#"2"#),
        ("parent.child_arr.2", r#"false"#),
        ("parent.child_arr.3", r#"null"#),
        ("uncle", r#"sam"#),
        ("arr_aunt", r#"["sibling_string",null]"#),
        ("arr_aunt.0", r#"sibling_string"#),
        ("arr_aunt.1", r#"null"#),
    ] {
        assert!(serialized.key_values.contains_key(key));
        let range = serialized.key_values.get(key).unwrap();
        let serialized_string = String::from_utf8((&serialized.data[range.start..range.end]).to_vec());
        assert!(serialized_string.is_ok());
        let serialized_value = serialized_string.unwrap();
        assert_eq!(expected, &serialized_value);
    }

    // Check for double_serialized
    assert!(double_serialized.is_some());
    let double_serialized = double_serialized.unwrap();
    for (key, expected) in [
        ("parent", r#"{\"child_int\":12.5,\"child_string\":\"toy\",\"child_bool\":false,\"child_map\":{\"one\":1,\"two\":\"second\"},\"child_arr\":[\"one\",2,false,null]}"#),
        ("parent.child_int", r#"12.5"#),
        ("parent.child_string", r#"toy"#),
        ("parent.child_bool", r#"false"#),
        ("parent.child_map", r#"{\"one\":1,\"two\":\"second\"}"#),
        ("parent.child_map.one", r#"1"#),
        ("parent.child_map.two", r#"second"#),
        ("parent.child_arr", r#"[\"one\",2,false,null]"#),
        ("parent.child_arr.0", r#"one"#),
        ("parent.child_arr.1", r#"2"#),
        ("parent.child_arr.2", r#"false"#),
        ("parent.child_arr.3", r#"null"#),
        ("uncle", r#"sam"#),
        ("arr_aunt", r#"[\"sibling_string\",null]"#),
        ("arr_aunt.0", r#"sibling_string"#),
        ("arr_aunt.1", r#"null"#),
    ] {
        assert!(double_serialized.key_values.contains_key(key));
        let range = double_serialized.key_values.get(key).unwrap();
        let double_serialized_string = String::from_utf8((&double_serialized.data[range.start..range.end]).to_vec());
        assert!(double_serialized_string.is_ok());
        let double_serialized_value = double_serialized_string.unwrap();
        assert_eq!(expected, &double_serialized_value);
    }
}