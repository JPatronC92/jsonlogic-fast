use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jsonlogic_fast::{evaluate, evaluate_batch_numeric, evaluate_numeric};

fn bench_single_numeric_evaluation(c: &mut Criterion) {
    let rule = r#"{"*":[{"var":"amount"},0.029]}"#;
    let context = r#"{"amount":1500.0,"method":"CREDIT_CARD"}"#;

    c.bench_function("single_numeric_evaluation", |b| {
        b.iter(|| {
            let result = evaluate_numeric(black_box(rule), black_box(context)).unwrap();
            black_box(result);
        })
    });
}

fn bench_single_generic_evaluation(c: &mut Criterion) {
    let rule = r#"{"if":[{"==":[{"var":"country"},"MX"]},"domestic","intl"]}"#;
    let context = r#"{"country":"MX"}"#;

    c.bench_function("single_generic_evaluation", |b| {
        b.iter(|| {
            let result = evaluate(black_box(rule), black_box(context)).unwrap();
            black_box(result);
        })
    });
}

fn bench_batch_numeric_10k(c: &mut Criterion) {
    let rule = r#"{"if":[{">":[{"var":"amount"},1000]},{"*":[{"var":"amount"},0.025]},{"*":[{"var":"amount"},0.035]}]}"#;
    let contexts: Vec<String> = (0..10_000)
        .map(|i| format!(r#"{{"amount": {}}}"#, (i as f64) * 1.5))
        .collect();

    c.bench_function("batch_numeric_10k", |b| {
        b.iter(|| {
            let results = evaluate_batch_numeric(black_box(rule), black_box(&contexts)).unwrap();
            black_box(results);
        })
    });
}

criterion_group!(
    benches,
    bench_single_numeric_evaluation,
    bench_single_generic_evaluation,
    bench_batch_numeric_10k
);
criterion_main!(benches);
