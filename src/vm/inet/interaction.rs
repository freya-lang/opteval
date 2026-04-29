use std::cmp::Ordering;
use std::mem::swap;

use crate::vm::inet::base::{Data, Kind, Node, Port};
use crate::vm::inet::util::{anchor, join_slice};

fn mirror<A, B>((a, b): (A, B)) -> (B, A) {
	(b, a)
}

pub(crate) fn interact(left: &Port, right: &Port) {
	debug_assert!(*left.kind() == Kind::Main);
	debug_assert!(*right.kind() == Kind::Main);
	debug_assert!(left.linked().as_ref() == Some(right));
	debug_assert!(right.linked().as_ref() == Some(left));

	left.unlink();

	let left_aux: Vec<_> = left.node().iter_aux().map(|port| port.retract()).collect();
	let right_aux: Vec<_> = right.node().iter_aux().map(|port| port.retract()).collect();

	let (left_new, right_new) = match (left.node().data(), right.node().data()) {
		(&Data::Application { live: true }, &Data::Lambda { live: true }) => application_lambda(),
		(&Data::Lambda { live: true }, &Data::Application { live: true }) => mirror(application_lambda()),

		(&Data::Replicator { level, count }, &Data::Lambda { live: true }) => replicator_lambda(level, count),
		(&Data::Lambda { live: true }, &Data::Replicator { level, count }) => mirror(replicator_lambda(level, count)),

		(&Data::Replicator { level, count }, &Data::Application { live }) => replicator_application(level, count, live),
		(&Data::Application { live }, &Data::Replicator { level, count }) => {
			mirror(replicator_application(level, count, live))
		},

		(
			&Data::Replicator {
				level: left_level,
				count: left_count,
			},
			&Data::Replicator {
				level: right_level,
				count: right_count,
			},
		) => replicator_replicator(left_level, left_count, right_level, right_count),

		(&Data::Replicator { level, count }, &Data::Reformat) => replicator_reformat(level, count),
		(&Data::Reformat, &Data::Replicator { level, count }) => mirror(replicator_reformat(level, count)),

		(&Data::Replicator { count, .. }, &Data::Binding { index }) => replicator_binding(count, index),
		(&Data::Binding { index }, &Data::Replicator { count, .. }) => mirror(replicator_binding(count, index)),

		(&Data::Ascend { level }, &Data::Lambda { live: true }) => ascend_lambda(level),
		(&Data::Lambda { live: true }, &Data::Ascend { level }) => mirror(ascend_lambda(level)),

		(&Data::Ascend { level }, &Data::Application { live }) => ascend_application(level, live),
		(&Data::Application { live }, &Data::Ascend { level }) => mirror(ascend_application(level, live)),

		(
			&Data::Ascend { level: ascend_level },
			&Data::Replicator {
				level: replicator_level,
				count: replicator_count,
			},
		) => ascend_replicator(ascend_level, replicator_level, replicator_count),
		(
			&Data::Replicator {
				level: replicator_level,
				count: replicator_count,
			},
			&Data::Ascend { level: ascend_level },
		) => mirror(ascend_replicator(ascend_level, replicator_level, replicator_count)),

		(&Data::Ascend { level: left_level }, &Data::Ascend { level: right_level }) => {
			ascend_ascend(left_level, right_level)
		},

		(&Data::Ascend { .. }, &Data::Binding { index }) => binding_unaffected(index),
		(&Data::Binding { index }, &Data::Ascend { .. }) => mirror(binding_unaffected(index)),

		(&Data::Ascend { level: ascend_level }, &Data::Descend { level: descend_level }) => {
			ascend_descend(ascend_level, descend_level)
		},
		(&Data::Descend { level: descend_level }, &Data::Ascend { level: ascend_level }) => {
			mirror(ascend_descend(ascend_level, descend_level))
		},

		(&Data::Descend { level }, &Data::Lambda { live: true }) => descend_lambda(level),
		(&Data::Lambda { live: true }, &Data::Descend { level }) => mirror(descend_lambda(level)),

		(&Data::Descend { level }, &Data::Application { live }) => descend_application(level, live),
		(&Data::Application { live }, &Data::Descend { level }) => mirror(descend_application(level, live)),

		(
			&Data::Descend { level: ascend_level },
			&Data::Replicator {
				level: replicator_level,
				count: replicator_count,
			},
		) => descend_replicator(ascend_level, replicator_level, replicator_count),
		(
			&Data::Replicator {
				level: replicator_level,
				count: replicator_count,
			},
			&Data::Descend { level: ascend_level },
		) => mirror(descend_replicator(ascend_level, replicator_level, replicator_count)),

		(&Data::Descend { level: left_level }, &Data::Descend { level: right_level }) => {
			descend_descend(left_level, right_level)
		},

		(&Data::Descend { .. }, &Data::Binding { index }) => binding_unaffected(index),
		(&Data::Binding { index }, &Data::Descend { .. }) => mirror(binding_unaffected(index)),

		(&Data::Reformat, &Data::Lambda { live: true }) => reformat_lambda(),
		(&Data::Lambda { live: true }, &Data::Reformat) => mirror(reformat_lambda()),

		(&Data::Reformat, &Data::Application { live: true }) => reformat_application(),
		(&Data::Application { live: true }, &Data::Reformat) => mirror(reformat_application()),

		(&Data::Reformat, &Data::Ascend { level }) => reformat_ascend(level),
		(&Data::Ascend { level }, &Data::Reformat) => mirror(reformat_ascend(level)),

		(&Data::Reformat, &Data::Descend { level }) => reformat_descend(level),
		(&Data::Descend { level }, &Data::Reformat) => mirror(reformat_descend(level)),

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

	let ascend = Node::new(Data::Ascend { level: 0 });
	let descend = Node::new(Data::Descend { level: 0 });

	Port::link(&left_anchor_in, &ascend.main());
	Port::link(&ascend.aux(0), &right_anchor_in);

	Port::link(&left_anchor_out, &descend.aux(0));
	Port::link(&descend.main(), &right_anchor_out);

	(vec![left_anchor_in, left_anchor_out], vec![
		right_anchor_in,
		right_anchor_out,
	])
}

fn replicator_lambda(level: usize, count: usize) -> (Vec<Port>, Vec<Port>) {
	let mut left_anchors = Vec::new();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	let replicator_in = Node::new(Data::Replicator {
		level: level + 1,
		count,
	});
	let replicator_out = Node::new(Data::Replicator {
		level: level + 1,
		count,
	});

	for i in 0 .. count {
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

fn replicator_application(level: usize, count: usize, live: bool) -> (Vec<Port>, Vec<Port>) {
	let mut left_anchors = Vec::new();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	let replicator_in = Node::new(Data::Replicator { level, count });
	let replicator_out = Node::new(Data::Replicator { level, count });

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
	left_level: usize,
	left_count: usize,
	right_level: usize,
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

	if left_level == right_level {
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
			level: right_level,
			count: right_count,
		});
		left_aux.push(replicator.iter_aux().collect());

		Port::link(&left_anchors[i], &replicator.main());
	}

	for i in 0 .. right_count {
		let replicator = Node::new(Data::Replicator {
			level: left_level,
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

fn replicator_reformat(level: usize, count: usize) -> (Vec<Port>, Vec<Port>) {
	let mut left_anchors = Vec::new();
	let right_anchor = anchor();

	let replicator = Node::new(Data::Replicator { level, count });

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

fn ascend_lambda(level: usize) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	let lambda = Node::new(Data::Lambda { live: true });
	let ascend_in = Node::new(Data::Ascend { level: level + 1 });
	let ascend_out = Node::new(Data::Ascend { level: level + 1 });

	Port::link(&left_anchor, &lambda.main());

	Port::link(&lambda.aux(0), &ascend_in.aux(0));
	Port::link(&ascend_in.main(), &right_anchor_in);

	Port::link(&lambda.aux(1), &ascend_out.aux(0));
	Port::link(&ascend_out.main(), &right_anchor_out);

	(vec![left_anchor], vec![right_anchor_in, right_anchor_out])
}

fn ascend_application(level: usize, live: bool) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	let application = Node::new(Data::Application { live });
	let ascend_in = Node::new(Data::Ascend { level });
	let ascend_out = Node::new(Data::Ascend { level });

	Port::link(&left_anchor, &application.main());

	Port::link(&application.aux(0), &ascend_in.aux(0));
	Port::link(&ascend_in.main(), &right_anchor_in);

	Port::link(&application.aux(1), &ascend_out.aux(0));
	Port::link(&ascend_out.main(), &right_anchor_out);

	(vec![left_anchor], vec![right_anchor_in, right_anchor_out])
}

fn ascend_replicator(
	ascend_level: usize,
	mut replicator_level: usize,
	replicator_count: usize,
) -> (Vec<Port>, Vec<Port>) {
	if replicator_level >= ascend_level {
		replicator_level += 1;
	}

	let left_anchor = anchor();
	let mut right_anchors = Vec::new();

	let replicator = Node::new(Data::Replicator {
		level: replicator_level,
		count: replicator_count,
	});

	Port::link(&left_anchor, &replicator.main());

	for i in 0 .. replicator_count {
		let right_anchor = anchor();

		let ascend = Node::new(Data::Ascend { level: ascend_level });

		Port::link(&replicator.aux(i), &ascend.aux(0));
		Port::link(&ascend.main(), &right_anchor);
		right_anchors.push(right_anchor);
	}

	(vec![left_anchor], right_anchors)
}

fn ascend_ascend(mut left_level: usize, mut right_level: usize) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor = anchor();

	match left_level.cmp(&right_level) {
		Ordering::Equal => {
			Port::link(&left_anchor, &right_anchor);

			return (vec![left_anchor], vec![right_anchor]);
		},
		Ordering::Less => right_level += 1,
		Ordering::Greater => left_level += 1,
	}

	swap(&mut left_level, &mut right_level);

	let left_ascend = Node::new(Data::Ascend { level: left_level });
	let right_ascend = Node::new(Data::Ascend { level: right_level });

	Port::link(&left_anchor, &left_ascend.main());
	Port::link(&left_ascend.aux(0), &right_ascend.aux(0));
	Port::link(&right_ascend.main(), &right_anchor);

	(vec![left_anchor], vec![right_anchor])
}

fn ascend_descend(mut ascend_level: usize, mut descend_level: usize) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor = anchor();

	match ascend_level.cmp(&descend_level) {
		Ordering::Equal => panic!(),
		Ordering::Less => descend_level += 1,
		Ordering::Greater => ascend_level -= 1,
	}

	let descend = Node::new(Data::Descend { level: descend_level });
	let ascend = Node::new(Data::Ascend { level: ascend_level });

	Port::link(&left_anchor, &descend.main());
	Port::link(&descend.aux(0), &ascend.aux(0));
	Port::link(&ascend.main(), &right_anchor);

	(vec![left_anchor], vec![right_anchor])
}

fn descend_lambda(level: usize) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	let lambda = Node::new(Data::Lambda { live: true });
	let descend_in = Node::new(Data::Descend { level: level + 1 });
	let descend_out = Node::new(Data::Descend { level: level + 1 });

	Port::link(&left_anchor, &lambda.main());

	Port::link(&lambda.aux(0), &descend_in.aux(0));
	Port::link(&descend_in.main(), &right_anchor_in);

	Port::link(&lambda.aux(1), &descend_out.aux(0));
	Port::link(&descend_out.main(), &right_anchor_out);

	(vec![left_anchor], vec![right_anchor_in, right_anchor_out])
}

fn descend_application(level: usize, live: bool) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor_in = anchor();
	let right_anchor_out = anchor();

	let application = Node::new(Data::Application { live });
	let descend_in = Node::new(Data::Descend { level });
	let descend_out = Node::new(Data::Descend { level });

	Port::link(&left_anchor, &application.main());

	Port::link(&application.aux(0), &descend_in.aux(0));
	Port::link(&descend_in.main(), &right_anchor_in);

	Port::link(&application.aux(1), &descend_out.aux(0));
	Port::link(&descend_out.main(), &right_anchor_out);

	(vec![left_anchor], vec![right_anchor_in, right_anchor_out])
}

