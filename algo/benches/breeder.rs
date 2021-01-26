use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;

fn breeder_benchmark(c: &mut Criterion) {
  c.bench_function("breeder 100000 generations", |b| b.iter(|| {
    let src = fs::read_to_string("tests/fixtures/Breeder.lif").unwrap();
    let mut uni = algo::rle::read(src).unwrap();

    uni.simulate(black_box(100000));
  }));
}

criterion_group!(benches, breeder_benchmark);
criterion_main!(benches);