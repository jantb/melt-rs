use criterion::{Criterion, criterion_group, criterion_main};
use melt_rs::index::SearchIndex;

fn criterion_benchmark(c: &mut Criterion) {
    let mut index = SearchIndex::default();
    for i in 1..1_000_000 {
        let _ = index.add(format!("Long and winding text where ever it leads its my fault oh no{}", i).as_str());
    }
    for i in 1_000_001..2_000_001 {
        let _ = index.add(format!("Wrong wrong is the key{}", i).as_str());
    }
    for i in 3_000_001..4_000_001 {
        let _ = index.add(format!("There has been some improper way to do this{}", i).as_str());
    }
    for i in 2_000_001..3_000_001 {
        let _ = index.add(format!("World is ending and I dont like it at all, there would be sa time for this and that and where does it evolve around the sun so bright and big{}", i).as_str());
    }
    c.bench_function("search for hello", |b| b.iter(|| index.search("hello")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);