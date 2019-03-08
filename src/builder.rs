use super::*;

pub struct Builder<'a> {
	parents: Vec<SubClass<'a>>,
	current: SubClass<'a>,
}

impl<'a> Builder<'a> {
	pub fn new(root_name: &str) -> Self {
		Builder {
			parents: Vec::new(),
			current: SubClass::with_name(root_name, "base class of commander tree"),
		}
	}

	pub fn begin_class(mut self, name: &str, help_msg: &'a str) -> Result<Self, BuildError> {
		check_names(name, &self.current).map(|_| {
			self.parents.push(self.current);
			self.current = SubClass::with_name(name, help_msg);
			self
		})
	}

	pub fn end_class(mut self) -> Result<Self, &'static str> {
		let mut parent = self
			.parents
			.pop()
			.ok_or("called end_class when there are no parents left")?;
		parent.classes.push(Rc::new(self.current)); // push the child class onto the parent's classes vector
		self.current = parent;
		Ok(self)
	}

	pub fn add_action<F>(
		mut self,
		name: &str,
		help_msg: &'a str,
		closure: F,
	) -> Result<Self, BuildError>
	where
		F: FnMut(&[&str]) + 'a,
	{
		check_names(name, &self.current).map(|_| {
			self.current.actions.push(Action {
				name: name.to_lowercase(),
				help: help_msg,
				closure: RefCell::new(Box::new(closure)),
			});
			self
		})
	}

	pub fn into_commander<'c>(self) -> Commander<'a> {
		let mut root = self;
		while root.parents.len() > 0 {
			root = root.end_class().expect("shouldn't dip below zero parents");
		}
		let rc = Rc::new(root.current);
		Commander {
			root: Rc::clone(&rc),
			current: Rc::clone(&rc),
		}
	}
}

fn check_names(name: &str, subclass: &SubClass) -> Result<(), BuildError> {
	let lwr = name.to_lowercase();
	// check names
	if subclass.actions.iter().any(|x| x.name == lwr) {
		Err(BuildError::NameExistsAsAction)
	} else if subclass.classes.iter().any(|x| x.name == lwr) {
		Err(BuildError::NameExistsAsClass)
	} else {
		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub enum BuildError {
	NameExistsAsClass,
	NameExistsAsAction,
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

	#[test]
	fn check_names_test() {
		let mut sc = SubClass::with_name("name", "adsf");
		assert_eq!(check_names("name1", &sc), Ok(()));
		sc.classes
			.push(Rc::new(SubClass::with_name("sub-name", "asdf")));
		assert_eq!(check_names("name1", &sc), Ok(()));
		assert_eq!(
			check_names("sub-name", &sc),
			Err(BuildError::NameExistsAsClass)
		);
		sc.actions.push(Action {
			name: "name1".to_string(),
			help: "adf",
			closure: RefCell::new(Box::new(|_| ())),
		});
		assert_eq!(
			check_names("name1", &sc),
			Err(BuildError::NameExistsAsAction)
		);
	}

}
