mod encode;

use std::collections::HashSet;
use std::iter::once;
use std::mem::replace;
use std::rc::Rc;

pub(crate) use crate::vm::inet::arena::encode::encode;
use crate::vm::inet::base::{Data, LambdaKind, Node, Port, PortKind};
use crate::vm::inet::interaction::interact;
use crate::vm::inet::util::anchor;
use crate::vm::term::Term;

#[derive(Clone)]
pub(crate) struct Output {
	arena: Rc<Arena>,
	port: Port,
}

pub(crate) struct Arena {
	deletion_anchors: Vec<Node>,
}

fn recursive_deletion(node: Node) {
	let mut nodes = vec![node];

	while let Some(node) = nodes.pop() {
		for port in once(node.main()).chain(node.iter_aux()) {
			let Some(port) = port.try_unlink() else {
				continue;
			};

			nodes.push(port.node().clone());
		}
	}
}

fn is_anchor_node(node: &Node) -> bool {
	matches!(node.data(), Data::Anchor { .. })
}

impl Output {
	pub(crate) fn pull(self) -> Term<Output> {
		let mut stack = vec![self.port.clone()];
		let mut tags = HashSet::new();

		loop {
			let mut current = stack.pop().unwrap();

			if let Data::Replicator { tag, .. } = current.node().data() {
				tags.remove(tag);
			}

			let linked = loop {
				let linked = current.linked().unwrap();

				if matches!(linked.kind(), PortKind::Main) {
					break linked;
				}

				if let Data::Replicator { tag, .. } = linked.node().data() {
					tags.insert(*tag);
				}

				stack.push(replace(&mut current, linked.node().main()));
			};

			if is_anchor_node(current.node()) {
				debug_assert!(stack.len() == 0);
				debug_assert!(matches!(current.node().data(), Data::Anchor));

				current.unlink();

				return match linked.node().data() {
					Data::Lambda {
						kind: LambdaKind::NotLive,
					} => {
						let new_output = Self {
							arena: self.arena.clone(),
							port: anchor(),
						};

						linked.node().aux(0).swap(&new_output.port);

						Term::Lambda { body: new_output }
					},
					Data::Application { live: false } => {
						let new_output_left = Self {
							arena: self.arena.clone(),
							port: anchor(),
						};
						let new_output_right = Self {
							arena: self.arena.clone(),
							port: anchor(),
						};

						linked.node().aux(0).swap(&new_output_left.port);
						linked.node().aux(1).swap(&new_output_right.port);

						Term::Application {
							left: new_output_left,
							right: new_output_right,
						}
					},
					Data::Binding { index: level } => Term::Binding { index: *level },
					_ => unreachable!(),
				};
			} else {
				match linked.node().data() {
					Data::Replicator { tag, .. } if tags.contains(tag) => interact(&linked, &current),
					_ => interact(&current, &linked),
				}
			}
		}
	}
}

impl Drop for Arena {
	fn drop(&mut self) {
		for anchor in self.deletion_anchors.drain(..) {
			recursive_deletion(anchor);
		}
	}
}
