//! [![Build Status](https://travis-ci.com/kurtlawrence/cmdtree.svg?branch=master)](https://travis-ci.com/kurtlawrence/cmdtree)
//! [![Latest Version](https://img.shields.io/crates/v/cmdtree.svg)](https://crates.io/crates/cmdtree)
//! [![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/cmdtree)
//! [![codecov](https://codecov.io/gh/kurtlawrence/cmdtree/branch/master/graph/badge.svg)](https://codecov.io/gh/kurtlawrence/cmdtree)
//!
//! (Rust) commands tree.
//!
//! See the [rs docs](https://docs.rs/cmdtree/).
//! Look at progress and contribute on [github.](https://github.com/kurtlawrence/cmdtree)
//!
//! # cmdtree
//!
//! Create a tree-like data structure of commands and actions to add an intuitive and interactive experience to an application.
//! cmdtree uses a builder pattern to make constructing the tree ergonomic.
//!
//! # Example
//!
//! ```rust,no_run
//! extern crate cmdtree;
//! use cmdtree::*;
//!
//! fn main() {
//!   let cmder = Builder::default_config("cmdtree-example")
//!     .begin_class("class1", "class1 help message") // a class
//!     .begin_class("inner-class1", "nested class!") // can nest a class
//!     .add_action("name", "print class name", |mut wtr, _args| {
//!       writeln!(wtr, "inner-class1",).unwrap()
//!     })
//!     .end_class()
//!     .end_class() // closes out the classes
//!     .begin_class("print", "pertains to printing stuff") // start another class sibling to `class1`
//!     .add_action("echo", "repeat stuff", |mut wtr, args| {
//!       writeln!(wtr, "{}", args.join(" ")).unwrap()
//!     })
//!     .add_action("countdown", "countdown from a number", |mut wtr, args| {
//!       if args.len() != 1 {
//!         println!("need one number",);
//!       } else {
//!         match str::parse::<u32>(args[0]) {
//!           Ok(n) => {
//!             for i in (0..=n).rev() {
//!               writeln!(wtr, "{}", i).unwrap();
//!             }
//!           }
//!           Err(_) => writeln!(wtr, "expecting a number!",).unwrap(),
//!         }
//!       }
//!     })
//!     .into_commander() // can short-circuit the closing out of classes
//!     .unwrap();
//!
//!   cmder.run(); // run interactively
//! }
//! ```
//!
//! Now run and in your shell:
//!
//! ```sh
//! cmdtree-example=> help            <-- Will print help messages
//! help -- prints the help messages
//! cancel | c -- returns to the root class
//! exit -- sends the exit signal to end the interactive loop
//! Classes:
//!         class1 -- class1 help message
//!         print -- pertains to printing stuff
//! cmdtree-example=> print            <-- Can navigate the tree
//! cmdtree-example.print=> help
//! help -- prints the help messages
//! cancel | c -- returns to the root class
//! exit -- sends the exit signal to end the interactive loop
//! Actions:
//!         echo -- repeat stuff
//!         countdown -- countdown from a number
//! cmdtree-example.print=> echo hello, world!  <-- Call the actions
//! hello, world!
//! cmdtree-example.print=> countdown
//! need one number
//! cmdtree-example.print=> countdown 10
//! 10
//! 9
//! 8
//! 7
//! 6
//! 5
//! 4
//! 3
//! 2
//! 1
//! 0
//! cmdtree-example.print=> exit      <-- exit the loop!
//! ```

#![warn(missing_docs)]

use colored::*;
use linefeed::{Interface, ReadResult};
use std::collections::BTreeSet;
use std::fmt;
use std::sync::{Arc, Mutex};

pub mod builder;
pub mod completion;
mod parse;

pub use self::parse::LineResult;
pub use builder::{BuildError, Builder, BuilderChain};
pub use std::io::Write;

/// A constructed command tree.
///
/// Most of the time a user will want to use `run()` which will handle all the parsing and navigating of the tree.
/// Alternatively, `parse_line` can be used to simulate a read input and update the command tree position.
///
/// To construct a command tree, look at the [`builder` module](./builder/index.html).
pub struct Commander<'r, R> {
    root: Arc<SubClass<'r, R>>,
    current: Arc<SubClass<'r, R>>,
    path: String,
}

impl<'r, R> Commander<'r, R> {
    /// Return the root name.
    ///
    /// # Example
    /// ```rust
    /// # use cmdtree::*;
    /// let mut cmder = Builder::default_config("base")
    ///		.begin_class("one", "")
    ///		.begin_class("two", "")
    ///		.into_commander().unwrap();
    ///
    ///	assert_eq!(cmder.root_name(), "base");
    /// ```
    pub fn root_name(&self) -> &str {
        &self.root.name
    }

    /// Return the path of the current class, separated by `.`.
    ///
    /// # Example
    /// ```rust
    /// # use cmdtree::*;
    /// let mut cmder = Builder::default_config("base")
    ///		.begin_class("one", "")
    ///		.begin_class("two", "")
    ///		.into_commander().unwrap();
    ///
    ///	assert_eq!(cmder.path(), "base");
    ///	cmder.parse_line("one two", true,  &mut std::io::sink());
    ///	assert_eq!(cmder.path(), "base.one.two");
    /// ```
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns if the commander is sitting at the root class.
    ///
    /// # Example
    /// ```rust
    /// # use cmdtree::*;
    /// let mut cmder = Builder::default_config("base")
    ///		.begin_class("one", "")
    ///		.begin_class("two", "")
    ///		.into_commander().unwrap();
    ///
    ///	assert!(cmder.at_root());
    ///	cmder.parse_line("one two", true,  &mut std::io::sink());
    ///	assert_eq!(cmder.at_root(), false);
    /// ```
    pub fn at_root(&self) -> bool {
        self.current == self.root
    }

