// smooth-json/benches/flatten.rs
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use serde_json::{json, Map, Value};
use smooth_json::Flattener;

/// Build a nested object of the given depth and breadth. Example shape:
/// { "k0": { "k0": ... }, "k1": { ... }, ... }
fn make_nested(depth: usize, breadth: usize) -> Value {
    if depth == 0 {
        json!("leaf")
    } else {
        let mut obj = Map::new();
        for i in 0..breadth {
            let key = format!("k{}", i);
            obj.insert(key, make_nested(depth - 1, breadth));
        }
        Value::Object(obj)
    }
}

/// Build an array with `count` objects that all contain the same key `x`.
/// This is useful to produce many collisions once flattened.
fn make_collision_array(count: usize) -> Value {
    let mut arr = Vec::with_capacity(count);
    for i in 0..count {
        arr.push(json!({ "x": format!("value{}", i) }));
    }
    Value::Array(arr)
}

fn bench_flatten_inputs(c: &mut Criterion) {
    let mut group = c.benchmark_group("flatten_inputs");
    group.throughput(Throughput::Elements(1));

    // Small / medium / large inputs intended to exercise depth and breadth
    let inputs = vec![
        ("small", make_nested(2, 3)),
        ("medium", make_nested(4, 4)),
        ("large", make_nested(6, 4)),
    ];

    for (name, input) in inputs.into_iter() {
        // Default
        group.bench_with_input(BenchmarkId::new("default", name), &input, |b, v| {
            let fl = Flattener::new();
            b.iter(|| {
                let _ = fl.flatten(black_box(v));
            })
        });

        // preserve_arrays = true
        group.bench_with_input(BenchmarkId::new("preserve_arrays", name), &input, |b, v| {
            let fl = Flattener {
                preserve_arrays: true,
                ..Default::default()
            };
            b.iter(|| {
                let _ = fl.flatten(black_box(v));
            })
        });

        // alt_array_flattening = true
        group.bench_with_input(BenchmarkId::new("alt_array_flattening", name), &input, |b, v| {
            let fl = Flattener {
                alt_array_flattening: true,
                ..Default::default()
            };
            b.iter(|| {
                let _ = fl.flatten(black_box(v));
            })
        });
    }

    group.finish();
}

fn bench_collision_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("flatten_collisions");
    group.throughput(Throughput::Elements(1));

    // Test a range of collision sizes
    for &n in &[10usize, 100, 1000] {
        let input = json!({
            "a": make_collision_array(n),
            // also include a direct key that will collide with nested values when flattened
            "a.x": "direct"
        });

        group.bench_with_input(BenchmarkId::new("collisions_default", n), &input, |b, v| {
            let fl = Flattener::new();
            b.iter(|| {
                let _ = fl.flatten(black_box(v));
            })
        });

        // With alt_array_flattening enabled (different aggregation behavior)
        group.bench_with_input(BenchmarkId::new("collisions_alt_flatten", n), &input, |b, v| {
            let fl = Flattener {
                alt_array_flattening: true,
                ..Default::default()
            };
            b.iter(|| {
                let _ = fl.flatten(black_box(v));
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_flatten_inputs, bench_collision_cases);
criterion_main!(benches);