//! Completion of tree paths and action arguments.
//!
//! Completion is done functionally, see examples on github for how to implement.

use super::*;
#[cfg(feature = "runnable")]
use colored::*;
#[cfg(feature = "runnable")]
pub use linefeed::{Completer, Completion, Interface, Prompter, ReadResult, Terminal};

impl<'r, R> Commander<'r, R> {
    /// Run the `Commander` interactively, with a completer constructed on every loop.
    /// Consumes the instance, and blocks the thread until the loop is exited.
    ///
    /// See examples for how to construct a completer.
    #[cfg(feature = "runnable")]
    pub fn run_with_completion<
        C: 'static + Completer<linefeed::DefaultTerminal>,
        F: Fn(&Self) -> C,
    >(
        mut self,
        completer_fn: F,
    ) {
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

/// Match string and qualified name of action.
#[derive(Debug, PartialEq)]
pub struct ActionMatch<'a> {
    /// The match str, space delimited from current path.
    ///
    /// > Notice the extra space at the end. This is intentional.
    ///
    /// eg `"a nested action "`.
    pub info: CompletionInfo<'a>,
    /// Qualified action name from root, as produced from [`structure`].
    /// eg `a.nested..action`
    ///
    /// [`structure`]: Commander::structure
    pub qualified_path: String,
}

/// Completion item.
#[derive(Debug, PartialEq)]
pub struct CompletionInfo<'a> {
    /// The string to match. Similar to path but space delimited.
    pub completestr: String,
    /// Class or action.
    pub itemtype: ItemType,
    /// The help message.
    pub help_msg: &'a str,
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
/// let v: Vec<_> = create_tree_completion_items(&cmder).into_iter().map(|x| x.completestr).collect();
/// assert_eq!(v, vec!["hello", "one", "one two", "one two three"]
/// 	.into_iter().map(|x| x.to_string()).collect::<Vec<_>>());
///
/// cmder.parse_line("one", true, &mut std::io::sink());
///
/// let v: Vec<_> = create_tree_completion_items(&cmder).into_iter().map(|x| x.completestr).collect();
/// assert_eq!(v, vec!["two", "two three"]
/// 	.into_iter().map(|x| x.to_string()).collect::<Vec<_>>());
/// ```
pub fn create_tree_completion_items<'a, R>(cmdr: &Commander<'a, R>) -> Vec<CompletionInfo<'a>> {
    cmdr.structure(false)
        .into_iter()
        .filter_map(|info| {
            let StructureInfo {
                path,
                itemtype,
                help_msg,
            } = info;

            dbg!(&path);

            let completestr =
                path.split('.')
                    .filter(|x| !x.is_empty())
                    .fold(String::new(), |mut s, x| {
                        if s.len() != 0 {
                            s.push(' ');
                        }
                        s.push_str(x);
                        s
                    });

            if completestr.is_empty() {
                None
            } else {
                Some(CompletionInfo {
                    completestr,
                    itemtype,
                    help_msg,
                })
            }
        })
        .collect()
}

/// Constructs a set of space delimited actions that could be completed at the
/// current path.
pub fn create_action_completion_items<'a, R>(cmdr: &Commander<'a, R>) -> Vec<ActionMatch<'a>> {
    let cpath = cmdr.path();
    let rname = cmdr.root_name();

    let starter = if cpath == rname {
        "" // no starting prefix
    } else {
        &cpath[rname.len() + 1..] // remove the 'root_name.' portion
    };

    cmdr.structure(true)
        .into_iter()
        .filter(|x| x.path.contains("..") && x.path.starts_with(starter))
        .filter_map(|x| {
            let StructureInfo {
                path,
                itemtype,
                help_msg,
            } = x;

            let qualified_path = path.clone();

            let completestr = path[starter.len()..]
                .split('.')
                .filter(|x| !x.is_empty())
                .fold(String::new(), |mut s, x| {
                    s.push_str(x);
                    s.push(' ');
                    s
                });

            if completestr.is_empty() {
                None
            } else {
                let info = CompletionInfo {
                    completestr,
                    itemtype,
                    help_msg,
                };

                Some(ActionMatch {
                    info,
                    qualified_path,
                })
            }
        })
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
/// [`create_tree_completion_items`]: completion::create_tree_completion_items
pub fn tree_completions<'l: 'i, 'i, 'a: 'i, I>(
    line: &'l str,
    items: I,
) -> impl Iterator<Item = (&'i str, &'i CompletionInfo<'a>)>
where
    I: Iterator<Item = &'i CompletionInfo<'a>>,
{
    items
        .filter(move |x| x.completestr.starts_with(line))
        .map(move |x| {
            // src code makes word_idx = line.len(), then counts backwards.
            // will not panic on out of bounds.
            let word_idx = word_break_start(line, &[' ']);
            let word = &x.completestr[word_idx..];
            (word, x)
        })
}

/// Returns the start position of the _last_ word, delimited by any character.
pub fn word_break_start(s: &str, word_break_ch: &[char]) -> usize {
    let mut start = s.len();

    for (idx, ch) in s.char_indices().rev() {
        if word_break_ch.contains(&ch) {
            break;
        }
        start = idx;
    }

    start
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

        let v: Vec<_> = {
            let v: Vec<_> = create_tree_completion_items(&cmder)
                .into_iter()
                .map(|x| x.completestr)
                .collect();
            v
        };
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

        let v: Vec<_> = create_tree_completion_items(&cmder)
            .into_iter()
            .map(|x| x.completestr)
            .collect();
        assert_eq!(v, vec_str(vec!["inner-class1", "inner-class1 name",]));
    }

    #[test]
    fn create_action_completion_items_test() {
        let mut cmder = Builder::default_config("eg")
            .begin_class("one", "") // a class
            .begin_class("two", "")
            .add_action("three", "", |_, _| ())
            .end_class()
            .end_class()
            .begin_class("hello", "")
            .end_class()
            .into_commander()
            .unwrap();

        let v: Vec<_> = create_action_completion_items(&cmder)
            .into_iter()
            .map(|x| (x.info.completestr, x.qualified_path))
            .collect();
        assert_eq!(
            v,
            vec![("one two three ".to_string(), "one.two..three".to_string(),)]
        );

        cmder.parse_line("one", true, &mut std::io::sink());

        let v: Vec<_> = create_action_completion_items(&cmder)
            .into_iter()
            .map(|x| (x.info.completestr, x.qualified_path))
            .collect();
        assert_eq!(
            v,
            vec![("two three ".to_string(), "one.two..three".to_string(),)]
        );
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
        let completions = tree_completions("", v.iter())
            .map(|x| x.0)
            .collect::<Vec<_>>();
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

        let completions = tree_completions("cl", v.iter())
            .map(|x| x.0)
            .collect::<Vec<_>>();
        assert_eq!(
            completions,
            vec![
                "clone",
                "class1",
                "class1 inner-class1",
                "class1 inner-class1 name",
            ]
        );

        let completions = tree_completions("class1 ", v.iter())
            .map(|x| x.0)
            .collect::<Vec<_>>();
        assert_eq!(completions, vec!["inner-class1", "inner-class1 name",]);

        cmder.parse_line("class1", true, &mut std::io::sink());

        let v = create_tree_completion_items(&cmder);
        let completions = tree_completions("inn", v.iter())
            .map(|x| x.0)
            .collect::<Vec<_>>();
        assert_eq!(completions, vec!["inner-class1", "inner-class1 name",]);
    }

    fn vec_str(v: Vec<&str>) -> Vec<String> {
        v.into_iter().map(|x| x.to_string()).collect()
    }
}
