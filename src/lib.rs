#![allow(unknown_lints)]
#![deny(missing_docs)]
#![deny(rustdoc::missing_doc_code_examples)]

//! smooth-json
//!
//! `smooth-json` provides a utility to flatten a `serde_json` `Value` into a flat `serde_json` `Object`
//! # Examples
//! ```
//! use smooth_json::Flattener;
//! ```

use serde_json::Map;
use serde_json::Value;
use serde_json::json;

/// Flattener is the main driver when flattening JSON
/// # Examples
/// ```
/// use smooth_json;
///
/// let flattener = smooth_json::Flattener { ..Default::default() };
/// ```
pub struct Flattener<'a> {
    /// Alternate separator used between keys when flattening
    /// # Examples
    /// ```
    /// use smooth_json;
    /// let flattener = smooth_json::Flattener { separator: "_", ..Default::default()};
    /// ```
    pub separator: &'a str,
    /// Opinionated flattening format that places values in an array if the object is nested inside an array
    /// # Examples
    /// ```
    /// use smooth_json;
    /// let flattener = smooth_json::Flattener { alt_array_flattening: true, ..Default::default()};
    /// ```
    pub alt_array_flattening: bool,
    /// Completely flatten JSON and keep array structure in the key when flattening
    /// # Examples
    /// ```
    /// use smooth_json;
    /// let flattener = smooth_json::Flattener { preserve_arrays: true, ..Default::default()};
    /// ```
    pub preserve_arrays: bool,
}

impl<'a> Default for Flattener<'a> {
    fn default() -> Self {
        Flattener {
            separator: ".",
            alt_array_flattening: false,
            preserve_arrays: false,
        }
    }
}

/// This implementation defines the core usage for the `Flattener` structure.
/// # Examples
/// ```
/// use smooth_json;
/// use serde_json::json;
///
/// let flattener = smooth_json::Flattener::new();
/// let example = json!({
///     "a": {
///         "b": "c"
///     }
///  });
///
/// let flattened_example = flattener.flatten(&example);
/// ```
impl<'a> Flattener<'a> {
    /// Returns a flattener with the default arguments
    /// # Examples
    /// ```
    /// use smooth_json;
    ///
    /// let flattener = smooth_json::Flattener::new();
    /// ```
    pub fn new() -> Self {
        Flattener {
            ..Default::default()
        }
    }

    /// Builds a composite key by combining a prefix and suffix with the configured separator.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix part of the key (can be empty string for root level)
    /// * `suffix` - The suffix part of the key
    ///
    /// # Returns
    ///
    /// A formatted key string with the separator inserted between prefix and suffix.
    fn build_key(&self, prefix: &str, suffix: &str) -> String {
        let mut key = String::with_capacity(prefix.len() + self.separator.len() + suffix.len());
        key.push_str(prefix);
        key.push_str(self.separator);
        key.push_str(suffix);
        key
    }

    /// Flattens JSON variants into a JSON object
    ///
    /// # Arguments
    ///
    /// * `json` - A serde_json Value to flatten
    ///
    /// # Examples
    /// ```
    /// use smooth_json;
    /// use serde_json::json;
    ///
    /// let flattener = smooth_json::Flattener::new();
    /// let example = json!({
    ///     "name": "John Doe",
    ///     "age": 43,
    ///     "address": {
    ///         "street": "10 Downing Street",
    ///         "city": "London"
    ///     },
    ///     "phones": [
    ///         "+44 1234567",
    ///         "+44 2345678"
    ///     ]
    ///  });
    ///
    /// let flattened_example = flattener.flatten(&example);
    /// ```
    pub fn flatten(&self, json: &Value) -> Value {
        let mut flattened_val = Map::<String, Value>::new();
        match json {
            Value::Array(obj_arr) => self.flatten_array(&mut flattened_val, "", obj_arr),
            Value::Object(obj_val) => self.flatten_object(&mut flattened_val, None, obj_val, false),
            _ => self.flatten_value(&mut flattened_val, "", json, false),
        }
        Value::Object(flattened_val)
    }

    fn flatten_object(
        &self,
        builder: &mut Map<String, Value>,
        identifier: Option<&str>,
        obj: &Map<String, Value>,
        arr: bool,
    ) {
        for (k, v) in obj {
            let expanded_identifier = match identifier {
                None => k.clone(),
                Some(id) => self.build_key(id, k),
            };

            match v {
                Value::Object(obj_val) => {
                    self.flatten_object(builder, Some(expanded_identifier.as_str()), obj_val, arr)
                }
                Value::Array(obj_arr) => {
                    self.flatten_array(builder, expanded_identifier.as_str(), obj_arr)
                }
                _ => self.flatten_value(builder, expanded_identifier.as_str(), v, arr),
            }
        }
    }

