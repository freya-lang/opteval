use std::cell::{Cell, OnceCell};
use std::rc::Rc;

use crate::vm::inet::{Output, encode};

#[derive(Clone)]
pub(crate) enum Term<T> {
	Lambda { body: T },
	Application { left: T, right: T },
	Binding { index: usize },
}

#[derive(Clone)]
pub(crate) struct Strict(Rc<Term<Strict>>);

pub(crate) struct Lazy {
	resolved: OnceCell<Term<Box<Lazy>>>,
	unresolved: Cell<Option<Output>>,
}

impl<A> Term<A> {
	fn as_ref(&self) -> Term<&A> {
		match self {
			Self::Lambda { body } => Term::Lambda { body },
			Self::Application { left, right } => Term::Application { left, right },
			Self::Binding { index } => Term::Binding { index: *index },
		}
	}

	fn map<B>(self, map: impl Fn(A) -> B) -> Term<B> {
		match self {
			Self::Lambda { body } => Term::Lambda { body: map(body) },
			Self::Application { left, right } => Term::Application {
				left: map(left),
				right: map(right),
			},
			Self::Binding { index } => Term::Binding { index },
		}
	}
}

impl Strict {
	pub(crate) fn new(inner: Term<Self>) -> Self {
		Self(Rc::new(inner))
	}

	pub(crate) fn get(&self) -> Term<&Strict> {
		(*self.0).as_ref()
	}
}

impl Lazy {
	fn new(output: Output) -> Self {
		Self {
			resolved: OnceCell::new(),
			unresolved: Cell::new(Some(output)),
		}
	}

	fn resolve(&self) -> &Term<Box<Lazy>> {
		self.resolved.get_or_init(|| {
			let output = self.unresolved.take().unwrap();

			output.pull().map(|new_output| Box::new(Lazy::new(new_output)))
		})
	}

	fn into(mut self) -> Term<Lazy> {
		self.resolve();

		self.resolved.take().unwrap().map(|x| *x)
	}

	pub(crate) fn get(&self) -> Term<&Lazy> {
		self.resolve().as_ref().map(|x| &**x)
	}

	pub(crate) fn to_strict(self) -> Strict {
		Strict(Rc::new(self.into().map(|x| x.to_strict())))
	}

	pub(crate) fn encode(input: &Strict) -> Self {
		Lazy::new(encode(input))
	}
}
