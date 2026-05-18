use crate::vm::inet::base::{Data, Node, Port, PortKind};
use crate::vm::inet::util::{anchor, join_slice};
use crate::vm::oracle::Tag;

fn mirror<A, B>((a, b): (A, B)) -> (B, A) {
	(b, a)
}

pub(crate) fn interact(left: &Port, right: &Port) {
	debug_assert!(*left.kind() == PortKind::Main);
	debug_assert!(*right.kind() == PortKind::Main);
	debug_assert!(left.linked().as_ref() == Some(right));
	debug_assert!(right.linked().as_ref() == Some(left));

	left.unlink();

	let left_aux: Vec<_> = left.node().iter_aux().map(|port| port.retract()).collect();
	let right_aux: Vec<_> = right.node().iter_aux().map(|port| port.retract()).collect();

	let (left_new, right_new) = match (left.node().data(), right.node().data()) {
		(&Data::Application { live: true }, &Data::Lambda { live: true }) => application_lambda(),
		(&Data::Lambda { live: true }, &Data::Application { live: true }) => mirror(application_lambda()),

		(
			&Data::Replicator {
				id_tag,
				ref output_tags,
			},
			&Data::Lambda { live: true },
		) => replicator_lambda(id_tag, output_tags),
		(
			&Data::Lambda { live: true },
			&Data::Replicator {
				id_tag,
				ref output_tags,
			},
		) => mirror(replicator_lambda(id_tag, output_tags)),

		(
			&Data::Replicator {
				id_tag,
				ref output_tags,
			},
			&Data::Application { live },
		) => replicator_application(id_tag, output_tags, live),
		(
			&Data::Application { live },
			&Data::Replicator {
				id_tag,
				ref output_tags,
			},
		) => mirror(replicator_application(id_tag, output_tags, live)),

		(
			&Data::Replicator {
				id_tag: left_id_tag,
				output_tags: ref left_output_tags,
			},
			&Data::Replicator {
				id_tag: right_id_tag,
				output_tags: ref right_output_tags,
			},
		) => replicator_replicator(left_id_tag, left_output_tags, right_id_tag, right_output_tags),

		(
			&Data::Replicator {
				id_tag,
				ref output_tags,
			},
			&Data::Reformat,
		) => replicator_reformat(id_tag, output_tags),
		(
			&Data::Reformat,
			&Data::Replicator {
				id_tag,
				ref output_tags,
			},
		) => mirror(replicator_reformat(id_tag, output_tags)),

		(&Data::Replicator { ref output_tags, .. }, &Data::Binding { index }) => {
			replicator_binding(output_tags.len(), index)
		},
		(&Data::Binding { index }, &Data::Replicator { ref output_tags, .. }) => {
			mirror(replicator_binding(output_tags.len(), index))
		},

		(&Data::Reformat, &Data::Lambda { live: true }) => reformat_lambda(),
		(&Data::Lambda { live: true }, &Data::Reformat) => mirror(reformat_lambda()),

		(&Data::Reformat, &Data::Application { live: true }) => reformat_application(),
		(&Data::Application { live: true }, &Data::Reformat) => mirror(reformat_application()),

		(&Data::Reformat, &Data::Reformat) => reformat_reformat(),

		(&Data::Unlink { level }, &Data::Lambda { live: true }) => unlink_lambda(level),
		(&Data::Lambda { live: true }, &Data::Unlink { level }) => mirror(unlink_lambda(level)),

		(&Data::Unlink { level }, &Data::Application { live: false }) => unlink_application(level),
		(&Data::Application { live: false }, &Data::Unlink { level }) => mirror(unlink_application(level)),

		(&Data::Unlink { level }, &Data::Binding { index }) => unlink_binding(level, index),
		(&Data::Binding { index }, &Data::Unlink { level }) => mirror(unlink_binding(level, index)),

		_ => unreachable!("invalid active pair"),
	};

	join_slice(&left_aux, &left_new);
	join_slice(&right_aux, &right_new);
}

