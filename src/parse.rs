use super::*;
use std::io::{self, Write};

const PATH_SEP: char = '.';

#[derive(Debug, PartialEq)]
enum WordResult<'a, 'b, R> {
    Help(&'b SubClass<'a, R>),
    Cancel,
    Exit,
    Class(&'b Rc<SubClass<'a, R>>),
    Action(&'b Action<'a, R>),
    Unrecognized,
}

#[derive(Debug, PartialEq)]
pub enum LineResult<R> {
    Help,
    Cancel,
    Exit,
    Class,
    Action(R),
    Unrecognized,
}

impl<'r, R> Commander<'r, R> {
    /// Parse a line of commands and updates the `Commander` state.
    ///
    /// Parsing a line is akin to sending an input line to the commander in the run loop.
    /// Commands are space separated, and executed within this function, any actions that are specified will be invoked.
    ///
    /// Most branches result in a `LineResult::Continue` apart from an exit command which will result in a `LineResult::Exit`.
    /// It is up to the developer to decide on the behaviour.
    ///
    /// # Example
    /// ```rust
    /// use cmdtree::*;
    /// let mut cmder = Builder::default_config("base")
    ///		.begin_class("one", "")
    ///		.begin_class("two", "")
    /// 	.add_action("echo", "", |args| println!("{}", args.join(" ")))
    ///		.into_commander().unwrap();
    ///
    ///	assert_eq!(cmder.path(), "base");
    ///	cmder.parse_line("one two", true,  &mut std::io::sink());
    ///	assert_eq!(cmder.path(), "base.one.two");
    /// cmder.parse_line("echo Hello, world!", true, &mut std::io::sink());	// should print "Hello, world!"
    /// ```
    pub fn parse_line<W: Write>(
        &mut self,
        line: &str,
        colourise: bool,
        writer: &mut W,
    ) -> LineResult<R> {
        let line = line.replace("\n", "").replace("\r", "");
        let words: Vec<_> = line.trim().split(' ').collect();
        let mut idx = 0;
        let mut words_iter = words.iter();
        let mut next_word = words_iter.next();

        // if there is no current class, use the root
        let start_class = Rc::clone(&self.current);
        let start_path = self.path.clone();

        while let Some(word) = next_word {
            idx += 1;
            next_word = match parse_word(&self.current, word) {
                WordResult::Help(sc) => {
                    if colourise {
                        write_help_coloured(&sc, writer).expect("failed writing output to writer");
                    } else {
                        write_help(&sc, writer).expect("failed writing output to writer");
                    }
                    self.current = Rc::clone(&start_class);
                    self.path = start_path.clone();
                    return LineResult::Help;
                }
                WordResult::Cancel => {
                    self.current = Rc::clone(&self.root);
                    self.path = self.root.name.clone();
                    return LineResult::Cancel;
                }
                WordResult::Exit => {
                    return LineResult::Exit;
                }
                WordResult::Class(sc) => {
                    self.path.push_str(&format!("{}{}", PATH_SEP, sc.name));
                    self.current = Rc::clone(&sc);
                    words_iter.next()
                }
                WordResult::Action(a) => {
                    let slice = &words[idx..];
                    let r = a.call(slice);
                    self.current = Rc::clone(&start_class);
                    self.path = start_path.clone();
                    return LineResult::Action(r);
                }
                WordResult::Unrecognized => {
                    let mut s = format!(
                        "'{}' does not match any keywords, classes, or actions",
                        word
                    )
                    .bright_red();

                    if !colourise {
                        s = s.white();
                    }

                    writeln!(writer, "{}", s).expect("failed writing output to writer");
                    self.current = Rc::clone(&start_class);
                    self.path = start_path.clone();
                    return LineResult::Unrecognized;
                }
            };
        }

        LineResult::Class // default
    }
}

