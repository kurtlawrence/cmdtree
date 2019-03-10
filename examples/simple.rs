//! Simple example highlighting nesting of commands

extern crate cmdtree;
use cmdtree::*;

fn main() {
	let cmder = Builder::default_config("cmdtree-example")
		.begin_class("class1", "class1 help message")
			.begin_class("inner-class1", "nested class!")
				.add_action("name", "print class name", |_| println!("inner-class1", ))
			.end_class()
		.end_class()
		.into_commander().unwrap();

	cmder.run();	// run interactively
}