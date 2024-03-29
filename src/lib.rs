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

use serde_json::json;
use serde_json::Map;
use serde_json::Value;

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
            Value::Array(obj_arr) => {
                self.flatten_array(&mut flattened_val, &"".to_string(), obj_arr)
            }
            Value::Object(obj_val) => self.flatten_object(&mut flattened_val, None, obj_val, false),
            _ => self.flatten_value(&mut flattened_val, &"".to_string(), json, false),
        }
        Value::Object(flattened_val)
    }

    fn flatten_object(
        &self,
        builder: &mut Map<String, Value>,
        identifier: Option<&String>,
        obj: &Map<String, Value>,
        arr: bool,
    ) {
        for (k, v) in obj {
            let expanded_identifier = identifier.map_or_else(
                || k.clone(),
                |identifier| format!("{identifier}{}{k}", self.separator),
            );

            match v {
                Value::Object(obj_val) => {
                    self.flatten_object(builder, Some(&expanded_identifier), obj_val, arr)
                }
                Value::Array(obj_arr) => self.flatten_array(builder, &expanded_identifier, obj_arr),
                _ => self.flatten_value(builder, &expanded_identifier, v, arr),
            }
        }
    }

    fn flatten_array(
        &self,
        builder: &mut Map<String, Value>,
        identifier: &String,
        obj: &Vec<Value>,
    ) {
        for (k, v) in obj.iter().enumerate() {
            let with_key = format!("{identifier}{}{k}", self.separator);
            match v {
                Value::Object(obj_val) => self.flatten_object(
                    builder,
                    Some(if self.preserve_arrays {
                        &with_key
                    } else {
                        identifier
                    }),
                    obj_val,
                    self.alt_array_flattening,
                ),
                Value::Array(obj_arr) => self.flatten_array(
                    builder,
                    if self.preserve_arrays {
                        &with_key
                    } else {
                        identifier
                    },
                    obj_arr,
                ),
                _ => self.flatten_value(
                    builder,
                    if self.preserve_arrays {
                        &with_key
                    } else {
                        identifier
                    },
                    v,
                    self.alt_array_flattening,
                ),
            }
        }
    }

    fn flatten_value(
        &self,
        builder: &mut Map<String, Value>,
        identifier: &String,
        obj: &Value,
        arr: bool,
    ) {
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
}
