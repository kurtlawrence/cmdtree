//! Builder Pattern
//!
//! To construct a `Commander` a `Builder` is used. It allows chaining together the common actions, whilst also construct the structure of the tree in an ergonomic manner.
//! The builder pattern is supported by the [`BuilderChain`](./trait.BuilderChain.html) trait, which is implemented on the `Builder` struct, and also the common result type `BuilderResult`.
//! This allows for chaining methods without needing to intersperse `.unwrap()` or `.expect()` calls everywhere.
//!
//! # Example
//!
//! ```rust
//! use cmdtree::*;
//!
//! let cmder = Builder::default_config("cmdtree-example")
//!     .begin_class("class1", "class1 help message")
//!         .begin_class("inner-class1", "nested class!")
//!             .add_action("name", "print class name", |_, _| println!("inner-class1", ))
//!         .end_class()
//!     .end_class()
//!     .into_commander().unwrap();
//! ```
use super::*;

/// The persistent `Builder` structure to construct a `Commander` command tree.
/// See module level documentation for more information.
#[derive(Debug, PartialEq)]
pub struct Builder<R> {
    parents: Vec<SubClass<R>>,
    current: SubClass<R>,
}

/// The common functions across a `Builder` or a `BuilderResult`.
/// See module level documentation for more information.
pub trait BuilderChain<R> {
    /// Start a new nested class. If the name already exists a `BuildError` will be returned.
    fn begin_class<H: Into<CmdStr>>(self, name: &str, help_msg: H) -> BuilderResult<R>;
    /// Close a class and move to it's parent.
    /// If no parent exists (this function is called on the root), a `BuildError` will be returned.
    fn end_class(self) -> BuilderResult<R>;
    /// Add an action. The closure type gives the arguments after the action command as an array of strings.
    fn add_action<H: Into<CmdStr>, F: FnMut(&mut dyn Write, &[&str]) -> R + Send + 'static>(
        self,
        name: &str,
        help_msg: H,
        closure: F,
    ) -> BuilderResult<R>;

    /// Navigates to the root class, closing out the classes as it goes.
    fn root(self) -> BuilderResult<R>;

    /// Finishes the construction of the command tree and returns the build `Commander`.
    ///
    /// # Note
    /// This can be called even when not on a root class. The implmentation will continue to call `end_class` until the root is reached,
    /// short-circuiting the closing out process.
    ///
    /// If an error is propogating through then the function will error. If there was no error (ie `into_commander` was called on a `Builder` instance)
    /// then this function should not fail.
    fn into_commander(self) -> Result<Commander<R>, BuildError>;
}

/// The common result of `BuilderChain` functions.
pub type BuilderResult<R> = Result<Builder<R>, BuildError>;

impl Builder<()> {
    /// Initialise a `Builder` instance with the given root name.
    pub fn default_config(root_name: &str) -> Self {
        Builder::<()>::new(root_name)
    }
}

impl<R> Builder<R> {
    /// Initialise new `Builder` instance with no configuration.
    pub fn new(root_name: &str) -> Self {
        Builder {
            parents: Vec::new(),
            current: SubClass::with_name(root_name, "base class of commander tree"),
        }
    }
}

impl<R> BuilderChain<R> for Builder<R> {
    fn begin_class<H: Into<CmdStr>>(mut self, name: &str, help_msg: H) -> BuilderResult<R> {
        check_names(name, &self.current).map(|_| {
            self.parents.push(self.current);
            self.current = SubClass::with_name(name, help_msg);
            self
        })
    }

    fn end_class(mut self) -> BuilderResult<R> {
        let mut parent = self.parents.pop().ok_or(BuildError::NoParent)?;
        parent.classes.push(Arc::new(self.current)); // push the child class onto the parent's classes vector
        self.current = parent;
        Ok(self)
    }

    fn root(self) -> BuilderResult<R> {
        let mut root = self;
        while !root.parents.is_empty() {
            root = root.end_class().expect("shouldn't dip below zero parents");
        }
        Ok(root)
    }

