use serde_json::json;
use serde_json::Map;
use serde_json::Value;

pub fn flatten(json: &Value, separator: Option<&str>) -> Value {
    let mut flattened_val = Map::<String, Value>::new();
    match json {
        Value::Array(_) => {
            flatten_array(&mut flattened_val, "", json.as_array().unwrap(), separator)
        }
        Value::Object(obj_val) => {
            flatten_object(&mut flattened_val, None, obj_val, false, separator)
        }
        _ => {}
    }
    Value::Object(flattened_val)
}

fn flatten_object(
    builder: &mut Map<String, Value>,
    identifier: Option<&str>,
    obj: &Map<String, Value>,
    arr: bool,
    separator: Option<&str>,
) {
    for (k, v) in obj {
        let sep = separator.unwrap_or(".");
        let expanded_identifier =
            identifier.map_or_else(|| k.clone(), |identifier| format!("{identifier}{sep}{k}"));

        match v {
            Value::Object(obj_val) => flatten_object(
                builder,
                Some(expanded_identifier.as_str()),
                obj_val,
                arr,
                separator,
            ),
            Value::Array(obj_arr) => {
                flatten_array(builder, expanded_identifier.as_str(), obj_arr, separator)
            }
            _ => flatten_value(builder, expanded_identifier.as_str(), v, arr),
        }
    }
}

fn flatten_array(
    builder: &mut Map<String, Value>,
    identifier: &str,
    obj: &Vec<Value>,
    separator: Option<&str>,
) {
    for v in obj {
        match v {
            Value::Object(obj_val) => {
                flatten_object(builder, Some(identifier), obj_val, false, separator)
            }
            Value::Array(obj_arr) => flatten_array(builder, identifier, obj_arr, separator),
            _ => flatten_value(builder, identifier, v, true),
        }
    }
}

fn flatten_value(builder: &mut Map<String, Value>, identifier: &str, obj: &Value, arr: bool) {
    if let Some(v) = builder.get_mut(identifier) {
        if let Some(arr) = v.as_array_mut() {
            arr.push(obj.clone());
        } else {
            let new_val = json!(vec![v, obj]);
            builder.remove(identifier);
            builder.insert(identifier.to_string(), new_val);
        }
    } else {
        builder.insert(
            identifier.to_string(),
            if arr {
                json!(vec![obj.clone()])
            } else {
                obj.clone()
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn serde_example() {
        let base: Value = json!({
            "name": "John Doe",
            "age": 43,
            "address": {
                "street": "10 Downing Street",
                "city": "London"
            },
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        });

        let flat = flatten(&base, None);
        
        assert_eq!(
            flat,
            json!({
                "name": "John Doe",
                "age": 43,
                "address.street": "10 Downing Street",
                "address.city": "London",
                "phones": [
                    "+44 1234567",
                    "+44 2345678"
                ]
            })
        );
    }

    #[test]
    fn collision_object() {
        let base: Value = json!({
          "a": {
            "b": "c",
          },
          "a.b": "d",
        });
        let flat = flatten(&base, None);

        assert_eq!(
            flat,
            json!({
                "a.b": ["c", "d"],
            })
        );
    }

    #[test]
    fn collision_array() {
        let base: Value = json!({
          "a": [
            { "b": "c" },
            { "b": "d", "c": "e" },
            [35],
          ],
          "a.b": "f",
        });
        let flat = flatten(&base, None);

        assert_eq!(
            flat,
            json!({
                "a.b": ["c", "d", "f"],
                "a.c": "e",
                "a": [35],
            })
        );
    }

    #[test]
    fn nested_arrays() {
        let base: Value = json!({
          "a": [
            ["b", "c"],
            { "d": "e" },
            ["f", "g"],
            [
                { "h": "i" },
                { "d": "j" },
            ],
            ["k", "l"],
          ]
        });
        let flat = flatten(&base, None);

        assert_eq!(
            flat,
            json!({
                "a": ["b", "c", "f", "g", "k", "l"],
                "a.d": ["e", "j"],
                "a.h": "i",
            })
        );
    }

    #[test]
    fn nested_arrays_and_objects() {
        let base: Value = json!({
          "a": [
            "b",
            ["c", "d"],
            { "e": ["f", "g"] },
            [
                { "h": "i" },
                { "e": ["j", { "z": "y" }] },
            ],
            ["l"],
            "m",
          ]
        });
        let flat = flatten(&base, None);

        assert_eq!(
            flat,
            json!({
                "a": ["b", "c", "d", "l", "m"],
                "a.e": ["f", "g", "j"],
                "a.h": "i",
                "a.e.z": "y",
            })
        );
    }

    #[test]
    fn custom_separator() {
        let input: Value = json!({
        "a": {
            "b": 1
        }});

        let result: Value = flatten(&input, Some("$"));
        assert_eq!(
            result,
            json!({
                "a$b": 1
            })
        );
    }
    #[test]
    fn object() {
        let input: Value = json!({
            "a": {
                "b": "1",
                "c": "2",
                "d": "3"
            }
        });

        let result: Value = flatten(&input, None);
        assert_eq!(
            result,
            json!({
                "a.b": "1",
                "a.c": "2",
                "a.d": "3"
            })
        );
    }

    #[test]
    fn array() {
        let input: Value = json!({
            "a": [
                {"b": "1"},
                {"b": "2"},
                {"b": "3"},
            ]
        });

        let result: Value = flatten(&input, None);
        assert_eq!(
            result,
            json!({
                "a.b": ["1", "2", "3"]
            })
        );
    }

    #[test]
    fn array_no_collision() {
        let input: Value = json!({
            "a": [
                {"b": ["1"]}
            ]
        });

        let result: Value = flatten(&input, None);
        assert_eq!(
            result,
            json!({
                "a.b": ["1"]
            })
        );
    }

    // its allowed https://ecma-international.org/publications-and-standards/standards/ecma-404/
    #[test]
    fn arr_no_key() {
        let input: Value = json!(["a", "b"]);

        let result: Value = flatten(&input, None);

        assert_eq!(result, json!({"": ["a", "b"]}));
    }

    // its allowed https://ecma-international.org/publications-and-standards/standards/ecma-404/
    #[test]
    fn arr_empty_key() {
        let input: Value = json!({
            "": [
                "a",
                "b",
                {"b": ["1"]}
            ],
        });
        let result: Value = flatten(&input, None);

        assert_eq!(result, json!({"": ["a", "b"], ".b": ["1"]}));
    }
}
