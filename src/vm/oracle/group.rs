use std::ops::{Mul, MulAssign};

use crate::vm::oracle::field::Element as FieldElement;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Element {
	a: FieldElement,
	b: FieldElement,
	c: FieldElement,
	d: FieldElement,
}

impl Element {
	pub(crate) fn new(a: FieldElement, b: FieldElement, c: FieldElement, d: FieldElement) -> Self {
		Self { a, b, c, d }
	}

	pub(crate) fn inv(self) -> Self {
		let constant = (self.a * self.d - self.b * self.c).inv();

		Self {
			a: constant * self.d,
			b: constant * self.b.neg(),
			c: constant * self.c.neg(),
			d: constant * self.a,
		}
	}
}

impl Mul for Element {
	type Output = Self;

	fn mul(self, other: Self) -> Self {
		Self {
			a: self.a * other.a + self.b * other.c,
			b: self.a * other.b + self.b * other.d,
			c: self.c * other.a + self.d * other.c,
			d: self.c * other.b + self.d * other.d,
		}
	}
}

impl MulAssign for Element {
	fn mul_assign(&mut self, other: Self) {
		*self = *self * other;
	}
}

#[test]
fn inversions() {
	let identity = Element {
		a: FieldElement::new_raw(1),
		b: FieldElement::new_raw(0),
		c: FieldElement::new_raw(0),
		d: FieldElement::new_raw(1),
	};

	for i in 0 .. 100 {
		let element = Element {
			a: FieldElement::new_raw(4 * i + 0),
			b: FieldElement::new_raw(4 * i + 1),
			c: FieldElement::new_raw(4 * i + 2),
			d: FieldElement::new_raw(4 * i + 3),
		};

		assert!(element * element.inv() == identity);
	}
}
