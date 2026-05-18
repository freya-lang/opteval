use crate::vm::{Strict, Term};

fn lambda(body: Strict) -> Strict {
	Strict::new(Term::Lambda { body })
}

fn application(left: Strict, right: Strict) -> Strict {
	Strict::new(Term::Application { left, right })
}

fn binding(index: usize) -> Strict {
	Strict::new(Term::Binding { index })
}

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

				let ')' = text.chars().next().expect("expected ), found <eof>") else {
					panic!("expected )");
				};
				*text = &text[1 ..];

				continue 'top;
			}

			if let ')' = chr {
				*text = &text[i ..];
				break 'top;
			}

			panic!("unexpected character: \"{chr}\"");
		}

		break;
	}

	let mut iter = vec.into_iter();
	let Some(mut out) = iter.next() else {
		panic!(
			"expected expression, found {}",
			text.chars()
				.next()
				.map(|x| format!("\"{x}\""))
				.unwrap_or("<eof>".to_string())
		);
	};

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
	let initial = iter.next().expect("expected name after !, found <eof>");

	let ('a' ..= 'z' | 'A' ..= 'Z' | '_') = initial.1 else {
		panic!("expected name after !, found \"{}\"", initial.1);
	};

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
			let Some(frame) = current.inner else {
				panic!("could not find corresponding lambda for binding {binding}");
			};

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
	let parsed = parse(&mut term);

	if term != "" {
		panic!(
			"parsing ended early, found \"{}\" instead of <eof>",
			term.chars().next().unwrap()
		);
	}

	convert(parsed, Stack { inner: None })
}
