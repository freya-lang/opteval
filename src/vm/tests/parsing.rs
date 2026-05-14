use crate::vm::Strict;
use crate::vm::tests::{application, assert_stricts_equal, binding, lambda};

enum Ast<'a> {
	Lambda { binding_name: &'a str, inner: Box<Self> },
	Application { left: Box<Self>, right: Box<Self> },
	Binding { name: &'a str },
}

fn parse<'a>(text: &mut &'a str) -> Ast<'a> {
	let mut vec = Vec::new();

	'top: loop {
		let mut iter = text.char_indices();

		while let Some((i, chr)) = iter.next() {
			if let ' ' | '\t' | '\n' = chr {
				continue;
			}

			if let '!' = chr {
				*text = &text[iter.offset() ..];
				vec.push(Ast::Lambda {
					binding_name: parse_name(text),
					inner: Box::new(parse(text)),
				});

				continue 'top;
			}

			if let 'a' ..= 'z' | 'A' ..= 'Z' | '_' = chr {
				*text = &text[i ..];
				vec.push(Ast::Binding { name: parse_name(text) });

				continue 'top;
			}

			if let '(' = chr {
				*text = &text[iter.offset() ..];
				vec.push(parse(text));

				let ')' = text.chars().next().unwrap() else {
					panic!("expected )");
				};
				*text = &text[1 ..];

				continue 'top;
			}

			break 'top;
		}

		break;
	}

	let mut iter = vec.into_iter();
	let mut out = iter.next().unwrap();

	for term in &mut iter {
		out = Ast::Application {
			left: Box::new(out),
			right: Box::new(term),
		};
	}

	out
}

fn parse_name<'a>(text: &mut &'a str) -> &'a str {
	let mut iter = text.char_indices();
	iter.next().unwrap();

	let mut index = iter.offset();

	while let Some((_, chr)) = iter.next() {
		let ('a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | '_') = chr else {
			break;
		};

		index = iter.offset();
	}

	let out = &text[0 .. index];

	*text = &text[index ..];

	out
}

#[derive(Clone, Copy, Debug)]
struct Stack<'a> {
	inner: Option<&'a Frame<'a>>,
}

#[derive(Debug)]
struct Frame<'a> {
	binding_name: &'a str,
	up: Stack<'a>,
}

impl Stack<'_> {
	fn find(self, binding: &str) -> usize {
		let mut current = self;
		let mut out = 0;

		loop {
			let frame = current.inner.unwrap();

			if frame.binding_name == binding {
				break;
			}

			current = frame.up;
			out += 1;
		}

		out
	}
}

fn convert(ast: Ast<'_>, stack: Stack<'_>) -> Strict {
	match ast {
		Ast::Lambda { binding_name, inner } => {
			let frame = Frame {
				binding_name,
				up: stack,
			};

			lambda(convert(*inner, Stack { inner: Some(&frame) }))
		},
		Ast::Application { left, right } => application(convert(*left, stack), convert(*right, stack)),
		Ast::Binding { name } => binding(stack.find(name)),
	}
}

pub(crate) fn term(mut term: &str) -> Strict {
	convert(parse(&mut term), Stack { inner: None })
}

#[test]
fn parse_id() {
	let parsed = term("!a a");

	assert_stricts_equal(&parsed, &lambda(binding(0)));
}

#[test]
fn parse_omega() {
	let parsed = term("!a a a");

	assert_stricts_equal(&parsed, &lambda(application(binding(0), binding(0))));
}

#[test]
fn parse_k() {
	let parsed = term("!x !y x");

	assert_stricts_equal(&parsed, &lambda(lambda(binding(1))));
}

#[test]
fn parse_s() {
	let parsed = term("!a !b !c a c (b c)");

	assert_stricts_equal(
		&parsed,
		&lambda(lambda(lambda(application(
			application(binding(2), binding(0)),
			application(binding(1), binding(0)),
		)))),
	);
}

#[test]
fn parse_s_with_extra_parens() {
	let parsed = term("!a !b !c (a c) (b c)");

	assert_stricts_equal(
		&parsed,
		&lambda(lambda(lambda(application(
			application(binding(2), binding(0)),
			application(binding(1), binding(0)),
		)))),
	);
}
