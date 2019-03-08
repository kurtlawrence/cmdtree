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
		colorize: bool,
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
					if colorize {
						write_help_colored(&sc, writer).expect("failed writing output to writer");
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

					if colorize {
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

fn write_help_colored<W: Write>(class: &SubClass, writer: &mut W) -> io::Result<()> {
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
		"{} -- exits the interactive loop",
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
	writeln!(writer, "exit -- exits the interactive loop",)?;
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
	fn parse_word_test() {
		let mut sc = SubClass::with_name("Class-Name", "help msg");
		assert_eq!(parse_word(&sc, "HELP"), WordResult::Help(&sc));
		assert_eq!(parse_word(&sc, "EXIT"), WordResult::Exit);
		assert_eq!(parse_word(&sc, "CANCEL"), WordResult::Cancel);
		assert_eq!(parse_word(&sc, "C"), WordResult::Cancel);
		assert_eq!(parse_word(&sc, "asdf"), WordResult::Unrecognized);

		sc.classes
			.push(Rc::new(SubClass::with_name("name", "asdf")));
		assert_eq!(parse_word(&sc, "NAME"), WordResult::Class(&sc.classes[0]));
	}

}
