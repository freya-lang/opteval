mod parsing;
mod vm;

use std::env::args;
use std::fmt::Write;

use crate::parsing::term;
use crate::vm::{Lazy, Strict, Term};

fn main() {
	let mut args = args();
	args.next().expect("there should be an executable path");

	let to_parse = args.next().expect("no command line argument to parse");
	assert!(
		args.next().is_none(),
		"received more than one argument - not sure what to do with that, please pass only one"
	);

	let parsed = term(&to_parse);
	let encoded = Lazy::encode(&parsed);
	let resolved = encoded.to_strict();

	let mut output = String::new();

	print_term(&mut output, 0, &resolved);

	println!("{}", output);
}

fn print_term(output: &mut String, depth: usize, term: &Strict) {
	match term.get() {
		Term::Lambda { body } => {
			write!(output, "(!x{depth} ").unwrap();
			print_term(output, depth + 1, body);
			write!(output, ")").unwrap();
		},
		Term::Application { left, right } => {
			write!(output, "(").unwrap();
			print_term(output, depth, left);
			write!(output, " ").unwrap();
			print_term(output, depth, right);
			write!(output, ")").unwrap();
		},
		Term::Binding { index } => {
			write!(output, "x{}", depth - index - 1).unwrap();
		},
	}
}
