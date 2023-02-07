use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fnv::FnvHashSet;

use std::{collections::{BTreeSet, HashSet}, vec};

use rand::{thread_rng, Rng};
use rdxsort::RdxSort;

fn radix(mut A: Vec<u32>, mut B: Vec<u32>) -> Vec<u32> {

    A.rdxsort();
    B.rdxsort();

    let mut A = A.into_iter().peekable();
    let mut B = B.into_iter().peekable();

    let mut C = vec![];

    while let (Some(a), Some(b)) = (A.peek(), B.peek()) {
        if a > b {
            B.next();
        } else if b > a {
            A.next();
        } else {
            C.push(A.next().unwrap());
            B.next();
        }
    }

    C
}


fn btree(A: Vec<u32>, B: Vec<u32>) -> Vec<u32> {
    let A: BTreeSet<u32> = A.into_iter().collect();
    let B: BTreeSet<u32> = B.into_iter().collect();
    A.intersection(&B).copied().collect()
}

fn fnv_hash_A(A: Vec<u32>, B: Vec<u32>) -> Vec<u32> {
    let B: FnvHashSet<u32> = B.into_iter().collect();

    let mut C = vec![];

    for a in A {
        if B.contains(&a) {
            C.push(a);
        }
    }

    C
}

fn fnv_hash_B(A: Vec<u32>, B: Vec<u32>) -> Vec<u32> {
    let A: FnvHashSet<u32> = A.into_iter().collect();

    let mut C = vec![];

    for b in B {
        if A.contains(&b) {
            C.push(b);
        }
    }

    C
}

fn hash_A(A: Vec<u32>, B: Vec<u32>) -> Vec<u32> {
    let B: HashSet<u32> = B.into_iter().collect();

    let mut C = vec![];

    for a in A {
        if B.contains(&a) {
            C.push(a);
        }
    }

    C
}

fn hash_B(A: Vec<u32>, B: Vec<u32>) -> Vec<u32> {
    let A: HashSet<u32> = A.into_iter().collect();

    let mut C = vec![];

    for b in B {
        if A.contains(&b) {
            C.push(b);
        }
    }

    C
}

fn criterion_benchmark(c: &mut Criterion) {
    let A: Vec<u32> = (0..10000000).map(|_| thread_rng().gen()).collect();
    let B: Vec<u32> = (0..500000).map(|_| thread_rng().gen()).collect();

    c.bench_function("radix", |b| b.iter(|| radix(black_box(A.clone()), black_box(B.clone()))));

    // Awful slow
    // c.bench_function("btree", |b| b.iter(|| btree(black_box(A.clone()), black_box(B.clone()))));

    c.bench_function("hash_A", |b| b.iter(|| hash_A(black_box(A.clone()), black_box(B.clone()))));
    c.bench_function("hash_B", |b| b.iter(|| hash_B(black_box(A.clone()), black_box(B.clone()))));
    c.bench_function("fnv_hash_A", |b| b.iter(|| fnv_hash_A(black_box(A.clone()), black_box(B.clone()))));
    c.bench_function("fnv_hash_B", |b| b.iter(|| fnv_hash_B(black_box(A.clone()), black_box(B.clone()))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);