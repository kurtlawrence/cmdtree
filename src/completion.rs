pub use super::*;
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

    pub fn create_tree_completion_items(&self) -> Vec<String> {
        let cpath = self.path();

        self.structure()
            .into_iter()
            .map(|x| {
                x[cpath.len()..].split('.').filter(|x| !x.is_empty()).fold(
                    String::new(),
                    |mut s, x| {
                        if s.len() != 0 {
                            s.push(' ');
                        }
                        s.push_str(x);
                        s
                    },
                )
            })
            .collect()
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
