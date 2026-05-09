use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::rc::{Rc, Weak};

#[derive(Clone)]
pub(crate) struct OrderedElement {
	node: Rc<Node>,
	arena: Rc<Arena>,
}

struct Node {
	arena: Weak<Arena>,
	index: Cell<u128>,
	prev: RefCell<Weak<Node>>,
	next: RefCell<Weak<Node>>,
}

struct Arena {
	base: Rc<Node>,
	len: Cell<u64>,
}

impl Node {
	fn get_prev(&self) -> Rc<Node> {
		self.prev.borrow().upgrade().unwrap()
	}

	fn get_next(&self) -> Rc<Node> {
		self.next.borrow().upgrade().unwrap()
	}
}

impl OrderedElement {
	pub(crate) fn new_root() -> Self {
		let arena = Rc::new_cyclic(|arena| {
			let base = Rc::new_cyclic(|base| Node {
				arena: arena.clone(),
				index: Cell::new(0),
				prev: RefCell::new(base.clone()),
				next: RefCell::new(base.clone()),
			});

			Arena {
				base,
				len: Cell::new(1),
			}
		});

		OrderedElement { node: arena.base.clone(), arena }
	}

	pub(crate) fn iota(&self) -> Self {
		let len = self.arena.len.get();

		if len == u64::MAX {
			panic!("OrderedElement arena is full");
		}

		let index = if len == 1 {
			self.node.index.get().wrapping_add(u128::MAX ^ u128::MAX >> 1)
		} else {
			let base_index = self.node.index.get();

			let mut node = self.node.clone();
			let mut j = 1;
			let total_span = loop {
				node = node.get_next();

				let offset = node.index.get().wrapping_sub(base_index);
				if offset > j * j {
					break offset;
				}

				j += 1;
			};

			let divided = total_span / j;
			let remainder = total_span % j;

			let mut first_offset = total_span;

			let mut node = self.node.clone();

			for i in 1 .. j {
				node = node.get_next();

				let offset = divided * i + u128::from(i <= remainder);
				let index = base_index.wrapping_add(offset);
				node.index.set(index);

				if i == 1 {
					first_offset = offset;
				}
			}

			base_index.wrapping_add(first_offset / 2)
		};

		let next = self.node.get_next();

		let new_node = Rc::new(Node {
			arena: Rc::downgrade(&self.arena),
			index: Cell::new(index),
			prev: RefCell::new(Rc::downgrade(&self.node)),
			next: RefCell::new(Rc::downgrade(&next)),
		});

		*self.node.next.borrow_mut() = Rc::downgrade(&new_node);
		*next.prev.borrow_mut() = Rc::downgrade(&new_node);

		self.arena.len.set(len + 1);

		Self { arena: self.arena.clone(), node: new_node }
	}
}

impl Ord for OrderedElement {
	fn cmp(&self, other: &Self) -> Ordering {
		if !Rc::ptr_eq(&self.arena, &other.arena) {
			panic!("attempted to compare incompatible OrderedElements");
		}

		let base = &self.arena.base;

		if Rc::ptr_eq(&base, &self.node) || Rc::ptr_eq(&base, &other.node) {
			panic!("attempted to compare OrderedElement with root");
		}

		let base_index = base.index.get();

		let offset_self = self.node.index.get().wrapping_sub(base_index);
		let offset_other = other.node.index.get().wrapping_sub(base_index);

		offset_self.cmp(&offset_other)
	}
}

impl PartialOrd for OrderedElement {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Eq for OrderedElement {}

impl PartialEq for OrderedElement {
	fn eq(&self, other: &Self) -> bool {
		self.cmp(other).is_eq()
	}
}

impl Drop for OrderedElement {
	fn drop(&mut self) {
		eprintln!("OrderedElement dropped...");
	}
}

impl Drop for Node {
	fn drop(&mut self) {
		eprintln!("Node dropped...");

		let Some(arena) = self.arena.upgrade() else {
			eprintln!("short circuit");
			return;
		};

		arena.len.update(|len| len - 1);

		let prev = self.get_prev();
		let next = self.get_next();

		*prev.next.borrow_mut() = Rc::downgrade(&next);
		*next.prev.borrow_mut() = Rc::downgrade(&prev);
	}
}

impl Drop for Arena {
	fn drop(&mut self) {
		eprintln!("Arena dropped...");
	}
}

#[test]
fn create_and_delete() {
	OrderedElement::new_root();
}

#[test]
fn add_element() {
	let root = OrderedElement::new_root();
	root.iota();
}

#[test]
fn test_chained_iota() {
	let root = OrderedElement::new_root();
	let a = root.iota();
	let b = a.iota();

	assert!(a < b);
}

#[test]
fn test_repeated_iota() {
	let root = OrderedElement::new_root();
	let a = root.iota();
	let b = root.iota();

	assert!(a > b);
}

#[test]
fn test_many_repeated_iotas() {
	let root = OrderedElement::new_root();
	let mut elements = Vec::new();

	for _ in 0 .. 10000 {
		elements.push(root.iota());
	}

	for [a, b] in elements.array_windows() {
		assert!(a > b);
	}
}

#[test]
fn test_many_chained_iotas() {
	let mut element = OrderedElement::new_root();
	let mut elements = Vec::new();

	for _ in 0 .. 10000 {
		let new_element = element.iota();
		elements.push(new_element.clone());
		element = new_element;
	}

	for [a, b] in elements.array_windows() {
		assert!(a < b);
	}
}

#[test]
fn test_equality() {
	let root = OrderedElement::new_root();
	let a = root.iota();

	assert!(a == a);
}

#[test]
#[should_panic]
fn test_compare_with_root() {
	let root = OrderedElement::new_root();
	let a = root.iota();

	let _ = root.cmp(&a);
}

#[test]
#[should_panic]
fn test_compare_between_different_roots() {
	let root_a = OrderedElement::new_root();
	let root_b = OrderedElement::new_root();

	let a = root_a.iota();
	let b = root_b.iota();

	let _ = a.cmp(&b);
}
