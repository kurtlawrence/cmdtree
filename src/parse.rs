use super::*;

pub enum WordResult<'a, 'b> {
	Help(&'b SubClass<'a>),
	Cancel,
	Exit,
	Class(&'b SubClass<'a>),
	Action(&'b Action<'a>),
	Unrecognized,
}

enum LineResult {
	Success,
	Exit,
	PrintHelp,
	Unrecognized,
}

// impl Commander {
// 	fn parse_line(&self, current_class: &SubClass, line: &str) {
// 		let words: Vec<_> = line
// 			.replace("\n", "")
// 			.replace("\r", "")
// 			.trim()
// 			.split(' ')
// 			.collect();
// 		let mut idx = 0;
// 		let mut words_iter = words.iter();
// 		let mut next_word = words_iter.next();
// 		let start_class = current_class;

// 		while let Some(word) = next_word {
// 			idx += 1;
// 			next_word = match parse_word(current_class, word) {
// 				WordResult::Help(sc) => {
// 					print_help(&sc);
// 					current_class = start_class;
// 					None
// 				}
// 				WordResult::Cancel => {
// 					current_class = &self.root;
// 					None
// 				}
// 				WordResult::Exit => {
// 					exit = true;
// 					None
// 				}
// 				WordResult::Class(sc) => {
// 					current_class = sc;
// 					words_iter.next()
// 				}
// 				WordResult::Action(a) => {
// 					let slice = &words[idx..];
// 					a.call(slice);
// 					current_class = start_class;
// 					None
// 				}
// 				WordResult::Unrecognized => {
// 					println!("{}", s.bright_red());
// 					current_class = start_class;
// 					None
// 				}
// 			};
// 		}
// 	}
// }

pub fn parse_word<'a, 'b>(subclass: &'b SubClass<'a>, word: &str) -> WordResult<'a, 'b> {
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_word_test() {
		let sc = SubClass::with_name("Class-Name", "help msg");
	}
}
