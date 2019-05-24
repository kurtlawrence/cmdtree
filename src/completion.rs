//! Completion of tree paths and action arguments.
//!
//! Completion is done functionally, see examples on github for how to implement.

pub use super::*;
use colored::*;
pub use linefeed::{Completer, Completion, Prompter, Terminal};

impl<'r, R> Commander<'r, R> {
    /// Run the `Commander` interactively, with a completer constructed on every loop.
    /// Consumes the instance, and blocks the thread until the loop is exited.
    ///
    /// See examples for how to construct a completer.
    pub fn run_with_completion<
        C: 'static + Completer<linefeed::DefaultTerminal>,
        F: Fn(&Self) -> C,
    >(
        mut self,
        completer_fn: F,
    ) {
        use std::sync::Arc;

        let interface = Interface::new("commander").expect("failed to start interface");
        let mut exit = false;

        while !exit {
            interface
                .set_prompt(&format!("{}=> ", self.path().bright_cyan()))
                .expect("failed to set prompt");

            let completer = completer_fn(&self);
            interface.set_completer(Arc::new(completer));

            match interface.read_line() {
                Ok(ReadResult::Input(s)) => {
                    match self.parse_line(&s, true, &mut std::io::stdout()) {
                        LineResult::Exit => exit = true,
                        _ => (),
                    }
                    interface.add_history_unique(s);
                }
                _ => (),
            }
        }
    }
}

/// Constructs a set of space delimited items that could be completed at the
/// current path.
///
/// # Examples
/// ```rust
/// # use cmdtree::*;
/// # use cmdtree::completion::create_tree_completion_items;
///
/// let mut cmder = Builder::default_config("eg")
/// 	.begin_class("one", "") // a class
/// 		.begin_class("two", "")
/// 		.add_action("three", "", |_, _| ())
/// 		.end_class()
/// 	.end_class()
/// 	.begin_class("hello", "").end_class()
/// 	.into_commander().unwrap();
///
/// let v = create_tree_completion_items(&cmder);
/// assert_eq!(v, vec!["hello", "one", "one two", "one two three"]
/// 	.into_iter().map(|x| x.to_string()).collect::<Vec<_>>());
///
/// cmder.parse_line("one", true, &mut std::io::sink());
///
/// let v = create_tree_completion_items(&cmder);
/// assert_eq!(v, vec!["two", "two three"]
/// 	.into_iter().map(|x| x.to_string()).collect::<Vec<_>>());
/// ```
pub fn create_tree_completion_items<R>(cmdr: &Commander<R>) -> Vec<String> {
    let cpath = cmdr.path();

    cmdr.structure()
        .into_iter()
        .filter(|x| x.starts_with(cpath))
        .map(|x| {
            x[cpath.len()..]
                .split('.')
                .filter(|x| !x.is_empty())
                .fold(String::new(), |mut s, x| {
                    if s.len() != 0 {
                        s.push(' ');
                    }
                    s.push_str(x);
                    s
                })
        })
        .filter(|x| !x.is_empty())
        .collect()
}

/// Determines from a set of items the ones that could be
/// completed from the given line.
///
/// Effectively loops through each item and checks if it
/// starts with `line`.
/// `items` should be constructed by [`create_tree_completion_items`].
///
/// The returned items are only the slice from the final _word_ in `line`,
/// such that `hello wo` would return `world`, and `he` would return `hello world`.
///
/// # Examples
/// ```rust
/// # use cmdtree::completion::tree_completions;
///
/// let v = vec!["one", "one two", "only"];
///
/// let completions = tree_completions("o", v.iter());
/// assert_eq!(completions.collect::<Vec<_>>(), vec!["one", "one two", "only"]);
///
/// let completions = tree_completions("one", v.iter());
/// assert_eq!(completions.collect::<Vec<_>>(), vec!["one", "one two"]);
/// ```
///
/// [`create_tree_completion_items`]: completion::create_tree_completion_items
pub fn tree_completions<'a, I, T>(line: &'a str, items: I) -> impl Iterator<Item = &'a str>
where
    I: Iterator<Item = &'a T>,
    T: 'a + AsRef<str>,
{
    items
        .map(|x| x.as_ref())
        .filter(move |x| x.starts_with(line))
        .map(move |x| {
            // src code makes word_idx = line.len(), then counts backwards.
            // will not panic on out of bounds.
            let word_idx = linefeed::complete::word_break_start(line, " ");
            &x[word_idx..]
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_tree_completion_items_test() {
        let mut cmder = Builder::default_config("cmdtree-example")
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

        let v = create_tree_completion_items(&cmder);
        assert_eq!(
            v,
            vec_str(vec![
                "class1",
                "class1 inner-class1",
                "class1 inner-class1 name",
                "print",
                "print countdown",
                "print echo"
            ])
        );

        cmder.parse_line("class1", true, &mut std::io::sink());

        let v = create_tree_completion_items(&cmder);
        assert_eq!(v, vec_str(vec!["inner-class1", "inner-class1 name",]));
    }

    #[test]
    fn tree_completions_test() {
        let mut cmder = Builder::default_config("cmdtree-example")
            .begin_class("class1", "")
            .begin_class("inner-class1", "")
            .add_action("name", "", |_, _| ())
            .end_class()
            .end_class()
            .begin_class("print", "")
            .add_action("echo", "", |_, _| ())
            .add_action("countdown", "", |_, _| ())
            .end_class()
            .add_action("clone", "", |_, _| ())
            .into_commander()
            .unwrap();

        let v = create_tree_completion_items(&cmder);
        let completions = tree_completions("", v.iter()).collect::<Vec<_>>();
        assert_eq!(
            completions,
            vec![
                "clone",
                "class1",
                "class1 inner-class1",
                "class1 inner-class1 name",
                "print",
                "print countdown",
                "print echo",
            ]
        );

        let completions = tree_completions("cl", v.iter()).collect::<Vec<_>>();
        assert_eq!(
            completions,
            vec![
                "clone",
                "class1",
                "class1 inner-class1",
                "class1 inner-class1 name",
            ]
        );

        let completions = tree_completions("class1 ", v.iter()).collect::<Vec<_>>();
        assert_eq!(completions, vec!["inner-class1", "inner-class1 name",]);

        cmder.parse_line("class1", true, &mut std::io::sink());

        let v = create_tree_completion_items(&cmder);
        let completions = tree_completions("inn", v.iter()).collect::<Vec<_>>();
        assert_eq!(completions, vec!["inner-class1", "inner-class1 name",]);
    }

    fn vec_str(v: Vec<&str>) -> Vec<String> {
        v.into_iter().map(|x| x.to_string()).collect()
    }
}