fn descend_replicator(
	descend_level: usize,
	mut replicator_level: usize,
	replicator_count: usize,
) -> (Vec<Port>, Vec<Port>) {
	debug_assert!(replicator_level != descend_level);

	if replicator_level > descend_level {
		replicator_level -= 1;
	}

	let left_anchor = anchor();
	let mut right_anchors = Vec::new();

	let replicator = Node::new(Data::Replicator {
		level: replicator_level,
		count: replicator_count,
	});

	Port::link(&left_anchor, &replicator.main());

	for i in 0 .. replicator_count {
		let right_anchor = anchor();

		let ascend = Node::new(Data::Descend { level: descend_level });

		Port::link(&replicator.aux(i), &ascend.aux(0));
		Port::link(&ascend.main(), &right_anchor);
		right_anchors.push(right_anchor);
	}

	(vec![left_anchor], right_anchors)
}

fn descend_descend(mut left_level: usize, mut right_level: usize) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor = anchor();

	match left_level.cmp(&right_level) {
		Ordering::Equal => {
			Port::link(&left_anchor, &right_anchor);

			return (vec![left_anchor], vec![right_anchor]);
		},
		Ordering::Less => right_level -= 1,
		Ordering::Greater => left_level -= 1,
	}

	swap(&mut left_level, &mut right_level);

	let left_ascend = Node::new(Data::Descend { level: left_level });
	let right_ascend = Node::new(Data::Descend { level: right_level });

	Port::link(&left_anchor, &left_ascend.main());
	Port::link(&left_ascend.aux(0), &right_ascend.aux(0));
	Port::link(&right_ascend.main(), &right_anchor);

	(vec![left_anchor], vec![right_anchor])
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

fn reformat_ascend(level: usize) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor = anchor();

	let ascend = Node::new(Data::Ascend { level });
	let reformat = Node::new(Data::Reformat);

	Port::link(&left_anchor, &ascend.main());
	Port::link(&ascend.aux(0), &reformat.aux(0));
	Port::link(&reformat.main(), &right_anchor);

	(vec![left_anchor], vec![right_anchor])
}

fn reformat_descend(level: usize) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();
	let right_anchor = anchor();

	let ascend = Node::new(Data::Descend { level });
	let reformat = Node::new(Data::Reformat);

	Port::link(&left_anchor, &ascend.main());
	Port::link(&ascend.aux(0), &reformat.aux(0));
	Port::link(&reformat.main(), &right_anchor);

	(vec![left_anchor], vec![right_anchor])
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

fn binding_unaffected(index: usize) -> (Vec<Port>, Vec<Port>) {
	let left_anchor = anchor();

	let binding = Node::new(Data::Binding { index });

	Port::link(&left_anchor, &binding.main());

	(vec![left_anchor], Vec::new())
}
