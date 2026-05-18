mod field;
mod group;
mod hash;

use std::sync::atomic::{AtomicU64, Ordering};

use crate::vm::oracle::field::Element as FieldElement;
use crate::vm::oracle::group::Element as GroupElement;
use crate::vm::oracle::hash::hash;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Tag {
	element: GroupElement,
}

impl Tag {
	pub(crate) fn new() -> Self {
		loop {
			if let Some(out) = try_gen_tag() {
				return out;
			}
		}
	}

	pub(crate) fn operate(self, other: Self) -> Self {
		Self {
			element: self.element * other.element * self.element.inv(),
		}
	}
}

fn try_gen_tag() -> Option<Tag> {
	static COUNTER: AtomicU64 = AtomicU64::new(0);

	let id = COUNTER.fetch_add(1, Ordering::Relaxed);

	if id == u64::MAX {
		panic!("tag generation counter overflowed");
	}

	let hashed = hash(id.to_le_bytes());
	let a = u64::from_le_bytes(hashed[0 .. 8].try_into().unwrap());
	let b = u64::from_le_bytes(hashed[8 .. 16].try_into().unwrap());
	let c = u64::from_le_bytes(hashed[16 .. 24].try_into().unwrap());
	let d = u64::from_le_bytes(hashed[24 .. 32].try_into().unwrap());

	let a = FieldElement::new_from_randomness(a)?;
	let b = FieldElement::new_from_randomness(b)?;
	let c = FieldElement::new_from_randomness(c)?;
	let d = FieldElement::new_from_randomness(d)?;

	Some(Tag {
		element: GroupElement::new(a, b, c, d),
	})
}

#[test]
fn uniqueness() {
	use std::collections::HashSet;

	let mut hash_set = HashSet::new();

	for _ in 0 .. 1000 {
		assert!(hash_set.insert(Tag::new()));
	}
}

#[test]
fn tag_identity() {
	let a = Tag::new();
	let b = Tag::new();
	let c = Tag::new();

	assert!(a.operate(b).operate(a.operate(c)) == a.operate(b.operate(c)));
}