fn application_lambda() -> (Vec<Port>, Vec<Port>) {
	let left_anchor_in = anchor();
	let left_anchor_out = anchor();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	Port::link(&left_anchor_in, &right_anchor_in);
	Port::link(&left_anchor_out, &right_anchor_out);

	(vec![left_anchor_in, left_anchor_out], vec![
		right_anchor_in,
		right_anchor_out,
	])
}

fn replicator_lambda(id_tag: Tag, output_tags: &[Tag]) -> (Vec<Port>, Vec<Port>) {
	let mut left_anchors = Vec::new();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	let replicator_in = Node::new(Data::Replicator {
		id_tag,
		output_tags: output_tags.to_owned(),
	});
	let replicator_out = Node::new(Data::Replicator {
		id_tag,
		output_tags: output_tags.to_owned(),
	});

	for i in 0 .. output_tags.len() {
		let left_anchor = anchor();

		let lambda = Node::new(Data::Lambda { live: true });

		Port::link(&left_anchor, &lambda.main());
		Port::link(&lambda.aux(0), &replicator_in.aux(i));
		Port::link(&lambda.aux(1), &replicator_out.aux(i));

		left_anchors.push(left_anchor);
	}

	Port::link(&replicator_in.main(), &right_anchor_in);
	Port::link(&replicator_out.main(), &right_anchor_out);

	(left_anchors, vec![right_anchor_in, right_anchor_out])
}

fn replicator_application(id_tag: Tag, output_tags: &[Tag], live: bool) -> (Vec<Port>, Vec<Port>) {
	let mut left_anchors = Vec::new();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	let replicator_in = Node::new(Data::Replicator {
		id_tag,
		output_tags: output_tags.to_owned(),
	});
	let replicator_out = Node::new(Data::Replicator {
		id_tag,
		output_tags: output_tags.to_owned(),
	});

	for i in 0 .. output_tags.len() {
		let left_anchor = anchor();

		let application = Node::new(Data::Application { live });

		Port::link(&left_anchor, &application.main());
		Port::link(&application.aux(0), &replicator_in.aux(i));
		Port::link(&application.aux(1), &replicator_out.aux(i));

		left_anchors.push(left_anchor);
	}

	Port::link(&replicator_in.main(), &right_anchor_in);
	Port::link(&replicator_out.main(), &right_anchor_out);

	(left_anchors, vec![right_anchor_in, right_anchor_out])
}

fn replicator_replicator(
	left_id_tag: Tag,
	left_output_tags: &[Tag],
	right_id_tag: Tag,
	right_output_tags: &[Tag],
) -> (Vec<Port>, Vec<Port>) {
	let left_count = left_output_tags.len();
	let right_count = right_output_tags.len();

	let mut left_anchors = Vec::new();
	let mut right_anchors = Vec::new();

	for _ in 0 .. left_count {
		left_anchors.push(anchor());
	}
	for _ in 0 .. right_count {
		right_anchors.push(anchor());
	}

	if left_id_tag == right_id_tag {
		debug_assert!(left_output_tags == right_output_tags);

		for i in 0 .. left_count {
			Port::link(&left_anchors[i], &right_anchors[i]);
		}

		return (left_anchors, right_anchors);
	}

	let mut left_aux: Vec<Vec<_>> = Vec::new();
	let mut right_aux: Vec<Vec<_>> = Vec::new();

	for i in 0 .. left_count {
		let left_tag = left_output_tags[i];

		let replicator = Node::new(Data::Replicator {
			id_tag: left_tag.operate(right_id_tag),
			output_tags: right_output_tags.iter().map(|&tag| left_tag.operate(tag)).collect(),
		});
		left_aux.push(replicator.iter_aux().collect());

		Port::link(&left_anchors[i], &replicator.main());
	}

	for i in 0 .. right_count {
		let replicator = Node::new(Data::Replicator {
			id_tag: left_id_tag,
			output_tags: left_output_tags.to_owned(),
		});
		right_aux.push(replicator.iter_aux().collect());

		Port::link(&right_anchors[i], &replicator.main());
	}

	for l in 0 .. left_output_tags.len() {
		for r in 0 .. right_output_tags.len() {
			Port::link(&left_aux[l][r], &right_aux[r][l]);
		}
	}

	(left_anchors, right_anchors)
}

