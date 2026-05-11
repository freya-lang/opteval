use crate::vm::oracle::hash::hash;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) struct Tag {
	inner: [u8; 32],
}

fn combine_tags(a: [u8; 32], b: [u8; 32], index: u64) -> [u8; 32] {
	let mut buffer = Vec::new();

	buffer.extend(a);
	buffer.extend(b);
	buffer.extend(index.to_le_bytes());

	hash(buffer)
}

impl Tag {
	pub(crate) fn new(counter: u64) -> Self {
		Tag {
			inner: hash(counter.to_le_bytes()),
		}
	}

	pub(crate) fn combine(&self, other: &Self, index: usize) -> Self {
		let mut buffer = Vec::new();

		buffer.extend(self.inner);
		buffer.extend(other.inner);
		buffer.extend(u64::try_from(index).expect("overflowed u64 in tag index").to_le_bytes());

		Tag { inner: hash(buffer) }
	}
}
