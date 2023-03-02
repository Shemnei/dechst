use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::os::raw::RawOsString;

pub type Segment = RawOsString;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathBuf(Vec<Segment>);

impl PathBuf {
	pub fn new() -> Self {
		Self(Vec::new())
	}

	pub fn with_capacity(capacity: usize) -> Self {
		Self(Vec::with_capacity(capacity))
	}

	pub fn push(&mut self, segment: Segment) {
		self.0.push(segment)
	}

	pub fn pop(&mut self) -> Option<Segment> {
		self.0.pop()
	}
}

impl fmt::Debug for PathBuf {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let path: &Path = self.as_ref();
		fmt::Debug::fmt(path, f)
	}
}

impl fmt::Display for PathBuf {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let path: &Path = self.as_ref();
		fmt::Display::fmt(path, f)
	}
}

impl AsRef<[Segment]> for PathBuf {
	fn as_ref(&self) -> &[Segment] {
		&self.0
	}
}

impl AsRef<Path> for PathBuf {
	fn as_ref(&self) -> &Path {
		Path::new(self)
	}
}

impl Deref for PathBuf {
	type Target = Path;

	fn deref(&self) -> &Self::Target {
		Path::new(self)
	}
}

pub struct Path([Segment]);

impl Path {
	const fn from_inner(inner: &[Segment]) -> &Self {
		unsafe { std::mem::transmute(inner) }
	}

	pub fn new<P: AsRef<[Segment]> + ?Sized>(p: &P) -> &Self {
		Self::from_inner(p.as_ref())
	}

	pub fn segments(&self) -> std::slice::Iter<'_, Segment> {
		self.0.iter()
	}

	pub fn split_head(&self) -> Option<(&Segment, &Path)> {
		let [head, tail @ .. ] = &self.0 else  {
				return None;
			};

		Some((head, Path::new(tail)))
	}
}

impl fmt::Debug for Path {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (i, segment) in self.0.iter().enumerate() {
			if i == 0 {
				f.write_fmt(format_args!("{}", segment))?;
			} else {
				f.write_fmt(format_args!("/{}", segment))?;
			}
		}

		Ok(())
	}
}

impl fmt::Display for Path {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(&self, f)
	}
}

impl AsRef<[Segment]> for Path {
	fn as_ref(&self) -> &[Segment] {
		&self.0
	}
}

impl AsRef<Path> for Path {
	fn as_ref(&self) -> &Path {
		self
	}
}

impl AsRef<Path> for [Segment] {
	fn as_ref(&self) -> &Path {
		Path::new(self)
	}
}
