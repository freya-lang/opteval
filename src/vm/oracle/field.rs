use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Element {
	inner: u64,
}

const SHIFT: u32 = 61;
const MODULUS: u64 = (1 << SHIFT) - 1;
const MODULUS_U128: u128 = (1 << SHIFT) - 1;

impl Element {
	pub(crate) fn new_raw(inner: u64) -> Self {
		debug_assert!(inner < MODULUS);

		Self { inner }
	}

	pub(crate) fn new_from_randomness(source: u64) -> Option<Self> {
		let masked = source & MODULUS;

		if masked == MODULUS {
			return None;
		}

		Some(Self::new_raw(masked))
	}

	pub(crate) fn inv(self) -> Self {
		debug_assert!(self.inner != 0);

		let mut out = self;

		for _ in 0 .. SHIFT - 3 {
			out *= out * self;
		}

		out *= out;
		out *= out;

		out * self
	}

	pub(crate) fn neg(self) -> Self {
		Self::new_raw(0) - self
	}
}

fn reduce_u64(val: u64) -> u64 {
	let val = (val & MODULUS) + (val >> SHIFT);

	if val >= MODULUS {
		return val - MODULUS;
	}

	val
}

fn reduce_u128(val: u128) -> u64 {
	let val = (val & MODULUS_U128) + (val >> SHIFT);
	let val = (val & MODULUS_U128) + (val >> SHIFT);

	let val: u64 = val.try_into().unwrap();

	if val >= MODULUS {
		return val - MODULUS;
	}

	val
}

impl Add for Element {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Self::new_raw(reduce_u64(self.inner + other.inner))
	}
}

impl Sub for Element {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		Self::new_raw(reduce_u64(MODULUS + self.inner - other.inner))
	}
}

impl Mul for Element {
	type Output = Self;

	fn mul(self, other: Self) -> Self {
		let a = u128::from(self.inner);
		let b = u128::from(other.inner);

		Self::new_raw(reduce_u128(a * b))
	}
}

impl Div for Element {
	type Output = Self;

	fn div(self, other: Self) -> Self {
		self * other.inv()
	}
}

impl AddAssign for Element {
	fn add_assign(&mut self, other: Self) {
		*self = *self + other;
	}
}

impl SubAssign for Element {
	fn sub_assign(&mut self, other: Self) {
		*self = *self - other;
	}
}

impl MulAssign for Element {
	fn mul_assign(&mut self, other: Self) {
		*self = *self * other;
	}
}

impl DivAssign for Element {
	fn div_assign(&mut self, other: Self) {
		*self = *self / other;
	}
}