fn parse_word<'a, 'b, R>(subclass: &'b SubClass<'a, R>, word: &str) -> WordResult<'a, 'b, R> {
    let lwr = word.to_lowercase();
    match lwr.as_str() {
        "help" => WordResult::Help(subclass),
        "cancel" | "c" => WordResult::Cancel,
        "exit" => WordResult::Exit,
        word => {
            if let Some(c) = subclass.classes.iter().find(|c| &c.name == word) {
                WordResult::Class(c)
            } else if let Some(a) = subclass.actions.iter().find(|a| &a.name == word) {
                WordResult::Action(a)
            } else {
                WordResult::Unrecognized
            }
        }
    }
}

fn write_help_coloured<W: Write, R>(class: &SubClass<'_, R>, writer: &mut W) -> io::Result<()> {
    writeln!(
        writer,
        "{} -- prints the help messages",
        "help".bright_yellow()
    )?;
    writeln!(
        writer,
        "{} | {} -- returns to the root class",
        "cancel".bright_yellow(),
        "c".bright_yellow()
    )?;
    writeln!(
        writer,
        "{} -- sends the exit signal to end the interactive loop",
        "exit".bright_yellow()
    )?;
    if class.classes.len() > 0 {
        writeln!(writer, "{}", "Classes:".bright_purple())?;
        for class in class.classes.iter() {
            writeln!(writer, "\t{} -- {}", class.name.bright_yellow(), class.help)?;
        }
    }

    if class.actions.len() > 0 {
        writeln!(writer, "{}", "Actions:".bright_purple())?;
        for action in class.actions.iter() {
            writeln!(
                writer,
                "\t{} -- {}",
                action.name.bright_yellow(),
                action.help
            )?;
        }
    }

    Ok(())
}

