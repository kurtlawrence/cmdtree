//! ([![Build Status](https://travis-ci.com/kurtlawrence/cmdtree.svg?branch=master)](https://travis-ci.com/kurtlawrence/cmdtree)
//! [![Latest Version](https://img.shields.io/crates/v/cmdtree.svg)](https://crates.io/crates/cmdtree)
//! [![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/cmdtree)
//! [![codecov](https://codecov.io/gh/kurtlawrence/cmdtree/branch/master/graph/badge.svg)](https://codecov.io/gh/kurtlawrence/cmdtree)
//!
//! (Rust) commands tree.
//!
//! See the [rs docs](https://docs.rs/cmdtree/).
//! Look at progress and contribute on [github.](https://github.com/kurtlawrence/cmdtree)
//!
//! Currently WIP placeholder.

#![warn(missing_docs)]

use colored::*;
use linefeed::{Interface, ReadResult};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

mod builder;
mod parse;

use self::parse::LineResult;
pub use builder::Builder;

pub struct Commander<'r> {
	root: Rc<SubClass<'r>>,
	current: Rc<SubClass<'r>>,
}

impl<'r> Commander<'r> {
	pub fn prompt(&self) -> &str {
		&self.current.name
	}

	pub fn run(mut self) {
		let interface = Interface::new("commander").expect("failed to start interface");
		let mut exit = false;

		while !exit {
			interface
				.set_prompt(&format!("{}=> ", self.prompt().bright_cyan()))
				.expect("failed to set prompt");

			match interface.read_line() {
				Ok(ReadResult::Input(s)) => match self.parse_line(&s, true, &mut std::io::stdout())
				{
					LineResult::Exit => exit = true,
					_ => (),
				},
				_ => (),
			}
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct SubClass<'a> {
	name: String,
	help: &'a str,
	classes: Vec<Rc<SubClass<'a>>>,
	actions: Vec<Action<'a>>,
}

impl<'a> SubClass<'a> {
	fn with_name(name: &str, help_msg: &'a str) -> Self {
		SubClass {
			name: name.to_lowercase(),
			help: help_msg,
			classes: Vec::new(),
			actions: Vec::new(),
		}
	}
}

struct Action<'a> {
	name: String,
	help: &'a str,
	closure: RefCell<Box<FnMut(&[&str]) + 'a>>,
}

impl<'a> Action<'a> {
	fn call(&self, arguments: &[&str]) {
		let c = &mut *self.closure.borrow_mut();
		c(arguments);
	}
}

impl<'a> PartialEq for Action<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name && self.help == other.help
	}
}

impl<'a> fmt::Debug for Action<'a> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "Action {{ name: {}, help: {} }}", self.name, self.help)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn subclass_with_name_test() {
		let sc = SubClass::with_name("NAME", "Help Message");
		assert_eq!(&sc.name, "name");
		assert_eq!(sc.help, "Help Message");
	}

}
