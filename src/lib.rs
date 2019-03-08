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
mod parse;

pub use self::builder::CommanderBuilder;
use self::parse::WordResult;

pub struct Commander<'a> {
	root: SubClass<'a>,
}

pub struct SubClass<'a> {
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
	pub fn run(self) {
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
						next_word = match parse::parse_word(current_class, word) {
							WordResult::Help(sc) => {
								print_help(&sc);
								current_class = start_class;
								None
							}
							WordResult::Cancel => {
								current_class = &self.root;
								None
							}
							WordResult::Exit => {
								exit = true;
								None
							}
							WordResult::Class(sc) => {
								current_class = sc;
								words_iter.next()
							}
							WordResult::Action(a) => {
								let slice = &words[idx..];
								a.call(slice);
								current_class = start_class;
								None
							}
							WordResult::Unrecognized => {
								println!(
									"{}",
									format!(
										"'{}' does not match any keywords, classes, or actions",
										word
									)
									.bright_red()
								);
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
