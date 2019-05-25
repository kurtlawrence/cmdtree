//! Example on implementing an argument completer.
//! 
//! try running and hitting TAB with the following inputs to see completion!
//! ```
//! arg-completion=> path  <-- make sure you tab with a space after `path`
//! arg-completion=> nested path
//! arg-completion.nested => path
//! arg-completion=> no-complete  <-- won't see any completions!

use cmdtree::completion::*;
use cmdtree::{Builder, BuilderChain};

fn main() {
    let cmder = Builder::default_config("arg-completion")
        .add_action("path", "complete path names", |_, _| ())
        .add_action("no-complete", "", |_, _| ())
        .begin_class("nested", "")
        .add_action("path", "", |_, _| ())
        .into_commander()
        .unwrap();

    cmder.run_with_completion(|c| ArgCompleter {
        items: create_action_completion_items(c),
    });
}

struct ArgCompleter {
    items: Vec<ActionMatch>,
}

impl<T: Terminal> Completer<T> for ArgCompleter {
    fn complete(
        &self,
        _word: &str,
        prompter: &Prompter<T>,
        _start: usize,
        end: usize,
    ) -> Option<Vec<Completion>> {
        // only want the completion on the 'path' action
        // notice the syntax for qualified paths.
        let actions = [".path", "nested..path"];

        let line = prompter.buffer();

		// retrieve the matched action is there is one.
		// also check that the action is one that should be completed on, using qualified path
        let action_match = self.items.iter().find(|x| {
            line.starts_with(x.match_str.as_str()) && actions.contains(&x.qualified_path.as_str())
        })?;

		// going to use this logic for compeletions, but can implement own
		let path_completer = linefeed::complete::PathCompleter;

		let arg_line = &line[action_match.match_str.len()..];	// portion of line just for args
		let word_start = linefeed::complete::word_break_start(arg_line, " ");	// break on spaces
		let word = &arg_line[word_start..];

		path_completer.complete(word, prompter, word_start, end)
    }
}
