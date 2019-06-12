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

use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;
use std::io::Write;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

pub mod builder;
pub mod completion;
mod parse;

pub use self::parse::LineResult;
pub use builder::{BuildError, Builder, BuilderChain};

/// A constructed command tree.
///
/// Most of the time a user will want to use `run()` which will handle all the parsing and navigating of the tree.
/// Alternatively, `parse_line` can be used to simulate a read input and update the command tree position.
///
/// To construct a command tree, look at the [`builder` module](./builder/index.html).
pub struct Commander<R> {
    root: Arc<SubClass<R>>,
    current: Arc<SubClass<R>>,
    path: String,
}

impl<R> Commander<R> {
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
    ///
    /// This is the most simple way of using a `Commander`.
    #[cfg(feature = "runnable")]
    pub fn run(self) {
        self.run_with_completion(|_| linefeed::complete::DummyCompleter)
    }

    /// Returns the command structure as a sorted set.
    ///
    /// Can return from the the current class or the root.
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
    /// let structure = cmder.structure(true);
    ///
    /// assert_eq!(structure.iter().map(|x| x.path.as_str()).collect::<Vec<_>>(), vec![
    /// 	"..action",
    /// 	"one",
    /// 	"one..action",
    /// 	"one.two",
    /// ]);
    /// ```
    pub fn structure(&self, from_root: bool) -> BTreeSet<StructureInfo> {
        let mut set = BTreeSet::new();

        let mut stack: Vec<(String, _)> = {
            let r = if from_root { &self.root } else { &self.current };

            for action in r.actions.iter() {
                set.insert(StructureInfo {
                    path: format!("..{}", action.name),
                    itemtype: ItemType::Action,
                    help_msg: action.help.clone(),
                });
            }

            r.classes.iter().map(|x| (x.name.clone(), x)).collect()
        };

        while let Some(item) = stack.pop() {
            let (parent_path, parent) = item;

            for action in parent.actions.iter() {
                set.insert(StructureInfo {
                    path: format!("{}..{}", parent_path, action.name),
                    itemtype: ItemType::Action,
                    help_msg: action.help.clone(),
                });
            }

            for class in parent.classes.iter() {
                stack.push((format!("{}.{}", parent_path, class.name), class));
            }

            set.insert(StructureInfo {
                path: parent_path,
                itemtype: ItemType::Class,
                help_msg: parent.help.clone(),
            });
        }

        set
    }
}

#[derive(Debug, Eq)]
struct SubClass<R> {
    name: String,
    help: CmdStr,
    classes: Vec<Arc<SubClass<R>>>,
    actions: Vec<Action<R>>,
}

impl<R> SubClass<R> {
    fn with_name<H: Into<CmdStr>>(name: &str, help_msg: H) -> Self {
        SubClass {
            name: name.to_lowercase(),
            help: help_msg.into(),
            classes: Vec::new(),
            actions: Vec::new(),
        }
    }
}

impl<R> PartialEq for SubClass<R> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.help == other.help
            && self.classes == other.classes
            && self.actions == other.actions
    }
}

struct Action<R> {
    name: String,
    help: CmdStr,
    closure: Mutex<Box<dyn FnMut(&mut dyn Write, &[&str]) -> R + Send>>,
}

impl<R> Action<R> {
    fn call<W: Write>(&self, wtr: &mut W, arguments: &[&str]) -> R {
        let c = &mut *self.closure.lock().expect("locking command action failed");
        c(wtr, arguments)
    }
}

impl Action<()> {
    #[cfg(test)]
    fn blank_fn<H: Into<CmdStr>>(name: &str, help_msg: H) -> Self {
        Action {
            name: name.to_lowercase(),
            help: help_msg.into(),
            closure: Mutex::new(Box::new(|_, _| ())),
        }
    }
}

impl<R> PartialEq for Action<R> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.help == other.help
    }
}

impl<R> Eq for Action<R> {}

impl<R> fmt::Debug for Action<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Action {{ name: {}, help: {} }}", self.name, self.help)
    }
}

/// An item in the command tree.
pub struct StructureInfo {
    /// Period delimited path. Actions are double delimted.
    ///
    /// Eg.
    /// - A class: `a.nested.class`
    /// - An action: `a.nested.class..action`
    pub path: String,
    /// Class or Action.
    pub itemtype: ItemType,
    /// The help message.
    pub help_msg: CmdStr,
}

impl PartialEq for StructureInfo {
    fn eq(&self, other: &StructureInfo) -> bool {
        self.path == other.path
    }
}

impl Eq for StructureInfo {}

impl PartialOrd for StructureInfo {
    fn partial_cmp(&self, other: &StructureInfo) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for StructureInfo {
    fn cmp(&self, other: &StructureInfo) -> Ordering {
        self.path.cmp(&other.path)
    }
}

/// A command type.
#[derive(Debug, PartialEq)]
pub enum ItemType {
    /// Class type.
    Class,
    /// Action type.
    Action,
}

/// A command string can be static or owned.
///
/// Wraps a `Cow<'static, str>`.
/// Implements `From<&'static str>` and `From<String>`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CmdStr {
    inner_cow: Cow<'static, str>,
}

impl CmdStr {
    /// Represent a string.
    pub fn as_str(&self) -> &str {
        &self.inner_cow
    }
}

impl Deref for CmdStr {
    type Target = str;
    fn deref(&self) -> &str {
        self.inner_cow.deref()
    }
}

impl fmt::Display for CmdStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&'static str> for CmdStr {
    fn from(s: &'static str) -> Self {
        Self {
            inner_cow: Cow::Borrowed(s),
        }
    }
}

impl From<String> for CmdStr {
    fn from(s: String) -> Self {
        Self {
            inner_cow: Cow::Owned(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subclass_with_name_test() {
        let sc = SubClass::<()>::with_name("NAME", "Help Message");
        assert_eq!(&sc.name, "name");
        assert_eq!(sc.help.as_str(), "Help Message");
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

    #[test]
    fn structure_test() {
        let mut cmder = Builder::default_config("base")
            .begin_class("one", "")
            .begin_class("two", "")
            .end_class()
            .add_action("action", "", |_, _| ())
            .end_class()
            .add_action("action", "", |_, _| ())
            .into_commander()
            .unwrap();

        cmder.parse_line("one", false, &mut std::io::sink());

        let structure = cmder.structure(true);

        assert_eq!(
            structure
                .iter()
                .map(|x| x.path.as_str())
                .collect::<Vec<_>>(),
            vec!["..action", "one", "one..action", "one.two",]
        );

        let structure = cmder.structure(false);

        assert_eq!(
            structure
                .iter()
                .map(|x| x.path.as_str())
                .collect::<Vec<_>>(),
            vec!["..action", "two",]
        );
    }
}