fn replicator_reformat(id_tag: Tag, output_tags: &[Tag]) -> (Vec<Port>, Vec<Port>) {
	let mut left_anchors = Vec::new();
	let right_anchor = anchor();

	let replicator = Node::new(Data::Replicator {
		id_tag,
		output_tags: output_tags.to_owned(),
	});

	for i in 0 .. output_tags.len() {
		let left_anchor = anchor();

		let reformat = Node::new(Data::Reformat);

		Port::link(&left_anchor, &reformat.main());
		Port::link(&reformat.aux(0), &replicator.aux(i));

		left_anchors.push(left_anchor);
	}

	Port::link(&replicator.main(), &right_anchor);

	(left_anchors, vec![right_anchor])
}

fn replicator_binding(count: usize, index: usize) -> (Vec<Port>, Vec<Port>) {
	let mut left_anchors = Vec::new();

	for _ in 0 .. count {
		let left_anchor = anchor();

		let binding = Node::new(Data::Binding { index });

		Port::link(&left_anchor, &binding.main());

		left_anchors.push(left_anchor);
	}

	(left_anchors, Vec::new())
}

fn reformat_lambda() -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	let lambda = Node::new(Data::Lambda { live: true });
	let reformat_in = Node::new(Data::Reformat);
	let reformat_out = Node::new(Data::Reformat);

	Port::link(&left_anchor, &lambda.main());

	Port::link(&lambda.aux(0), &reformat_in.aux(0));
	Port::link(&reformat_in.main(), &right_anchor_in);

	Port::link(&lambda.aux(1), &reformat_out.aux(0));
	Port::link(&reformat_out.main(), &right_anchor_out);

	(vec![left_anchor], vec![right_anchor_in, right_anchor_out])
}

fn reformat_application() -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	let application = Node::new(Data::Application { live: false });
	let reformat_in = Node::new(Data::Reformat);
	let reformat_out = Node::new(Data::Reformat);

	Port::link(&left_anchor, &application.aux(0));

	Port::link(&application.aux(1), &reformat_in.aux(0));
	Port::link(&reformat_in.main(), &right_anchor_in);

	Port::link(&application.main(), &reformat_out.aux(0));
	Port::link(&reformat_out.main(), &right_anchor_out);

	(vec![left_anchor], vec![right_anchor_in, right_anchor_out])
}

fn reformat_reformat() -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor = anchor();

	Port::link(&left_anchor, &right_anchor);

	(vec![left_anchor], vec![right_anchor])
}

fn unlink_lambda(mut level: usize) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	level += 1;

	let lambda = Node::new(Data::Lambda { live: false });
	let unlink = Node::new(Data::Unlink { level });
	let binding = Node::new(Data::Binding { index: level });

	Port::link(&left_anchor, &lambda.main());

	Port::link(&lambda.aux(0), &unlink.aux(0));
	Port::link(&unlink.main(), &right_anchor_out);

	Port::link(&binding.main(), &right_anchor_in);

	(vec![left_anchor], vec![right_anchor_in, right_anchor_out])
}

fn unlink_application(level: usize) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	let application = Node::new(Data::Application { live: false });
	let unlink_in = Node::new(Data::Unlink { level });
	let unlink_out = Node::new(Data::Unlink { level });

	Port::link(&left_anchor, &application.main());

	Port::link(&application.aux(0), &unlink_in.aux(0));
	Port::link(&unlink_in.main(), &right_anchor_in);

	Port::link(&application.aux(1), &unlink_out.aux(0));
	Port::link(&unlink_out.main(), &right_anchor_out);

	(vec![left_anchor], vec![right_anchor_in, right_anchor_out])
}

fn unlink_binding(level: usize, index: usize) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();

	let binding = Node::new(Data::Binding { index: level - index });

	Port::link(&left_anchor, &binding.main());

	(vec![left_anchor], Vec::new())
}
