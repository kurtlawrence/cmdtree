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

use colored::*;
use linefeed::{Interface, ReadResult};
use std::cell::RefCell;

mod builder;

pub use self::builder::CommanderBuilder;

pub struct Commander<'a> {
	root: SubClass<'a>,
}

struct SubClass<'a> {
	name: String,
	help: &'a str,
	classes: Vec<SubClass<'a>>,
	actions: Vec<Action<'a>>,
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

impl<'a> Commander<'a> {
	pub fn run_interactively(self) {
		let interface = Interface::new("commander").expect("failed to start interface");
		let mut current_class = &self.root;
		let mut exit = false;

		while !exit {
			interface
				.set_prompt(&format!("{}=> ", current_class.name.bright_cyan()))
				.expect("failed to set prompt");

			match interface.read_line() {
				Ok(ReadResult::Input(s)) => {
					let words: Vec<_> = s.split(' ').collect();
					let mut idx = 0;
					let mut words_iter = words.iter();
					let mut next_word = words_iter.next();
					let start_class = current_class;

					while let Some(word) = next_word {
						idx += 1;
						next_word = match parse_word(current_class, word) {
							Ok(CommandResult::Help(sc)) => {
								print_help(&sc);
								current_class = start_class;
								None
							}
							Ok(CommandResult::Cancel) => {
								current_class = &self.root;
								None
							}
							Ok(CommandResult::Exit) => {
								exit = true;
								None
							}
							Ok(CommandResult::Class(sc)) => {
								current_class = sc;
								words_iter.next()
							}
							Ok(CommandResult::Action(a)) => {
								let slice = &words[idx..];
								a.call(slice);
								current_class = start_class;
								None
							}
							Err(s) => {
								println!("{}", s.bright_red());
								current_class = start_class;
								None
							}
						};
					}
				}
				_ => (),
			}
		}
	}
}

fn print_help(class: &SubClass) {
	println!("{} -- prints the help messages", "help".bright_yellow());
	println!(
		"{} | {} -- returns to the root class",
		"cancel".bright_yellow(),
		"c".bright_yellow()
	);
	println!("{} -- exits the interactive loop", "exit".bright_yellow());
	if class.classes.len() > 0 {
		println!("{}", "Classes:".bright_purple());
		for class in class.classes.iter() {
			println!("\t{} -- {}", class.name.bright_yellow(), class.help);
		}
	}

	if class.actions.len() > 0 {
		println!("{}", "Actions:".bright_purple());
		for action in class.actions.iter() {
			println!("\t{} -- {}", action.name.bright_yellow(), action.help);
		}
	}
}

fn parse_word<'a, 'b>(
	subclass: &'b SubClass<'a>,
	word: &str,
) -> Result<CommandResult<'a, 'b>, String> {
	let lwr = word.to_lowercase();
	match lwr.as_str() {
		"help" => Ok(CommandResult::Help(subclass)),
		"cancel" | "c" => Ok(CommandResult::Cancel),
		"exit" => Ok(CommandResult::Exit),
		word => {
			if let Some(c) = subclass.classes.iter().find(|c| &c.name == word) {
				Ok(CommandResult::Class(c))
			} else if let Some(a) = subclass.actions.iter().find(|a| &a.name == word) {
				Ok(CommandResult::Action(a))
			} else {
				Err(format!(
					"'{}' does not match any keywords, classes, or actions",
					word
				))
			}
		}
	}
}

enum CommandResult<'a, 'b> {
	Help(&'b SubClass<'a>),
	Cancel,
	Exit,
	Class(&'b SubClass<'a>),
	Action(&'b Action<'a>),
}