    fn add_action<H, F>(mut self, name: &str, help_msg: H, closure: F) -> BuilderResult<R>
    where
        H: Into<CmdStr>,
        F: FnMut(&mut dyn Write, &[&str]) -> R + Send + 'static,
    {
        check_names(name, &self.current).map(|_| {
            self.current.actions.push(Action {
                name: name.to_lowercase(),
                help: help_msg.into(),
                closure: Mutex::new(Box::new(closure)),
            });
            self
        })
    }

    fn into_commander<'c>(self) -> Result<Commander<R>, BuildError> {
        let root = self.root()?;
        let rc = Arc::new(root.current);
        Ok(Commander {
            root: Arc::clone(&rc),
            current: Arc::clone(&rc),
            path: rc.name.to_string(),
        })
    }
}

impl<R> BuilderChain<R> for BuilderResult<R> {
    fn begin_class<H: Into<CmdStr>>(self, name: &str, help_msg: H) -> BuilderResult<R> {
        self?.begin_class(name, help_msg)
    }

    fn end_class(self) -> BuilderResult<R> {
        self?.end_class()
    }

    fn root(self) -> BuilderResult<R> {
        self?.root()
    }

    fn add_action<H, F>(self, name: &str, help_msg: H, closure: F) -> BuilderResult<R>
    where
        H: Into<CmdStr>,
        F: FnMut(&mut dyn Write, &[&str]) -> R + Send + 'static,
    {
        self?.add_action(name, help_msg, closure)
    }

    fn into_commander<'c>(self) -> Result<Commander<R>, BuildError> {
        self?.into_commander()
    }
}

fn check_names<R>(name: &str, subclass: &SubClass<R>) -> Result<(), BuildError> {
    let lwr = name.to_lowercase();
    // check names
    if lwr == "help"
        || lwr == "cancel"
        || lwr == "c"
        || lwr == "exit"
        || subclass.actions.iter().any(|x| x.name == lwr)
    {
        Err(BuildError::NameExistsAsAction)
    } else if subclass.classes.iter().any(|x| x.name == lwr) {
        Err(BuildError::NameExistsAsClass)
    } else {
        Ok(())
    }
}

/// Error variants when building a `Commander`.
#[derive(Debug, PartialEq)]
pub enum BuildError {
    /// The name already exists as a class.
    NameExistsAsClass,
    /// The name already exists as an action.
    NameExistsAsAction,
    /// Tried to get to a parent when none exists.
    /// This usually occurs when `end_class` is called too many times.
    NoParent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_names_test() {
        let mut sc = SubClass::with_name("name", "adsf");
        assert_eq!(check_names("name1", &sc), Ok(()));
        sc.classes
            .push(Arc::new(SubClass::with_name("sub-name", "asdf")));
        assert_eq!(check_names("name1", &sc), Ok(()));
        assert_eq!(
            check_names("sub-name", &sc),
            Err(BuildError::NameExistsAsClass)
        );
        sc.actions.push(Action {
            name: "name1".to_string(),
            help: "adf".into(),
            closure: Mutex::new(Box::new(|_, _| ())),
        });
        assert_eq!(
            check_names("name1", &sc),
            Err(BuildError::NameExistsAsAction)
        );
    }

    #[test]
    fn no_help_cancel_or_exit_classes() {
        let cmdr = Builder::default_config("adf").begin_class("help", "shouldn't work");
        assert_eq!(cmdr, Err(BuildError::NameExistsAsAction));
    }

    #[test]
    fn builder_root_test() {
        let cmdr = Builder::default_config("root")
            .begin_class("adsf", "adf")
            .begin_class("adsf", "adsf")
            .begin_class("asdf", "adsf")
            .root()
            .unwrap();
        assert_eq!(cmdr.parents.len(), 0);
        assert_eq!(cmdr.current.name, "root");
    }
}
