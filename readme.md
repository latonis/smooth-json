# smooth-json
![Publish Status](https://github.com/latonis/smooth-json/actions/workflows/publish.yml/badge.svg)

This crate allows for flattening JSON objects into objects ready for use in Parquet, CSV, or other or data data styles.


The flattening is similar to ElasticSearch's [ingestion flattening](https://www.elastic.co/guide/en/elasticsearch/reference/current/nested.html) or what would be needed for VAST's DB and Table [integrations](https://vastdata.com/platform/database).

## Features
- Flatten [serde_json](https://docs.rs/serde_json/latest/serde_json/)'s `Value` variants into structures suitable for use with applications that are expecting data or columnar data formats.

## Examples
```rust
use smooth_json::flatten;

let input: Value = json!({
    "a": [
        {
            "b": ["1"]
        }
    ]
});

let result: Value = flatten(&input, None);
/*
{
    "a.b": [1]
}
*/
```