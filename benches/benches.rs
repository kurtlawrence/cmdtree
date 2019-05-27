#[macro_use]
extern crate criterion;

use criterion::Criterion;

use cmdtree::*;
use completion::*;

fn parse_line(c: &mut Criterion) {
    let mut cmdr = build_cmdr();

    c.bench_function("parse_line_root", move |b| {
        b.iter(|| {
            assert_ne!(
                cmdr.parse_line("one-class nested some-action", false, &mut std::io::sink()),
                LineResult::Unrecognized
            )
        })
    });

    let mut cmdr = build_cmdr();

    c.bench_function("parse_line_lrg", move |b| {
        b.iter(|| {
            assert_eq!(
                cmdr.parse_line("c", false, &mut std::io::sink()),
                LineResult::Cancel
            );
            assert_ne!(
                cmdr.parse_line(
                    "more_stuff insider one two three four five six seven eight nine ten",
                    false,
                    &mut std::io::sink()
                ),
                LineResult::Unrecognized
            )
        })
    });
}

fn build_completion(c: &mut Criterion) {
    let cmdr = build_cmdr();

    c.bench_function("build_tree_completion_items", move |b| {
        b.iter(|| create_tree_completion_items(&cmdr))
    });

    let cmdr = build_cmdr();

    c.bench_function("build_action_completion_items", move |b| {
        b.iter(|| create_action_completion_items(&cmdr))
    });

    let cmdr = build_cmdr();
    let items = create_tree_completion_items(&cmdr);

    c.bench_function("tree_completions", move |b| {
        b.iter(|| tree_completions("", items.iter()))
    });
}

fn build_cmdr<'a>() -> Commander<'a, ()> {
    let cmdr = Builder::default_config("root")
        .begin_class("one-class", "")
        .begin_class("nested", "")
        .add_action("some-action", "", |_, _| ())
        .end_class()
        .begin_class("another", "")
        .end_class()
        .end_class()
        .begin_class("more_stuff", "")
        .begin_class("insider", "")
        .begin_class("one", "")
        .begin_class("two", "")
        .begin_class("three", "")
        .begin_class("four", "")
        .begin_class("five", "")
        .begin_class("six", "")
        .begin_class("seven", "")
        .begin_class("eight", "")
        .begin_class("nine", "")
        .begin_class("ten", "")
        .into_commander()
        .unwrap();

    cmdr
}

criterion_group!(benches, parse_line, build_completion);
criterion_main!(benches);
