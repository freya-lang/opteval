use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::vm::inet::arena::{Arena, Output};
use crate::vm::inet::base::{Data, LambdaKind, Node, Port};
use crate::vm::inet::util::anchor;
use crate::vm::oracle::Tag;
use crate::vm::term::{Strict, Term};

pub(crate) fn encode(input: &Strict) -> Output {
	let mut counter = 0;
	let encoded = encode_inner(&mut counter, &input);

	debug_assert!(encoded.bindings.len() == 0);

	let arena = Rc::new(Arena {
		deletion_anchors: encoded.deletion_anchors,
	});

	let port = anchor();

	let reformat = Node::new(Data::Reformat);
	let unlink = Node::new(Data::Unlink { level: 0 });

	encoded.main_output.swap(&reformat.main());

	Port::link(&reformat.aux(0), &unlink.main());
	Port::link(&unlink.aux(0), &port);

	Output { arena, port }
}

struct EncodingData {
	main_output: Port,
	bindings: HashMap<usize, Port>,
	deletion_anchors: Vec<Node>,
}

fn encode_inner(counter: &mut u64, input: &Strict) -> EncodingData {
	match input.get() {
		Term::Lambda { body } => {
			let mut body = encode_inner(counter, body);

			let main_output = anchor();

			let lambda = Node::new(Data::Lambda { kind: LambdaKind::Live });

			Port::link(&main_output, &lambda.main());

			body.main_output.swap(&lambda.aux(1));

			if let Some(binding_port) = body.bindings.remove(&0) {
				binding_port.swap(&lambda.aux(0));
			} else {
				let deletion_anchor = anchor();

				Port::link(&lambda.aux(0), &deletion_anchor);

				body.deletion_anchors.push(deletion_anchor.node().clone());
			}

			let mut bindings = HashMap::new();

			for (index, binding) in body.bindings {
				bindings.insert(index - 1, binding);
			}

			EncodingData {
				main_output,
				bindings,
				deletion_anchors: body.deletion_anchors,
			}
		},
		Term::Application { left, right } => {
			let mut left = encode_inner(counter, left);
			let mut right = encode_inner(counter, right);

			let main_output = anchor();

			let application = Node::new(Data::Application { live: true });

			Port::link(&main_output, &application.aux(1));

			left.main_output.swap(&application.main());
			right.main_output.swap(&application.aux(0));

			let mut all_indexes = HashSet::new();

			for &index in left.bindings.keys() {
				all_indexes.insert(index);
			}
			for &index in right.bindings.keys() {
				all_indexes.insert(index);
			}

			let mut bindings = HashMap::new();

			for index in all_indexes {
				let left = left.bindings.remove(&index);
				let right = right.bindings.remove(&index);

				match (left, right) {
					(Some(left), Some(right)) => {
						let binding_port = anchor();

						let replicator = Node::new(Data::Replicator {
							tag: Tag::new(*counter),
							count: 2,
						});
						*counter = counter.checked_add(1).unwrap();

						Port::link(&binding_port, &replicator.main());

						left.swap(&replicator.aux(0));
						right.swap(&replicator.aux(1));

						bindings.insert(index, binding_port);
					},
					(Some(single), None) | (None, Some(single)) => {
						bindings.insert(index, single);
					},
					(None, None) => unreachable!(),
				}
			}

			debug_assert!(left.bindings.len() == 0);
			debug_assert!(right.bindings.len() == 0);

			let mut deletion_anchors = Vec::new();
			deletion_anchors.extend(left.deletion_anchors);
			deletion_anchors.extend(right.deletion_anchors);

			EncodingData {
				main_output,
				bindings,
				deletion_anchors,
			}
		},
		Term::Binding { index } => {
			let anchor_in = anchor();
			let anchor_out = anchor();

			Port::link(&anchor_in, &anchor_out);

			let mut bindings = HashMap::new();
			bindings.insert(index, anchor_in);

			EncodingData {
				main_output: anchor_out,
				bindings,
				deletion_anchors: Vec::new(),
			}
		},
	}
}
