use std::cmp::Ordering;

use crate::vm::oracle::order::OrderedElement;

#[derive(Clone)]
pub(crate) struct Scope {
	start: OrderedElement,
	end: OrderedElement,
}

impl Scope {
	pub(crate) fn new_root() -> Self {
		let base = OrderedElement::new_base();

		let start = base.iota();
		let end = start.iota();

		Self { start, end }
	}

	pub(crate) fn new_child(&self) -> Self {
		let start = self.start.iota();
		let end = start.iota();

		Self { start, end }
	}

	pub(crate) fn is_directly_related(&self, other: &Self) -> bool {
		match self.start.cmp(&other.start) {
			Ordering::Less => match self.end.cmp(&other.end) {
				Ordering::Less => false,
				Ordering::Equal => unreachable!(),
				Ordering::Greater => true,
			},
			Ordering::Equal => {
				debug_assert!(self.end == other.end);

				true
			},
			Ordering::Greater => match self.end.cmp(&other.end) {
				Ordering::Less => true,
				Ordering::Equal => unreachable!(),
				Ordering::Greater => false,
			},
		}
	}
}

#[test]
fn basic_operations() {
	let root = Scope::new_root();

	let a = root.new_child();
	let b = root.new_child();

	assert!(root.is_directly_related(&a));
	assert!(root.is_directly_related(&b));

	assert!(!a.is_directly_related(&b));

	let aa = a.new_child();

	assert!(root.is_directly_related(&aa));

	drop(a);

	assert!(root.is_directly_related(&aa));
	assert!(!aa.is_directly_related(&b));
}
