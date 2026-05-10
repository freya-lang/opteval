use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::vm::oracle::hash::hash;
use crate::vm::oracle::scope::Scope;

pub(crate) struct Oracle {
	arena: Rc<Arena>,
}

struct Arena {
	map: RefCell<HashMap<[u8; 32], Weak<TagInner>>>,
	root: Scope,
	counter: Cell<u64>,
}

#[derive(Clone)]
pub(crate) struct Tag {
	inner: Rc<TagInner>,
}

struct TagInner {
	arena: Rc<Arena>,
	id: [u8; 32],
	scope: Scope,
}

fn combine_tags(a: [u8; 32], b: [u8; 32], index: u64) -> [u8; 32] {
	let mut buffer = Vec::new();

	buffer.extend(a);
	buffer.extend(b);
	buffer.extend(index.to_le_bytes());

	hash(buffer)
}

impl Oracle {
	pub(crate) fn new() -> Self {
		let arena = Arena {
			map: RefCell::new(HashMap::new()),
			root: Scope::new_root(),
			counter: Cell::new(0),
		};

		Self { arena: Rc::new(arena) }
	}

	pub(crate) fn new_tag(&self) -> Tag {
		let mut counter = self.arena.counter.get();
		counter = counter.checked_add(1).expect("overflowed tag counter");
		self.arena.counter.set(counter);

		let id = hash(counter.to_le_bytes());

		Tag::new(&self.arena, id, self.arena.root.new_child())
	}
}

impl Tag {
	fn new(arena: &Rc<Arena>, id: [u8; 32], scope: Scope) -> Self {
		let inner = Rc::new(TagInner {
			arena: arena.clone(),
			id,
			scope,
		});

		arena.map.borrow_mut().insert(id, Rc::downgrade(&inner));

		Self { inner }
	}

	pub(crate) fn should_meet(&self, other: &Self) -> bool {
		self.inner.scope.is_directly_related(&other.inner.scope)
	}

	pub(crate) fn combine(&self, other: &Self, index: usize) -> Self {
		let id = combine_tags(self.inner.id, other.inner.id, index.try_into().expect("overflowed u64 in tag index"));

		if let Some(weak) = self.inner.arena.map.borrow().get(&id) {
			Self {
				inner: weak.upgrade().unwrap(),
			}
		} else {
			let scope = self.inner.scope.new_child();

			Self::new(&self.inner.arena, id, scope)
		}
	}
}

impl Drop for TagInner {
	fn drop(&mut self) {
		self.arena.map.borrow_mut().remove(&self.id);
	}
}

#[test]
fn basic_operations() {
	let oracle = Oracle::new();

	let tag_a = oracle.new_tag();
	let tag_b = oracle.new_tag();

	assert!(tag_a.should_meet(&tag_a));
	assert!(!tag_a.should_meet(&tag_b));

	let tag_ab0 = tag_a.combine(&tag_b, 0);
	let tag_ab1 = tag_a.combine(&tag_b, 1);

	assert!(tag_a.should_meet(&tag_ab0));
	assert!(tag_a.should_meet(&tag_ab1));
	assert!(!tag_b.should_meet(&tag_ab0));
	assert!(!tag_b.should_meet(&tag_ab1));
	assert!(!tag_ab0.should_meet(&tag_ab1));
}
