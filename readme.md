# smooth-json
![Publish Status](https://github.com/latonis/smooth-json/actions/workflows/publish.yml/badge.svg)

This crate allows for flattening JSON objects into objects ready for use in Parquet, CSV, or other or data data styles.


The flattening is similar to ElasticSearch's [ingestion flattening](https://www.elastic.co/guide/en/elasticsearch/reference/current/nested.html) or what would be needed for VAST's DB and Table [integrations](https://vastdata.com/platform/database).

## Features
- Flatten [serde_json](https://docs.rs/serde_json/latest/serde_json/)'s `Value` variants into structures suitable for use with applications that are expecting data or columnar data formats.
- Pass a custom separator by instantiating a `Flattener` and passing the separator.
- 
## Examples
```rust
use smooth_json;
use serde_json::json;

fn main() {
    let flattener = smooth_json::Flattener::new();

    let example = json!({
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

    let flattened_example = flattener.flatten(&example);
    
    println!("{}", flattened_example);
    /*
    {
        "address.city": "London",
        "address.street": "10 Downing Street",
        "age": 43,
        "name": "John Doe",
        "phones": [
            "+44 1234567",
            "+44 2345678"
        ]
    }
    */
}
```

```rust
use serde_json::json;
use smooth_json;

fn main() {
    let flattener = smooth_json::Flattener{separator: "$"};

    let example = json!({
        "a": {
            "b": 1
        }});

    let flattened_example = flattener.flatten(&example);

    println!("{}", flattened_example);
    /*
    {
        "a$b": 1
    }
    */
}

```