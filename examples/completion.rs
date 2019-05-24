//! Example on implementing a completer.

use cmdtree::completion::*;
use cmdtree::*;

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

    cmder.run(); // run interactively
}

struct TreeCompleter {
    items: Vec<String>,
}

impl<T: Terminal> Completer<T> for TreeCompleter {
    fn complete(
        &self,
        word: &str,
        prompter: Prompter<T>,
        start: usize,
        end: usize,
    ) -> Option<Vec<Completion>> {
        None
    }
}
