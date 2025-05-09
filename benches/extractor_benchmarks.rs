// benches/extractor_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rstparser::extractor::RstExtractor;

// Sample C++ content for benchmarking
const CPP_CONTENT_SMALL: &str = r#"
/// @rst
/// Small C++ RST block.
/// @endrst
"#;

const CPP_CONTENT_MEDIUM: &str = r#"
/// Some C++ code
/// More C++ code
/// @rst
/// This is a medium-sized RST content block.
/// It has several lines.
/// - Point one
/// - Point two
/// @endrst
/// Even more C++ code
/// And some more.
/// @rst
/// Another block.
/// @endrst
"#;

// Sample Python content for benchmarking
const PY_CONTENT_SMALL: &str = r#"
def func_small():
    """
    @rst
    Small Python RST.
    @endrst
    """
    pass
"#;

const PY_CONTENT_MEDIUM: &str = r#"
def func_medium():
    """
    Some Python docstring.
    More lines here.
    @rst
    This is a medium Python RST block.
    It spans multiple lines.
    1. Item A
    2. Item B
    @endrst
    Trailing docstring.
    @rst
    Another Python block.
    @endrst
    """
    pass
"#;

fn benchmark_extract_from_cpp(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract_from_cpp_regex");

    group.bench_function("small_cpp_regex", |b| {
        b.iter(|| RstExtractor::extract_from_cpp(black_box(CPP_CONTENT_SMALL)))
    });

    group.bench_function("medium_cpp_regex", |b| {
        b.iter(|| RstExtractor::extract_from_cpp(black_box(CPP_CONTENT_MEDIUM)))
    });
    group.finish();
}

fn benchmark_extract_from_python(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract_from_python_regex");

    group.bench_function("small_py_regex", |b| {
        b.iter(|| RstExtractor::extract_from_python(black_box(PY_CONTENT_SMALL)))
    });

    group.bench_function("medium_py_regex", |b| {
        b.iter(|| RstExtractor::extract_from_python(black_box(PY_CONTENT_MEDIUM)))
    });
    group.finish();
}



criterion_group!(
    benches,
    benchmark_extract_from_cpp,
    benchmark_extract_from_python,

);
criterion_main!(benches);
