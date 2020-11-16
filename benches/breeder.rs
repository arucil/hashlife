use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hashlife::universe::*;
use hashlife::rle;
use std::fs;

fn breeder_benchmark(c: &mut Criterion) {
  c.bench_function("breeder 100000 generations", |b| b.iter(|| {
    let mut uni = Universe::new();
    let src = fs::read_to_string("tests/fixtures/Breeder.lif").unwrap();
    let node = rle::read(src, &mut uni);

    let _ = uni.simulate(node, black_box(100000));
  }));
}

criterion_group!(benches, breeder_benchmark);
criterion_main!(benches);