    /// Run the `Commander` interactively.
    /// Consumes the instance, and blocks the thread until the loop is exited.
    /// Reads from `stdin` using [`linefeed::Interface`](https://docs.rs/linefeed/0.5.4/linefeed/interface/struct.Interface.html).
    ///
    /// This is the most simple way of using a `Commander`.
    pub fn run(mut self) {
        self.run_with_completion(|_| linefeed::complete::DummyCompleter)
    }

    /// Returns the command structure as a sorted set.
    ///
    /// Each item is a dot separated path, except for actions which are separated by a double dot.
    ///
    /// # Examples
    /// ```rust
    /// # use cmdtree::*;
    /// let cmder = Builder::default_config("base")
    ///		.begin_class("one", "")
    ///		.begin_class("two", "")
    /// 	.end_class()
    /// 	.add_action("action", "", |_,_| ())
    /// 	.end_class()
    /// 	.add_action("action", "", |_,_| ())
    ///		.into_commander().unwrap();
    ///
    /// let structure = cmder.structure();
    ///
    /// assert_eq!(structure.iter().map(|x| x.as_str()).collect::<Vec<_>>(), vec![
    /// 	"base",
    /// 	"base..action",
    /// 	"base.one",
    /// 	"base.one..action",
    /// 	"base.one.two",
    /// ]);
    /// ```
    pub fn structure(&self) -> BTreeSet<String> {
        let mut set = BTreeSet::new();

        let mut stack = vec![(self.root.name.clone(), &self.root)];

        while let Some(item) = stack.pop() {
            let (parent_path, parent) = item;

            for action in parent.actions.iter() {
                set.insert(format!("{}..{}", parent_path, action.name));
            }

            for class in parent.classes.iter() {
                stack.push((format!("{}.{}", parent_path, class.name), class));
            }

            set.insert(parent_path);
        }

        set
    }
}

#[derive(Debug, Eq)]
struct SubClass<'a, R> {
    name: String,
    help: &'a str,
    classes: Vec<Arc<SubClass<'a, R>>>,
    actions: Vec<Action<'a, R>>,
}

impl<'a, R> SubClass<'a, R> {
    fn with_name(name: &str, help_msg: &'a str) -> Self {
        SubClass {
            name: name.to_lowercase(),
            help: help_msg,
            classes: Vec::new(),
            actions: Vec::new(),
        }
    }
}

impl<'a, R> PartialEq for SubClass<'a, R> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.help == other.help
            && self.classes == other.classes
            && self.actions == other.actions
    }
}

struct Action<'a, R> {
    name: String,
    help: &'a str,
    closure: Mutex<Box<for<'w> FnMut(Box<Write + 'w>, &[&str]) -> R + Send + 'a>>,
}

impl<'a, R> Action<'a, R> {
    fn call<W: Write>(&self, wtr: &mut W, arguments: &[&str]) -> R {
        let c = &mut *self.closure.lock().expect("locking command action failed");
        c(Box::new(wtr), arguments)
    }
}

impl<'a> Action<'a, ()> {
    #[cfg(test)]
    fn blank_fn(name: &str, help_msg: &'a str) -> Self {
        Action {
            name: name.to_lowercase(),
            help: help_msg,
            closure: Mutex::new(Box::new(|_, _| ())),
        }
    }
}

impl<'a, R> PartialEq for Action<'a, R> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.help == other.help
    }
}

impl<'a, R> Eq for Action<'a, R> {}

impl<'a, R> fmt::Debug for Action<'a, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Action {{ name: {}, help: {} }}", self.name, self.help)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subclass_with_name_test() {
        let sc = SubClass::<()>::with_name("NAME", "Help Message");
        assert_eq!(&sc.name, "name");
        assert_eq!(sc.help, "Help Message");
    }

    #[test]
    fn action_debug_test() {
        let a = Action::blank_fn("action-name", "help me!");
        assert_eq!(
            &format!("{:?}", a),
            "Action { name: action-name, help: help me! }"
        );
    }

    #[test]
    fn current_path_test() {
        let mut cmder = Builder::default_config("base")
            .begin_class("one", "")
            .begin_class("two", "")
            .into_commander()
            .unwrap();

        let w = &mut std::io::sink();

        assert_eq!(cmder.path(), "base");

        cmder.parse_line("one two", true, w);
        assert_eq!(cmder.path(), "base.one.two");

        cmder.parse_line("c", true, w);
        assert_eq!(cmder.path(), "base");

        cmder.parse_line("one", true, w);
        assert_eq!(cmder.path(), "base.one");
    }

    #[test]
    fn root_test() {
        let mut cmder = Builder::default_config("base")
            .begin_class("one", "")
            .begin_class("two", "")
            .into_commander()
            .unwrap();

        let w = &mut std::io::sink();

        assert_eq!(cmder.at_root(), true);

        cmder.parse_line("one two", true, w);
        assert_eq!(cmder.at_root(), false);

        cmder.parse_line("c", true, w);
        assert_eq!(cmder.at_root(), true);
    }
}
