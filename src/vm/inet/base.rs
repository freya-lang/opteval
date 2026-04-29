use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

use crate::vm::inet::util::anchor;

pub(crate) enum Data {
	Lambda { live: bool },
	Application { live: bool },
	Replicator { level: usize, count: usize },
	Ascend { level: usize },
	Descend { level: usize },
	Reformat,
	Unlink { level: usize },
	Binding { index: usize },
	Anchor,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
	Main,
	Aux(usize),
}

#[derive(Clone)]
struct Cell(RefCell<Option<Port>>);

#[derive(Clone)]
pub(crate) struct Port {
	node: Node,
	kind: Kind,
}

#[derive(Clone)]
pub(crate) struct Node(Rc<Backing>);

struct Backing {
	data: Data,
	main: Cell,
	aux: Vec<Cell>,
}

impl Data {
	fn num_aux(&self) -> usize {
		match self {
			Data::Lambda { live: true } => 2,
			Data::Lambda { live: false } => 1,
			Data::Application { .. } => 2,
			Data::Replicator { count, .. } => *count,
			Data::Ascend { .. } => 1,
			Data::Descend { .. } => 1,
			Data::Reformat { .. } => 1,
			Data::Unlink { .. } => 1,
			Data::Binding { .. } => 0,
			Data::Anchor => 0,
		}
	}
}

impl Cell {
	fn empty() -> Self {
		Self(RefCell::new(None))
	}

	fn get(&self) -> Option<Port> {
		self.0.borrow().clone()
	}

	fn modify(&self) -> impl DerefMut<Target = Option<Port>> {
		self.0.borrow_mut()
	}
}

impl Port {
	fn cell(&self) -> &Cell {
		match self.kind {
			Kind::Main => &self.node.0.main,
			Kind::Aux(i) => &self.node.0.aux[i],
		}
	}

	pub(crate) fn kind(&self) -> &Kind {
		&self.kind
	}

	pub(crate) fn node(&self) -> &Node {
		&self.node
	}

	pub(crate) fn link(&self, other: &Self) {
		let mut self_cell = self.cell().modify();
		let mut other_cell = other.cell().modify();

		debug_assert!(self_cell.is_none());
		debug_assert!(other_cell.is_none());

		*self_cell = Some(other.clone());
		*other_cell = Some(self.clone());
	}

	pub(crate) fn try_unlink(&self) -> Option<Self> {
		let other = match self.cell().modify().take() {
			Some(other) => other,
			None => return None,
		};

		self.unlink_reflection(&other);

		Some(other)
	}

	pub(crate) fn unlink(&self) -> Self {
		let other = self.cell().modify().take().unwrap();
		self.unlink_reflection(&other);

		other
	}

	fn unlink_reflection(&self, other: &Self) {
		let reflected = other.cell().modify().take().unwrap();
		debug_assert!(*self == reflected);
	}

	pub(crate) fn linked(&self) -> Option<Self> {
		self.cell().get()
	}

	pub(crate) fn retract(&self) -> Self {
		let linked = self.unlink();
		let anchor = anchor();

		Port::link(&linked, &anchor);

		anchor
	}

	pub(crate) fn swap(&self, swap_for: &Self) {
		let linked = self.unlink();
		Self::link(&swap_for, &linked);
	}
}

impl Node {
	pub(crate) fn main(&self) -> Port {
		Port {
			node: self.clone(),
			kind: Kind::Main,
		}
	}

	pub(crate) fn aux(&self, i: usize) -> Port {
		debug_assert!(i < self.0.aux.len());

		Port {
			node: self.clone(),
			kind: Kind::Aux(i),
		}
	}

	pub(crate) fn iter_aux(&self) -> impl Iterator<Item = Port> + use<> {
		let this = self.clone();

		(0 .. this.0.aux.len()).map(move |i| Port {
			node: this.clone(),
			kind: Kind::Aux(i),
		})
	}

	pub(crate) fn new(data: Data) -> Self {
		Self(Rc::new(Backing {
			main: Cell::empty(),
			aux: vec![Cell::empty(); data.num_aux()],
			data,
		}))
	}

	pub(crate) fn data(&self) -> &Data {
		&self.0.data
	}
}

impl PartialEq for Port {
	fn eq(&self, other: &Self) -> bool {
		Rc::ptr_eq(&self.node.0, &other.node.0) && self.kind == other.kind
	}
}

impl Eq for Port {}
