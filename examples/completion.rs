//! Example on implementing a completer.

use cmdtree::completion::*;
use cmdtree::{Builder, BuilderChain};

fn main() {
    let cmder = Builder::default_config("cmdtree-example")
        .begin_class("class1", "") // a class
        .begin_class("inner-class1", "") // can nest a class
        .add_action("name", "print class name", |_, _| ())
        .end_class()
        .end_class()
        .begin_class("print", "")
        .add_action("echo", "", |_, _| ())
        .add_action("countdown", "", |_, _| ())
        .into_commander()
        .unwrap();

    cmder.run_with_completion(|c| TreeCompleter {
        items: create_tree_completion_items(c),
    });
}

struct TreeCompleter {
    items: Vec<String>,
}

impl<T: Terminal> Completer<T> for TreeCompleter {
    fn complete(
        &self,
        _word: &str,
        prompter: &Prompter<T>,
        _start: usize,
        _end: usize,
    ) -> Option<Vec<Completion>> {
        Some(
            tree_completions(prompter.buffer(), self.items.iter())
                .map(|x| Completion::simple(x.to_string()))
                .collect(),
        )
    }
}
