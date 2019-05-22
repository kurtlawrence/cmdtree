//! Simple example highlighting nesting of commands

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
