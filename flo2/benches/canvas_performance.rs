use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use flow_between::scenery::document::canvas::*;
use std::time::{Duration};

fn test_shape_type() -> ShapeType {
    ShapeType::new("flowbetween::benchmark::rectangle")
}

fn create_test_rectangle(x: f32, y: f32) -> CanvasShape {
    CanvasShape::Rectangle(CanvasRectangle {
        min: CanvasPoint { x, y },
        max: CanvasPoint {
            x: x + 10.0,
            y: y + 10.0,
        },
    })
}

/// Creates default properties for a benchmark shape
fn create_test_properties(index: usize) -> Vec<(CanvasPropertyId, CanvasProperty)> {
    vec![
        (CanvasPropertyId::new("benchmark::id"), CanvasProperty::Int(index as i64)),
        (CanvasPropertyId::new("benchmark::opacity"), CanvasProperty::Float(0.8)),
        (CanvasPropertyId::new("benchmark::layer_depth"), CanvasProperty::Float(1.0)),
    ]
}

/// Creates a canvas with the specified number of layers and shapes per layer
fn create_populated_canvas(num_layers: usize, shapes_per_layer: usize) -> SqliteCanvas {
    let mut canvas = SqliteCanvas::new_in_memory().unwrap();

    // Create layers
    let layer_ids: Vec<CanvasLayerId> = (0..num_layers).map(|_| CanvasLayerId::new()).collect();

    for layer_id in &layer_ids {
        canvas.add_layer(*layer_id, None).unwrap();
    }

    // Add shapes to each layer
    for layer_id in &layer_ids {
        for i in 0..shapes_per_layer {
            let shape_id = CanvasShapeId::new();
            let x        = (i % 100) as f32 * 15.0;
            let y        = (i / 100) as f32 * 15.0;
            let shape = create_test_rectangle(x, y);

            canvas
                .add_shape(shape_id, test_shape_type(), shape)
                .unwrap();
            canvas
                .set_shape_parent(shape_id, CanvasShapeParent::Layer(*layer_id, Duration::from_nanos(0)))
                .unwrap();
            canvas
                .set_shape_properties(shape_id, create_test_properties(i))
                .unwrap();
        }
    }

    canvas
}