    fn flatten_array(&self, builder: &mut Map<String, Value>, identifier: &str, obj: &[Value]) {
        use std::fmt::Write;
        let mut index_buf = String::new();

        for (k, v) in obj.iter().enumerate() {
            write!(&mut index_buf, "{}", k).unwrap();
            let with_key = self.build_key(identifier, &index_buf);
            let current_identifier = if self.preserve_arrays {
                with_key.as_str()
            } else {
                identifier
            };

            match v {
                Value::Object(obj_val) => self.flatten_object(
                    builder,
                    Some(current_identifier),
                    obj_val,
                    self.alt_array_flattening,
                ),
                Value::Array(obj_arr) => self.flatten_array(builder, current_identifier, obj_arr),
                _ => self.flatten_value(builder, current_identifier, v, self.alt_array_flattening),
            }

            index_buf.clear();
        }
    }

    fn flatten_value(
        &self,
        builder: &mut Map<String, Value>,
        identifier: &str,
        obj: &Value,
        arr: bool,
    ) {
        let key = identifier.to_string();

        match builder.entry(key) {
            serde_json::map::Entry::Occupied(mut entry) => {
                let value = entry.get_mut();
                if let Some(array) = value.as_array_mut() {
                    array.push(obj.clone());
                } else {
                    let existing = value.clone();
                    *value = json!(vec![existing, obj.clone()]);
                }
            }
            serde_json::map::Entry::Vacant(entry) => {
                entry.insert(if arr {
                    json!(vec![obj.clone()])
                } else {
                    obj.clone()
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn serde_example() {
        let flattener = Flattener::new();
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

        let flat = flattener.flatten(&base);

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
        let flattener = Flattener::new();
        let base: Value = json!({
          "a": {
            "b": "c",
          },
          "a.b": "d",
        });
        let flat = flattener.flatten(&base);

        assert_eq!(
            flat,
            json!({
                "a.b": ["c", "d"],
            })
        );
    }

    #[test]
    fn collision_array() {
        let flattener = Flattener::new();
        let flattener_alt = Flattener {
            alt_array_flattening: true,
            ..Default::default()
        };

        let base: Value = json!({
          "a": [
            { "b": "c" },
            { "b": "d", "c": "e" },
            [35],
          ],
          "a.b": "f",
        });

        let flat = flattener.flatten(&base);
        let flat_alt = flattener_alt.flatten(&base);

        assert_eq!(
            flat,
            json!({
                "a.b": ["c", "d", "f"],
                "a.c": "e",
                "a": 35,
            })
        );

        assert_eq!(
            flat_alt,
            json!({
                "a.b": ["c", "d", "f"],
                "a.c": ["e"],
                "a": [35],
            })
        );
    }

    #[test]
    fn nested_arrays() {
        let flattener = Flattener::new();
        let flattener_alt = Flattener {
            alt_array_flattening: true,
            ..Default::default()
        };

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
        let flat = flattener.flatten(&base);
        let flat_alt = flattener_alt.flatten(&base);

        assert_eq!(
            flat,
            json!({
                "a": ["b", "c", "f", "g", "k", "l"],
                "a.d": ["e", "j"],
                "a.h": "i",
            })
        );

        assert_eq!(
            flat_alt,
            json!({
                "a": ["b", "c", "f", "g", "k", "l"],
                "a.d": ["e", "j"],
                "a.h": ["i"],
            })
        );
    }

    #[test]
    fn nested_arrays_and_objects() {
        let flattener = Flattener::new();
        let flattener_alt = Flattener {
            alt_array_flattening: true,
            ..Default::default()
        };

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
        let flat = flattener.flatten(&base);
        let flat_alt = flattener_alt.flatten(&base);

        assert_eq!(
            flat,
            json!({
                "a": ["b", "c", "d", "l", "m"],
                "a.e": ["f", "g", "j"],
                "a.h": "i",
                "a.e.z": "y",
            })
        );

        assert_eq!(
            flat_alt,
            json!({
                "a": ["b", "c", "d", "l", "m"],
                "a.e": ["f", "g", "j"],
                "a.h": ["i"],
                "a.e.z": ["y"],
            })
        )
    }

    #[test]
    fn custom_separator() {
        let flattener = Flattener {
            separator: "$",
            ..Default::default()
        };

        let input: Value = json!({
        "a": {
            "b": 1
        }});

        let result: Value = flattener.flatten(&input);
        assert_eq!(
            result,
            json!({
                "a$b": 1
            })
        );
    }
    #[test]
    fn object() {
        let flattener = Flattener::new();

        let input: Value = json!({
            "a": {
                "b": "1",
                "c": "2",
                "d": "3"
            }
        });

        let result: Value = flattener.flatten(&input);
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
        let flattener = Flattener::new();

        let input: Value = json!({
            "a": [
                {"b": "1"},
                {"b": "2"},
                {"b": "3"},
            ]
        });

        let result: Value = flattener.flatten(&input);
        assert_eq!(
            result,
            json!({
                "a.b": ["1", "2", "3"]
            })
        );
    }

    #[test]
    fn array_preserve() {
        let flattener = Flattener {
            preserve_arrays: true,
            ..Default::default()
        };

        let input: Value = json!({
            "a": [
                {"b": "1"},
                {"b": "2"},
                {"b": "3"},
            ]
        });

        let result: Value = flattener.flatten(&input);
        assert_eq!(
            result,
            json!({
                "a.0.b": "1",
                "a.1.b": "2",
                "a.2.b": "3"
            })
        );
    }

    #[test]
    fn array_no_collision() {
        let flattener = Flattener::new();
        let flattener_alt = Flattener {
            alt_array_flattening: true,
            ..Default::default()
        };

        let input: Value = json!({
            "a": [
                {"b": ["1"]}
            ]
        });

        let flat: Value = flattener.flatten(&input);
        let flat_alt = flattener_alt.flatten(&input);

        assert_eq!(
            flat,
            json!({
                "a.b": "1"
            })
        );

        assert_eq!(
            flat_alt,
            json!({
                "a.b": ["1"]
            })
        );
    }

    // its allowed https://ecma-international.org/publications-and-standards/standards/ecma-404/
    #[test]
    fn arr_no_key() {
        let flattener = Flattener::new();

        let input: Value = json!(["a", "b"]);

        let result: Value = flattener.flatten(&input);

        assert_eq!(result, json!({"": ["a", "b"]}));
    }

    // its allowed https://ecma-international.org/publications-and-standards/standards/ecma-404/
    #[test]
    fn arr_empty_key() {
        let flattener = Flattener::new();

        let input: Value = json!({
            "": [
                "a",
                "b",
                {"b": ["1"]}
            ],
        });
        let result: Value = flattener.flatten(&input);

        assert_eq!(result, json!({"": ["a", "b"], ".b": "1"}));
    }

    #[test]
    fn only_value() {
        let flattener = Flattener::new();

        let input: Value = json!("abc");
        let result: Value = flattener.flatten(&input);

        assert_eq!(result, json!({"": "abc"}));
    }

    #[test]
    fn nested_array_preserve() {
        let flattener = Flattener {
            preserve_arrays: true,
            ..Default::default()
        };

        let input: Value = json!({
        "a": [
                    "b",
                    ["c", "d"],
                    { "e": ["f", "g"] },
                    [
                        { "h": "i" },
                        { "e": ["j", { "z": "y" }] }
                    ],
                    ["l"],
                    "m"
                 ]
        });

        let result: Value = flattener.flatten(&input);

        assert_eq!(
            result,
            json!({
              "a.0": "b",
              "a.1.0": "c",
              "a.1.1": "d",
              "a.2.e.0": "f",
              "a.2.e.1": "g",
              "a.3.0.h": "i",
              "a.3.1.e.0": "j",
              "a.3.1.e.1.z": "y",
              "a.4.0": "l",
              "a.5": "m"
            })
        )
    }

    #[test]
    fn keys_with_separator() {
        let flattener = Flattener::new();

        let input: Value = json!({
            "a.b": "value1",
            "a": {
                "b": "value2"
            }
        });

        let result = flattener.flatten(&input);

        // Both "a.b" literal key and nested "a.b" should result in collision as an array
        // Order may vary based on map iteration
        match &result["a.b"] {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 2);
                assert!(arr.contains(&Value::String("value1".to_string())));
                assert!(arr.contains(&Value::String("value2".to_string())));
            }
            _ => panic!("Expected array for collided keys"),
        }
    }

    #[test]
    fn null_values() {
        let flattener = Flattener::new();

        let input: Value = json!({
            "a": null,
            "b": {
                "c": null,
                "d": "value"
            },
            "e": [null, "text", null]
        });

        let result = flattener.flatten(&input);

        assert_eq!(
            result,
            json!({
                "a": null,
                "b.c": null,
                "b.d": "value",
                "e": [null, "text", null]
            })
        );
    }

    #[test]
    fn collision_stress_test() {
        let flattener = Flattener::new();

        // Create a structure with many collisions to stress test the Entry API
        let input: Value = json!({
            "x": "value1",
            "data": [
                { "x": "value2" },
                { "x": "value3" },
                { "x": "value4" },
                { "x": "value5" }
            ]
        });

        let result = flattener.flatten(&input);

        // Should have two keys: "x" and "data.x", both should be arrays
        assert!(result.get("x").is_some());
        assert!(result.get("data.x").is_some());

        match &result["x"] {
            Value::String(s) => assert_eq!(s, "value1"),
            _ => panic!("Expected string for 'x'"),
        }

        match &result["data.x"] {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 4);
                assert_eq!(arr[0], Value::String("value2".to_string()));
                assert_eq!(arr[1], Value::String("value3".to_string()));
                assert_eq!(arr[2], Value::String("value4".to_string()));
                assert_eq!(arr[3], Value::String("value5".to_string()));
            }
            _ => panic!("Expected array for 'data.x'"),
        }
    }
}