fn write_help<W: Write, R>(class: &SubClass<'_, R>, writer: &mut W) -> io::Result<()> {
    writeln!(writer, "help -- prints the help messages",)?;
    writeln!(writer, "cancel | c -- returns to the root class",)?;
    writeln!(
        writer,
        "exit -- sends the exit signal to end the interactive loop",
    )?;
    if class.classes.len() > 0 {
        writeln!(writer, "{}", "Classes:")?;
        for class in class.classes.iter() {
            writeln!(writer, "\t{} -- {}", class.name, class.help)?;
        }
    }

    if class.actions.len() > 0 {
        writeln!(writer, "{}", "Actions:")?;
        for action in class.actions.iter() {
            writeln!(writer, "\t{} -- {}", action.name, action.help)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_test() {
        let mut cmder = Builder::default_config("test")
            .begin_class("class1", "class1 help")
            .begin_class("class1-class1", "adsf")
            .add_action("action1", "adf", |_| ())
            .end_class()
            .begin_class("class1-class2", "adsf")
            .add_action("action2", "adsf", |_| ())
            .end_class()
            .end_class()
            .begin_class("class2", "asdf")
            .end_class()
            .add_action("test-args", "", |args| {
                assert_eq!(&args, &["one", "two", "three"])
            })
            .into_commander()
            .unwrap();

        let w = &mut std::io::sink();

        assert_eq!(cmder.parse_line("adsf", true, w), LineResult::Unrecognized); // unrecognised branch
        assert_eq!(cmder.current, cmder.root);
        assert_eq!(cmder.parse_line("adsf", false, w), LineResult::Unrecognized); // unrecognised branch
        assert_eq!(cmder.current, cmder.root);

        assert_eq!(cmder.parse_line("class1", true, w), LineResult::Class);
        assert_ne!(cmder.current, cmder.root);
        assert_eq!(cmder.current.name, "class1");

        // should be able to action here
        assert_eq!(
            cmder.parse_line("class1-class1 action1", true, w),
            LineResult::Action(())
        );
        assert_eq!(cmder.current.name, "class1");
        assert_eq!(
            cmder.parse_line("class1-class2 action2", true, w),
            LineResult::Action(())
        );
        assert_eq!(cmder.current.name, "class1");

        // get back to root
        assert_eq!(cmder.parse_line("cancel", true, w), LineResult::Cancel);
        assert_eq!(cmder.current.name, "test");

        // test args
        assert_eq!(
            cmder.parse_line("test-args one two three", true, w),
            LineResult::Action(())
        );
        assert_eq!(cmder.current.name, "test");

        // test help
        assert_eq!(cmder.parse_line("help", true, w), LineResult::Help);
        assert_eq!(cmder.current.name, "test");
        assert_eq!(cmder.parse_line("help", false, w), LineResult::Help);
        assert_eq!(cmder.current.name, "test");

        // test exit
        assert_eq!(cmder.parse_line("exit", true, w), LineResult::Exit);
    }

    #[test]
    fn parse_word_test() {
        let mut sc = SubClass::with_name("Class-Name", "help msg");
        assert_eq!(parse_word(&sc, "HELP"), WordResult::Help(&sc));
        assert_eq!(parse_word(&sc, "EXIT"), WordResult::Exit);
        assert_eq!(parse_word(&sc, "CANCEL"), WordResult::Cancel);
        assert_eq!(parse_word(&sc, "C"), WordResult::Cancel);
        assert_eq!(parse_word(&sc, "asdf"), WordResult::Unrecognized);

        sc.classes
            .push(Rc::new(SubClass::with_name("name", "asdf")));
        sc.actions.push(Action::blank_fn("action", "adsf"));
        assert_eq!(parse_word(&sc, "NAME"), WordResult::Class(&sc.classes[0]));
        assert_eq!(
            parse_word(&sc, "aCtIoN"),
            WordResult::Action(&sc.actions[0])
        );
    }

    #[test]
    fn write_help_coloured_test() {
        let mut sc = SubClass::with_name("Class-Name", "root class");
        sc.classes
            .push(Rc::new(SubClass::with_name("class1", "class 1 help")));
        sc.classes
            .push(Rc::new(SubClass::with_name("class2", "class 2 help")));
        sc.actions
            .push(Action::blank_fn("action1", "action 1 help"));
        sc.actions
            .push(Action::blank_fn("action2", "action 2 help"));

        let mut help = Vec::new();
        write_help_coloured(&sc, &mut help).unwrap();
        let help = String::from_utf8_lossy(&help);

        assert_eq!(
            &help,
            &format!(
                r#"{} -- prints the help messages
{} | {} -- returns to the root class
{} -- sends the exit signal to end the interactive loop
{}
	{} -- class 1 help
	{} -- class 2 help
{}
	{} -- action 1 help
	{} -- action 2 help
"#,
                "help".bright_yellow(),
                "cancel".bright_yellow(),
                "c".bright_yellow(),
                "exit".bright_yellow(),
                "Classes:".bright_purple(),
                "class1".bright_yellow(),
                "class2".bright_yellow(),
                "Actions:".bright_purple(),
                "action1".bright_yellow(),
                "action2".bright_yellow()
            )
        );
    }

    #[test]
    fn write_help_test() {
        let mut sc = SubClass::with_name("Class-Name", "root class");
        sc.classes
            .push(Rc::new(SubClass::with_name("class1", "class 1 help")));
        sc.classes
            .push(Rc::new(SubClass::with_name("class2", "class 2 help")));
        sc.actions
            .push(Action::blank_fn("action1", "action 1 help"));
        sc.actions
            .push(Action::blank_fn("action2", "action 2 help"));

        let mut help = Vec::new();
        write_help(&sc, &mut help).unwrap();
        let help = String::from_utf8_lossy(&help);

        assert_eq!(
            &help,
            r#"help -- prints the help messages
cancel | c -- returns to the root class
exit -- sends the exit signal to end the interactive loop
Classes:
	class1 -- class 1 help
	class2 -- class 2 help
Actions:
	action1 -- action 1 help
	action2 -- action 2 help
"#
        );
    }

}
