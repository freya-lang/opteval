use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::rc::{Rc, Weak};

#[derive(Clone)]
pub(crate) struct OrderedElement {
	inner: Rc<Node>,
}

struct Node {
	owner: Rc<Arena>,
	index: Cell<u128>,
	prev: RefCell<Weak<Node>>,
	next: RefCell<Weak<Node>>,
}

struct Arena {
	base: RefCell<Option<Rc<Node>>>,
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
		let arena = Arena {
			base: RefCell::new(None),
			len: Cell::new(1),
		};
		let node = Rc::new_cyclic(|weak| Node {
			owner: Rc::new(arena),
			index: Cell::new(0),
			prev: RefCell::new(weak.clone()),
			next: RefCell::new(weak.clone()),
		});

		*node.owner.base.borrow_mut() = Some(node.clone());

		OrderedElement { inner: node }
	}

	pub(crate) fn iota(&self) -> Self {
		let len = self.inner.owner.len.get();

		if len == u64::MAX {
			panic!("OrderedElement arena is full");
		}

		let index = if len == 1 {
			self.inner.index.get().wrapping_add(u128::MAX ^ u128::MAX >> 1)
		} else {
			let base_index = self.inner.index.get();

			let mut node = self.inner.clone();
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

			let mut first_index = base_index.wrapping_add(total_span);

			let mut node = self.inner.clone();

			for i in 1 .. j {
				node = node.get_next();

				let index = base_index.wrapping_add(divided * i + u128::from(i <= remainder));
				node.index.set(index);

				if i == 1 {
					first_index = index;
				}
			}

			first_index / 2
		};

		let next = self.inner.get_next();

		let new_node = Rc::new(Node {
			owner: self.inner.owner.clone(),
			index: Cell::new(index),
			prev: RefCell::new(Rc::downgrade(&self.inner)),
			next: RefCell::new(Rc::downgrade(&next)),
		});

		*self.inner.next.borrow_mut() = Rc::downgrade(&new_node);
		*next.prev.borrow_mut() = Rc::downgrade(&new_node);

		Self { inner: new_node }
	}
}

impl Ord for OrderedElement {
	fn cmp(&self, other: &Self) -> Ordering {
		if !Rc::ptr_eq(&self.inner.owner, &other.inner.owner) {
			panic!("attempted to compare incompatible OrderedElements");
		}

		let base_ref = self.inner.owner.base.borrow();
		let base = base_ref.as_ref().unwrap();

		if Rc::ptr_eq(&base, &self.inner) || Rc::ptr_eq(&base, &other.inner) {
			panic!("attempted to compare OrderedElement with root");
		}

		let base_index = base.index.get();

		let offset_self = self.inner.index.get().wrapping_sub(base_index);
		let offset_other = other.inner.index.get().wrapping_sub(base_index);

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

impl Drop for Node {
	fn drop(&mut self) {
		let mut len = self.owner.len.get();

		if len == 1 {
			return;
		}

		len -= 1;

		self.owner.len.set(len);

		if len == 1 {
			let mut base_ref = self.owner.base.borrow_mut();

			match Rc::try_unwrap(base_ref.take().unwrap()) {
				Ok(_) => return,
				Err(base) => {
					*base_ref = Some(base);
				},
			}
		}

		let prev = self.get_prev();
		let next = self.get_next();

		*prev.next.borrow_mut() = Rc::downgrade(&next);
		*next.prev.borrow_mut() = Rc::downgrade(&prev);
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
