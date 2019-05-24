pub use super::*;
use colored::*;
pub use linefeed::{Completer, Completion, Prompter, Terminal};

impl<'r, R> Commander<'r, R> {
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

pub fn tree_completions<'a, I>(line: &str, items: I) -> Option<Vec<Completion>>
where
    I: Iterator<Item = &'a String>,
{
    let v: Vec<_> = items
        .filter(|x| x.starts_with(line))
        .map(|x| {
            // src code makes word_idx = x.len(), then counts backwards.
            // will not panic on out of bounds.
            let word_idx = linefeed::complete::word_break_start(x, " ");
            Completion::simple(x[word_idx..].to_string())
        })
        .collect();

    if v.len() > 0 {
        Some(v)
    } else {
        None
    }
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

    fn vec_str(v: Vec<&str>) -> Vec<String> {
        v.into_iter().map(|x| x.to_string()).collect()
    }
}

// pub struct TreeCompleter {
//     space_separated_elements: Vec<String>,
// }

// impl<'r, R> Commander<'r, R> {
// 	pub fn build_tree_completer(&self) -> TreeCompleter {
//         let cpath = self.path();

//         let prefix = if self.at_root() { "." } else { "" };

//         let space_separated_elements = self
//             .structure()
//             .into_iter()
//             .map(|x| {
//                 x[cpath.len()..].split('.').filter(|x| !x.is_empty()).fold(
//                     String::from(prefix),
//                     |mut s, x| {
//                         if s.len() != prefix.len() {
//                             s.push(' ');
//                         }
//                         s.push_str(x);
//                         s
//                     },
//                 )
//             })
//             .collect();

//         TreeCompleter {
//             space_separated_elements,
//         }
//     }
// }

// impl<T: Terminal> Completer<T> for TreeCompleter {
//     fn complete(
//         &self,
//         word: &str,
//         prompter: &Prompter<T>,
//         start: usize,
//         end: usize,
//     ) -> Option<Vec<Completion>> {
//         let line = &prompter.buffer();

//         // start is the index in the line
//         // need to return just the _word_ portion
//         Some(
//             self.space_separated_elements
//                 .iter()
//                 .filter(|x| x.starts_with(line))
//                 .map(|x| Completion::simple(x[start..].to_string()))
//                 .collect(),
//         )
//     }
// }
