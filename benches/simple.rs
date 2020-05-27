use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lieutenant::{any, literal, param, Input, Parser, ParserBase};

pub fn input_head(c: &mut Criterion) {
    let input: Input = "hello world".into();
    c.bench_function("input head", |b| {
        let mut input = input.clone();
        b.iter(|| {
            let _ = input.advance_until(" ");
        })
    });
}

pub fn single_literal(c: &mut Criterion) {
    let root = literal("hello");
    let input: Input = "hello".into();
    c.bench_function("literal", |b| {
        let input = input.clone();
        b.iter(|| {
            let _ = root.parse(&mut black_box(input));
        })
    });
}

pub fn zero_cost(c: &mut Criterion) {
    let root = any();
    let input: Input = "".into();
    c.bench_function("single any", |b| {
        let input = input.clone();
        b.iter(|| {
            let _ = root.parse(&mut black_box(input));
        })
    });

    let root = any()
        .then(any());

    c.bench_function("two anys", |b| {
        let input = input.clone();
        b.iter(|| {
            let _ = root.parse(&mut black_box(input));
        })
    });

    let root = any()
        .then(any())
        .then(any())
        .then(any())
        .then(any())
        .then(any())
        .then(any())
        .then(any());

    c.bench_function("multiple anys", |b| {
        let input = input.clone();
        b.iter(|| {
            let _ = root.parse(&mut black_box(input));
        })
    });
}

pub fn with_context(c: &mut Criterion) {
    let root = literal("hello")
        .then(literal("world"))
        .then(param())
        .map(|a: i32| move |n: &mut i32| *n += a);

    let mut n = 45;

    let input: Input = "hello world -3".into();

    c.bench_function("parse", |b| {
        let input = input.clone();
        b.iter(|| {
            let _ = root.parse(&mut black_box(input));
        })
    });

    c.bench_function("parse + call", |b| {
        b.iter(|| {
            if let Some((command,)) = root.parse(&mut black_box(input.clone())) {
                command(&mut n)
            }
        })
    });
}

criterion_group!(
    benches,
    input_head,
    zero_cost,
    single_literal,
    with_context
);
criterion_main!(benches);
