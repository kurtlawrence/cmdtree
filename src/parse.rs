use super::*;
use std::io::{self, Write};

#[derive(Debug, PartialEq)]
enum WordResult<'a, 'b> {
    Help(&'b SubClass<'a>),
    Cancel,
    Exit,
    Class(&'b Rc<SubClass<'a>>),
    Action(&'b Action<'a>),
    Unrecognized,
}

pub enum LineResult {
    Success,
    Exit,
}

impl<'r> Commander<'r> {
    pub fn parse_line<W: Write>(
        &mut self,
        line: &str,
        colourise: bool,
        writer: &mut W,
    ) -> LineResult {
        let line = line.replace("\n", "").replace("\r", "");
        let words: Vec<_> = line.trim().split(' ').collect();
        let mut idx = 0;
        let mut words_iter = words.iter();
        let mut next_word = words_iter.next();

        // if there is no current class, use the root
        let mut current_class = Rc::clone(&self.current);
        let start_class = Rc::clone(&current_class);

        while let Some(word) = next_word {
            idx += 1;
            next_word = match parse_word(&current_class, word) {
                WordResult::Help(sc) => {
                    if colourise {
                        write_help_coloured(&sc, writer).expect("failed writing output to writer");
                    } else {
                        write_help(&sc, writer).expect("failed writing output to writer");
                    }
                    self.current = Rc::clone(&start_class);
                    None
                }
                WordResult::Cancel => {
                    self.current = Rc::clone(&self.root);
                    None
                }
                WordResult::Exit => {
                    return LineResult::Exit;
                }
                WordResult::Class(sc) => {
                    current_class = Rc::clone(&sc);
                    words_iter.next()
                }
                WordResult::Action(a) => {
                    let slice = &words[idx..];
                    a.call(slice);
                    self.current = Rc::clone(&start_class);
                    None
                }
                WordResult::Unrecognized => {
                    let mut s = format!(
                        "'{}' does not match any keywords, classes, or actions",
                        word
                    )
                    .bright_red();

                    if colourise {
                        s = s.white();
                    }

                    writeln!(writer, "{}", s).expect("failed writing output to writer");
                    self.current = Rc::clone(&start_class);
                    None
                }
            };
        }

        LineResult::Success
    }
}

fn parse_word<'a, 'b>(subclass: &'b SubClass<'a>, word: &str) -> WordResult<'a, 'b> {
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

fn write_help_coloured<W: Write>(class: &SubClass, writer: &mut W) -> io::Result<()> {
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

fn write_help<W: Write>(class: &SubClass, writer: &mut W) -> io::Result<()> {
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
            .into_commander();

		let w = &mut std::io::stderr();
		
		cmder.parse_line("adsf", true,w);	// unrecognised branch
		assert_eq!(cmder.current, cmder.root);
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
