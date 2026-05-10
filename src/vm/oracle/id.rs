use crate::vm::oracle::hash::hash;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) struct Id([u8; 32]);

impl Id {
	pub(crate) fn combine(self, other: Id, index: u64) -> Id {
		let mut buffer = Vec::new();

		buffer.extend(self.0);
		buffer.extend(other.0);
		buffer.extend(index.to_le_bytes());

		Id(hash(buffer))
	}
}
