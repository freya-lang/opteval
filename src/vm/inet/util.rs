use crate::vm::inet::base::{Data, Node, Port};

pub(crate) fn anchor() -> Port {
	Node::new(Data::Anchor).main()
}

pub(crate) fn join(side_a: &Port, side_b: &Port) {
	Port::link(&side_a.unlink(), &side_b.unlink());
}

pub(crate) fn join_slice(side_a: &[Port], side_b: &[Port]) {
	for (side_a, side_b) in side_a.iter().zip(side_b.iter()) {
		join(side_a, side_b);
	}
}
