mod parsing;

use crate::vm::term::{Lazy, Strict, Term};
use crate::vm::tests::parsing::term;

fn lambda(body: Strict) -> Strict {
	Strict::new(Term::Lambda { body })
}

fn application(left: Strict, right: Strict) -> Strict {
	Strict::new(Term::Application { left, right })
}

fn application_chain(left: Strict, right: impl IntoIterator<Item = Strict>) -> Strict {
	let mut out = left;

	for item in right {
		out = application(out, item);
	}

	out
}

fn binding(index: usize) -> Strict {
	Strict::new(Term::Binding { index })
}

fn assert_stricts_equal(a: &Strict, b: &Strict) {
	match (a.get(), b.get()) {
		(Term::Lambda { body: body_a }, Term::Lambda { body: body_b }) => {
			assert_stricts_equal(body_a, body_b);
		},
		(
			Term::Application {
				left: left_a,
				right: right_a,
			},
			Term::Application {
				left: left_b,
				right: right_b,
			},
		) => {
			assert_stricts_equal(left_a, left_b);
			assert_stricts_equal(right_a, right_b);
		},
		(Term::Binding { index: index_a }, Term::Binding { index: index_b }) => {
			assert_eq!(index_a, index_b);
		},
		_ => panic!("mismatching term types"),
	}
}

fn id() -> Strict {
	lambda(binding(0))
}

fn encode_number(n: usize) -> Strict {
	let mut inner = binding(0);

	for _ in 0 .. n {
		inner = application(binding(1), inner);
	}

	lambda(lambda(inner))
}

#[test]
fn id_on_id() {
	let id = id();
	let id_on_id = application(id.clone(), id.clone());

	let resolved = Lazy::encode(&id_on_id).to_strict();

	assert_stricts_equal(&resolved, &id);
}

#[test]
fn two_on_id() {
	let two = encode_number(2);
	let id = id();
	let two_on_id = application(two.clone(), id.clone());

	let resolved = Lazy::encode(&two_on_id).to_strict();

	assert_stricts_equal(&resolved, &id);
}

#[test]
fn id_on_two() {
	let id = id();
	let two = encode_number(2);
	let id_on_two = application(id.clone(), two.clone());

	let resolved = Lazy::encode(&id_on_two).to_strict();

	assert_stricts_equal(&resolved, &two);
}

#[test]
fn two_on_two() {
	let two = encode_number(2);
	let two_on_two = application(two.clone(), two.clone());
	let four = encode_number(4);

	let resolved = Lazy::encode(&two_on_two).to_strict();

	assert_stricts_equal(&resolved, &four);
}

#[test]
fn fuse_two_and_two() {
	let two = encode_number(2);
	let fused = lambda(application(two.clone(), application(two.clone(), binding(0))));
	let four = encode_number(4);

	let resolved = Lazy::encode(&fused).to_strict();

	assert_stricts_equal(&resolved, &four);
}

#[test]
fn modular_exponentiation() {
	let type_0 = lambda(lambda(lambda(binding(2))));
	let type_1 = lambda(lambda(lambda(binding(1))));
	let type_2 = lambda(lambda(lambda(binding(0))));
	let rotate = lambda(lambda(application_chain(binding(0), [
		application(binding(1), type_1.clone()),
		application(binding(1), type_2.clone()),
		application(binding(1), type_0.clone()),
	])));
	let counter_rotate = lambda(lambda(application_chain(binding(0), [
		application(binding(1), type_2.clone()),
		application(binding(1), type_0.clone()),
		application(binding(1), type_1.clone()),
	])));

	const CYCLES: usize = 10;
	let expected = if CYCLES % 2 == 0 { &rotate } else { &counter_rotate };

	let mut expr = rotate.clone();

	for _ in 0 .. CYCLES {
		expr = application(encode_number(2), expr);
	}

	let resolved = Lazy::encode(&expr).to_strict();

	assert_stricts_equal(&resolved, expected);
}

#[test]
fn two_on_counter_rotate() {
	let type_0 = lambda(lambda(lambda(binding(2))));
	let type_1 = lambda(lambda(lambda(binding(1))));
	let type_2 = lambda(lambda(lambda(binding(0))));
	let counter_rotate = lambda(lambda(application_chain(binding(0), [
		application(binding(1), type_1.clone()),
		application(binding(1), type_2.clone()),
		application(binding(1), type_0.clone()),
	])));
	let expected = lambda(lambda(application_chain(binding(0), [
		application(binding(1), type_0.clone()),
		application(binding(1), type_1.clone()),
		application(binding(1), type_2.clone()),
	])));

	let expr = application(encode_number(3), counter_rotate);
	let resolved = Lazy::encode(&expr).to_strict();

	assert_stricts_equal(&resolved, &expected);
}

#[test]
fn duplicate_on_omega() {
	let duplicate = lambda(lambda(application_chain(binding(0), [binding(1), binding(1)])));
	let omega = lambda(application(binding(0), binding(0)));
	let expected = lambda(application_chain(binding(0), [omega.clone(), omega.clone()]));

	let expr = application(duplicate.clone(), omega.clone());
	let resolved = Lazy::encode(&expr).to_strict();

	assert_stricts_equal(&resolved, &expected);
}

#[test]
fn triplicate_on_omega() {
	let triplicate = lambda(lambda(application_chain(binding(0), [
		binding(1),
		binding(1),
		binding(1),
	])));
	let omega = lambda(application(binding(0), binding(0)));
	let expected = lambda(application_chain(binding(0), [
		omega.clone(),
		omega.clone(),
		omega.clone(),
	]));

	let expr = application(triplicate.clone(), omega.clone());
	let resolved = Lazy::encode(&expr).to_strict();

	assert_stricts_equal(&resolved, &expected);
}

#[test]
fn many_exponentiations_on_id_id() {
	const N: usize = 250;

	let id = id();

	let expr = application(
		application(application(encode_number(N), encode_number(2)), id.clone()),
		id.clone(),
	);

	let resolved = Lazy::encode(&expr).to_strict();

	assert_stricts_equal(&resolved, &id);
}

#[test]
fn counterterm() {
	let expr = term("(!f f (f (!x x))) (!i (!f f (!x x) (f (!x x))) (!a (!f !x f (f x)) (!b a (i b))))");
	let expected = id();
	let resolved = Lazy::encode(&expr).to_strict();

	assert_stricts_equal(&resolved, &expected);
}
