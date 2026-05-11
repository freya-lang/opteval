use crate::vm::inet::base::{Data, LambdaKind, Node, Port, PortKind};
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
		(&Data::Application { live: true }, &Data::Lambda { kind: LambdaKind::Live }) => application_lambda(),
		(&Data::Lambda { kind: LambdaKind::Live }, &Data::Application { live: true }) => mirror(application_lambda()),

		(&Data::Replicator { ref tag, count }, &Data::Lambda { kind: LambdaKind::Live }) => {
			replicator_lambda(tag, count)
		},
		(&Data::Lambda { kind: LambdaKind::Live }, &Data::Replicator { ref tag, count }) => {
			mirror(replicator_lambda(tag, count))
		},

		(&Data::Replicator { ref tag, count }, &Data::Application { live }) => replicator_application(tag, count, live),
		(&Data::Application { live }, &Data::Replicator { ref tag, count }) => {
			mirror(replicator_application(tag, count, live))
		},

		(
			&Data::Replicator {
				tag: ref left_tag,
				count: left_count,
			},
			&Data::Replicator {
				tag: ref right_tag,
				count: right_count,
			},
		) => replicator_replicator(left_tag, left_count, right_tag, right_count),

		(&Data::Replicator { ref tag, count }, &Data::Reformat) => replicator_reformat(tag, count),
		(&Data::Reformat, &Data::Replicator { ref tag, count }) => mirror(replicator_reformat(tag, count)),

		(&Data::Replicator { count, .. }, &Data::Binding { index }) => replicator_binding(count, index),
		(&Data::Binding { index }, &Data::Replicator { count, .. }) => mirror(replicator_binding(count, index)),

		(&Data::Reformat, &Data::Lambda { kind: LambdaKind::Live }) => reformat_lambda(),
		(&Data::Lambda { kind: LambdaKind::Live }, &Data::Reformat) => mirror(reformat_lambda()),

		(&Data::Reformat, &Data::Application { live: true }) => reformat_application(),
		(&Data::Application { live: true }, &Data::Reformat) => mirror(reformat_application()),

		(&Data::Reformat, &Data::Reformat) => reformat_reformat(),

		(
			&Data::Unlink { level },
			&Data::Lambda {
				kind: LambdaKind::Live { .. },
			},
		) => unlink_lambda(level),
		(
			&Data::Lambda {
				kind: LambdaKind::Live { .. },
			},
			&Data::Unlink { level },
		) => mirror(unlink_lambda(level)),

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

fn replicator_lambda(tag: &Tag, count: usize) -> (Vec<Port>, Vec<Port>) {
	let mut left_anchors = Vec::new();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	let replicator_in = Node::new(Data::Replicator {
		tag: tag.clone(),
		count,
	});
	let replicator_out = Node::new(Data::Replicator {
		tag: tag.clone(),
		count,
	});

	for i in 0 .. count {
		let left_anchor = anchor();

		let lambda = Node::new(Data::Lambda { kind: LambdaKind::Live });

		Port::link(&left_anchor, &lambda.main());
		Port::link(&lambda.aux(0), &replicator_in.aux(i));
		Port::link(&lambda.aux(1), &replicator_out.aux(i));

		left_anchors.push(left_anchor);
	}

	Port::link(&replicator_in.main(), &right_anchor_in);
	Port::link(&replicator_out.main(), &right_anchor_out);

	(left_anchors, vec![right_anchor_in, right_anchor_out])
}

fn replicator_application(tag: &Tag, count: usize, live: bool) -> (Vec<Port>, Vec<Port>) {
	let mut left_anchors = Vec::new();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	let replicator_in = Node::new(Data::Replicator {
		tag: tag.clone(),
		count,
	});
	let replicator_out = Node::new(Data::Replicator {
		tag: tag.clone(),
		count,
	});

	for i in 0 .. count {
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
	left_tag: &Tag,
	left_count: usize,
	right_tag: &Tag,
	right_count: usize,
) -> (Vec<Port>, Vec<Port>) {
	let mut left_anchors = Vec::new();
	let mut right_anchors = Vec::new();

	for _ in 0 .. left_count {
		left_anchors.push(anchor());
	}
	for _ in 0 .. right_count {
		right_anchors.push(anchor());
	}

	if left_tag == right_tag {
		debug_assert!(left_count == right_count);

		for i in 0 .. left_count {
			Port::link(&left_anchors[i], &right_anchors[i]);
		}

		return (left_anchors, right_anchors);
	}

	let mut left_aux: Vec<Vec<_>> = Vec::new();
	let mut right_aux: Vec<Vec<_>> = Vec::new();

	for i in 0 .. left_count {
		let replicator = Node::new(Data::Replicator {
			tag: right_tag.combine(left_tag, i),
			count: right_count,
		});
		left_aux.push(replicator.iter_aux().collect());

		Port::link(&left_anchors[i], &replicator.main());
	}

	for i in 0 .. right_count {
		let replicator = Node::new(Data::Replicator {
			tag: left_tag.clone(),
			count: left_count,
		});
		right_aux.push(replicator.iter_aux().collect());

		Port::link(&right_anchors[i], &replicator.main());
	}

	for l in 0 .. left_count {
		for r in 0 .. right_count {
			Port::link(&left_aux[l][r], &right_aux[r][l]);
		}
	}

	(left_anchors, right_anchors)
}

fn replicator_reformat(tag: &Tag, count: usize) -> (Vec<Port>, Vec<Port>) {
	let mut left_anchors = Vec::new();
	let right_anchor = anchor();

	let replicator = Node::new(Data::Replicator {
		tag: tag.clone(),
		count,
	});

	for i in 0 .. count {
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

	let lambda = Node::new(Data::Lambda { kind: LambdaKind::Live });
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

	let lambda = Node::new(Data::Lambda {
		kind: LambdaKind::NotLive,
	});
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

fn unlink_ascend(level: usize) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor = anchor();

	let unlink = Node::new(Data::Unlink { level });

	Port::link(&left_anchor, &unlink.aux(0));
	Port::link(&unlink.main(), &right_anchor);

	(vec![left_anchor], vec![right_anchor])
}

fn unlink_binding(level: usize, index: usize) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();

	let binding = Node::new(Data::Binding { index: level - index });

	Port::link(&left_anchor, &binding.main());

	(vec![left_anchor], Vec::new())
}
