[![Build Status](https://travis-ci.com/kurtlawrence/cmdtree.svg?branch=master)](https://travis-ci.com/kurtlawrence/cmdtree)
[![Latest Version](https://img.shields.io/crates/v/cmdtree.svg)](https://crates.io/crates/cmdtree) 
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/cmdtree) 
[![codecov](https://codecov.io/gh/kurtlawrence/cmdtree/branch/master/graph/badge.svg)](https://codecov.io/gh/kurtlawrence/cmdtree)

(Rust) commands tree.

See the [rs docs](https://docs.rs/cmdtree/).
Look at progress and contribute on [github.](https://github.com/kurtlawrence/cmdtree)

# cmdtree

Create a tree-like data structure of commands and actions to add an intuitive and interactive experience to an application.
cmdtree uses a builder pattern to make constructing the tree ergonomic.

# Example

```rust,no_run
extern crate cmdtree;
use cmdtree::*;

fn main() {
  let cmder = Builder::default_config("cmdtree-example")
    .begin_class("class1", "class1 help message") // a class
    .begin_class("inner-class1", "nested class!") // can nest a class
    .add_action("name", "print class name", |mut wtr, _args| {
      writeln!(wtr, "inner-class1",).unwrap()
    })
    .end_class()
    .end_class() // closes out the classes
    .begin_class("print", "pertains to printing stuff") // start another class sibling to `class1`
    .add_action("echo", "repeat stuff", |mut wtr, args| {
      writeln!(wtr, "{}", args.join(" ")).unwrap()
    })
    .add_action("countdown", "countdown from a number", |mut wtr, args| {
      if args.len() != 1 {
        println!("need one number",);
      } else {
        match str::parse::<u32>(args[0]) {
          Ok(n) => {
            for i in (0..=n).rev() {
              writeln!(wtr, "{}", i).unwrap();
            }
          }
          Err(_) => writeln!(wtr, "expecting a number!",).unwrap(),
        }
      }
    })
    .into_commander() // can short-circuit the closing out of classes
    .unwrap();

  cmder.run(); // run interactively
}
```

Now run and in your shell:

```sh
cmdtree-example=> help            <-- Will print help messages
help -- prints the help messages
cancel | c -- returns to the root class
exit -- sends the exit signal to end the interactive loop
Classes:
        class1 -- class1 help message
        print -- pertains to printing stuff
cmdtree-example=> print            <-- Can navigate the tree
cmdtree-example.print=> help
help -- prints the help messages
cancel | c -- returns to the root class
exit -- sends the exit signal to end the interactive loop
Actions:
        echo -- repeat stuff
        countdown -- countdown from a number
cmdtree-example.print=> echo hello, world!  <-- Call the actions
hello, world!
cmdtree-example.print=> countdown
need one number
cmdtree-example.print=> countdown 10
10
9
8
7
6
5
4
3
2
1
0
cmdtree-example.print=> exit      <-- exit the loop!
```