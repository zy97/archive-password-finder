use criterion::{black_box, criterion_group, criterion_main, Criterion};
use password_crack::{charset_lowercase_letters, PasswordGenerator};

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("generate_password", |b| {
        let charset = charset_lowercase_letters();
        let min_password_len = 3;
        let max_password_len = 5;
        b.iter(|| {
            let iterator =
                PasswordGenerator::new(charset.clone(), min_password_len, max_password_len);
            let _last = black_box(iterator.last());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