/// Benchmark querying shapes on a layer with varying numbers of shapes
fn bench_query_shapes_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_shapes_on_layer_scaling");

    // Test with increasing numbers of shapes on a single layer
    for num_shapes in [100, 500, 1000, 2500, 5000, 10000].iter() {
        group.throughput(Throughput::Elements(*num_shapes as u64));

        let mut canvas = create_populated_canvas(1, *num_shapes);
        let layer_id = CanvasLayerId::new();
        canvas.add_layer(layer_id, None).unwrap();

        // Add all shapes to the benchmark layer
        for i in 0..*num_shapes {
            let shape_id = CanvasShapeId::new();
            let x        = (i % 100) as f32 * 15.0;
            let y        = (i / 100) as f32 * 15.0;
            let shape    = create_test_rectangle(x, y);

            canvas
                .add_shape(shape_id, test_shape_type(), shape)
                .unwrap();
            canvas
                .set_shape_parent(shape_id, CanvasShapeParent::Layer(layer_id, Duration::from_nanos(0)))
                .unwrap();
            canvas
                .set_shape_properties(shape_id, create_test_properties(i))
                .unwrap();
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(num_shapes),
            num_shapes,
            |b, _num_shapes| {
                b.iter(|| {
                    let mut response = vec![];
                    canvas
                        .query_shapes_on_layer(black_box(layer_id), &mut response, Duration::ZERO)
                        .unwrap();
                    black_box(response);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark querying shapes on a layer with a large database (10 layers, 1000 shapes each, for 10k shapes total)
fn bench_query_shapes_large_database(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_shapes_on_layer_large_db");

    // Create a canvas with 10 layers and 10,000 shapes per layer
    let num_layers          = 10;
    let shapes_per_layer    = 1000;

    let mut canvas = create_populated_canvas(num_layers, shapes_per_layer);

    // Get the layer IDs
    let mut layers = vec![];
    canvas.query_document_outline(&mut layers).unwrap();

    let layer_ids: Vec<CanvasLayerId> = layers
        .iter()
        .filter_map(|response| {
            if let VectorResponse::Layer(id, _) = response {
                Some(*id)
            } else {
                None
            }
        })
        .collect();

    group.throughput(Throughput::Elements(shapes_per_layer as u64));

    // Benchmark querying the first layer
    let first_layer = layer_ids[0];
    group.bench_function("first_layer", |b| {
        b.iter(|| {
            let mut response = vec![];
            canvas
                .query_shapes_on_layer(black_box(first_layer), &mut response, Duration::ZERO)
                .unwrap();
            black_box(response);
        });
    });

    // Benchmark querying the middle layer
    let middle_layer = layer_ids[num_layers / 2];
    group.bench_function("middle_layer", |b| {
        b.iter(|| {
            let mut response = vec![];
            canvas
                .query_shapes_on_layer(black_box(middle_layer), &mut response, Duration::ZERO)
                .unwrap();
            black_box(response);
        });
    });

    // Benchmark querying the last layer
    let last_layer = layer_ids[num_layers - 1];
    group.bench_function("last_layer", |b| {
        b.iter(|| {
            let mut response = vec![];
            canvas
                .query_shapes_on_layer(black_box(last_layer), &mut response, Duration::ZERO)
                .unwrap();
            black_box(response);
        });
    });

    group.finish();
}

/// Benchmark to show performance per shape as database size increases
fn bench_performance_per_shape(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_per_shape");

    // For each database size, measure the time per shape
    for total_shapes in [1000, 5000, 10000, 25000].iter() {
        let shapes_per_layer    = total_shapes / 10;
        let mut canvas          = create_populated_canvas(10, shapes_per_layer);

        // Get the first layer
        let mut layers = vec![];
        canvas.query_document_outline(&mut layers).unwrap();

        let layer_id = layers
            .iter()
            .find_map(|response| {
                if let VectorResponse::Layer(id, _) = response {
                    Some(*id)
                } else {
                    None
                }
            })
            .unwrap();

        group.throughput(Throughput::Elements(shapes_per_layer as u64));

        group.bench_with_input(
            BenchmarkId::new("total_shapes", total_shapes),
            total_shapes,
            |b, _total_shapes| {
                b.iter(|| {
                    let mut response = vec![];
                    canvas
                        .query_shapes_on_layer(black_box(layer_id), &mut response, Duration::ZERO)
                        .unwrap();
                    black_box(response);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark adding shapes to a layer as it grows
/// Measures the time to double the number of shapes: 10->20, 20->40, ..., 10000->20000
fn bench_add_shapes_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_shapes_scaling");

    // Test doubling from various starting sizes
    let sizes = [10, 100, 1000, 10000];

    for &starting_size in &sizes {
        let shapes_to_add = starting_size; // We're doubling, so add the same amount
        group.throughput(Throughput::Elements(shapes_to_add as u64));

        group.bench_with_input(
            BenchmarkId::new("from_size", starting_size),
            &starting_size,
            |b, &start_size| {
                b.iter_batched(
                    || {
                        // Setup: Create a canvas with a layer containing start_size shapes
                        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
                        let layer_id    = CanvasLayerId::new();
                        canvas.add_layer(layer_id, None).unwrap();

                        // Add the initial shapes
                        for i in 0..start_size {
                            let shape_id = CanvasShapeId::new();
                            let x        = (i % 100) as f32 * 15.0;
                            let y        = (i / 100) as f32 * 15.0;
                            let shape = create_test_rectangle(x, y);

                            canvas
                                .add_shape(shape_id, test_shape_type(), shape)
                                .unwrap();
                            canvas
                                .set_shape_parent(shape_id, CanvasShapeParent::Layer(layer_id, Duration::from_nanos(0)))
                                .unwrap();
                            canvas
                                .set_shape_properties(shape_id, create_test_properties(i))
                                .unwrap();
                        }

                        (canvas, layer_id, start_size)
                    },
                    |(mut canvas, layer_id, start_size)| {
                        // Benchmark: Add another start_size shapes (doubling the total)
                        for i in start_size..(start_size * 2) {
                            let shape_id = CanvasShapeId::new();
                            let x = (i % 100) as f32 * 15.0;
                            let y = (i / 100) as f32 * 15.0;
                            let shape = create_test_rectangle(x, y);

                            canvas
                                .add_shape(black_box(shape_id), test_shape_type(), shape)
                                .unwrap();
                            canvas
                                .set_shape_parent(
                                    black_box(shape_id),
                                    CanvasShapeParent::Layer(layer_id, Duration::from_nanos(0)),
                                )
                                .unwrap();
                            canvas
                                .set_shape_properties(black_box(shape_id), create_test_properties(i))
                                .unwrap();
                        }
                        black_box(canvas);
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }

    group.finish();
}

/// Benchmark adding individual shapes at different layer sizes
/// Shows the per-shape cost as the layer grows
fn bench_add_single_shape_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_single_shape_scaling");

    // Test adding a single shape to layers of varying sizes
    let layer_sizes = [10, 50, 100, 500, 1000, 2500, 5000, 10000];

    for &layer_size in &layer_sizes {
        group.throughput(Throughput::Elements(1));

        group.bench_with_input(
            BenchmarkId::new("layer_size", layer_size),
            &layer_size,
            |b, &size| {
                b.iter_batched(
                    || {
                        // Setup: Create a canvas with a layer containing size shapes
                        let mut canvas  = SqliteCanvas::new_in_memory().unwrap();
                        let layer_id    = CanvasLayerId::new();
                        canvas.add_layer(layer_id, None).unwrap();

                        // Add the initial shapes
                        for i in 0..size {
                            let shape_id = CanvasShapeId::new();
                            let x        = (i % 100) as f32 * 15.0;
                            let y        = (i / 100) as f32 * 15.0;
                            let shape = create_test_rectangle(x, y);

                            canvas
                                .add_shape(shape_id, test_shape_type(), shape)
                                .unwrap();
                            canvas
                                .set_shape_parent(shape_id, CanvasShapeParent::Layer(layer_id, Duration::from_nanos(0)))
                                .unwrap();
                            canvas
                                .set_shape_properties(shape_id, create_test_properties(i))
                                .unwrap();
                        }

                        (canvas, layer_id, size)
                    },
                    |(mut canvas, layer_id, size)| {
                        // Benchmark: Add a single shape to the layer
                        let shape_id = CanvasShapeId::new();
                        let x = (size % 100) as f32 * 15.0;
                        let y = (size / 100) as f32 * 15.0;
                        let shape = create_test_rectangle(x, y);

                        canvas
                            .add_shape(black_box(shape_id), test_shape_type(), shape)
                            .unwrap();
                        canvas
                            .set_shape_parent(
                                black_box(shape_id),
                                CanvasShapeParent::Layer(layer_id, Duration::from_nanos(0)),
                            )
                            .unwrap();
                        canvas
                            .set_shape_properties(black_box(shape_id), create_test_properties(size))
                            .unwrap();
                        black_box(canvas);
                    },
                    criterion::BatchSize::LargeInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_query_shapes_scaling,
    bench_query_shapes_large_database,
    bench_performance_per_shape,
    bench_add_shapes_scaling,
    bench_add_single_shape_scaling
);
criterion_main!(benches);
