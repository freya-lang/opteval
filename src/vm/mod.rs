mod inet;
mod oracle;
mod term;

#[cfg(test)]
mod tests;

pub(crate) use crate::vm::term::{Lazy, Strict, Term};
