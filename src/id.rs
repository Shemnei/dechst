use std::fmt;
use std::hash::Hash;
use std::ops::Deref;
use std::str::FromStr;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub(crate) const WIDTH: usize = 32;

#[rustfmt::skip]
const HEX_LOWER: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'a', 'b', 'c', 'd', 'e', 'f',
];

#[rustfmt::skip]
const HEX_UPPER: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    'A', 'B', 'C', 'D', 'E', 'F',
];

macro_rules! to_hex {
    ( $( $fn:ident: $alphabet:ident )+ ) => {
        $(
            pub fn $fn(self) -> String {
                // One byte expands to two hexadecimal digits
                let mut out = String::with_capacity(WIDTH * 2);

                for byte in &self.0 {
                    // Index for upper hexadecimal digit
                    let idxu = (byte >> 4) as usize;
                    // Index for lower hexadecimal digit
                    let idxl = (byte & 0xf) as usize;

                    out.push($alphabet[idxu]);
                    out.push($alphabet[idxl]);
                }

                out
            }
        )+
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(pub(crate) [u8; WIDTH]);

impl Id {
	to_hex! {
		to_hex_lower: HEX_LOWER
		to_hex_upper: HEX_UPPER
	}

	pub fn from_bytes(slice: &[u8]) -> Self {
		let mut id = [0u8; WIDTH];

		// Only copy up to `WIDTH` digits
		let len = ::std::cmp::min(slice.len(), WIDTH);
		id[..len].copy_from_slice(&slice[..len]);

		Self(id)
	}

	pub fn as_bytes(&self) -> &[u8] {
		&self.0
	}

	pub fn idd<T>(self, value: T) -> Idd<T> {
		Idd::new(self, value)
	}
}

impl Deref for Id {
	type Target = [u8; WIDTH];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl AsRef<[u8; WIDTH]> for Id {
	fn as_ref(&self) -> &[u8; WIDTH] {
		&self.0
	}
}

impl From<[u8; WIDTH]> for Id {
	fn from(value: [u8; WIDTH]) -> Self {
		Self(value)
	}
}

impl From<&[u8]> for Id {
	fn from(value: &[u8]) -> Self {
		Self::from_bytes(value)
	}
}

impl From<Id> for [u8; WIDTH] {
	fn from(value: Id) -> Self {
		value.0
	}
}

impl fmt::Debug for Id {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.to_hex_lower())
	}
}

impl fmt::Display for Id {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.to_hex_lower())
	}
}

impl fmt::LowerHex for Id {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.to_hex_lower())
	}
}

impl fmt::UpperHex for Id {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.to_hex_upper())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseHexIdError;

impl fmt::Display for ParseHexIdError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(&self, f)
	}
}

impl ::std::error::Error for ParseHexIdError {}

impl FromStr for Id {
	type Err = ParseHexIdError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let bytes = s.as_bytes();

		if bytes.len() != WIDTH * 2 {
			return Err(ParseHexIdError);
		}

		let mut id = [0u8; WIDTH];

		for (idx, hex_digit) in bytes.chunks_exact(2).take(WIDTH).enumerate() {
			id[idx] = u8::from_str_radix(
				&format!("{}{}", hex_digit[0] as char, hex_digit[1] as char),
				16,
			)
			.map_err(|_| ParseHexIdError)?;
		}

		Ok(Id(id))
	}
}

impl Serialize for Id {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.to_hex_lower())
	}
}

struct IdVisitor;

impl<'de> Visitor<'de> for IdVisitor {
	type Value = Id;

	fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str("an valid hexadecimal id")
	}

	fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		value.parse().map_err(E::custom)
	}

	fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		value.parse().map_err(E::custom)
	}
}

impl<'de> Deserialize<'de> for Id {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(IdVisitor)
	}
}

/// # Note
///
/// Equality (PartialEq, Eq) is only done via the id, the `inner` field is
/// completly ignored (even the type).
pub struct Idd<T> {
	id: Id,
	value: T,
}

impl<T> Idd<T> {
	pub fn new(id: Id, value: T) -> Self {
		Self { id, value }
	}

	pub fn id(&self) -> &Id {
		&self.id
	}

	pub fn value(&self) -> &T {
		&self.value
	}

	pub fn into_inner(self) -> (Id, T) {
		(self.id, self.value)
	}

	pub fn into_value(self) -> T {
		self.value
	}
}

impl<T> From<(Id, T)> for Idd<T> {
	fn from(value: (Id, T)) -> Self {
		Self::new(value.0, value.1)
	}
}

impl<T> From<Idd<T>> for (Id, T) {
	fn from(value: Idd<T>) -> Self {
		(value.id, value.value)
	}
}

impl<T> Deref for Idd<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T> AsRef<T> for Idd<T> {
	fn as_ref(&self) -> &T {
		&self.value
	}
}

impl<T> fmt::Debug for Idd<T>
where
	T: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Idd")
			.field("id", &self.id)
			.field("value", &self.value)
			.finish()
	}
}

impl<T> Clone for Idd<T>
where
	T: Clone,
{
	fn clone(&self) -> Self {
		Self::new(self.id, self.value.clone())
	}
}

impl<T> Copy for Idd<T> where T: Copy {}

impl<T, U> PartialEq<Idd<U>> for Idd<T> {
	fn eq(&self, other: &Idd<U>) -> bool {
		self.id.eq(&other.id)
	}
}

impl<U> PartialEq<Idd<U>> for Id {
	fn eq(&self, other: &Idd<U>) -> bool {
		self.eq(&other.id)
	}
}

impl<T> PartialEq<Id> for Idd<T> {
	fn eq(&self, other: &Id) -> bool {
		self.id.eq(other)
	}
}

impl<T> Eq for Idd<T> {}

impl<T> PartialOrd for Idd<T>
where
	T: PartialOrd,
{
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.value.partial_cmp(&other.value)
	}
}

impl<T> Ord for Idd<T>
where
	T: Ord,
{
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.value.cmp(&other.value)
	}
}

impl<T> Hash for Idd<T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.id.hash(state)
	}
}

#[cfg(test)]
mod tests {
	use pretty_assertions::{assert_eq, assert_ne};
	use rand::prelude::*;

	use super::*;

	fn random_data(len: usize) -> Vec<u8> {
		let mut data = vec![0u8; len];
		thread_rng().fill_bytes(&mut data);
		data
	}

	fn rs_to_hex_lower(slice: &[u8]) -> String {
		slice.iter().map(|b| format!("{:02x}", b)).collect()
	}

	fn rs_to_hex_upper(slice: &[u8]) -> String {
		slice.iter().map(|b| format!("{:02X}", b)).collect()
	}

	#[test]
	fn from_bytes() {
		let data = random_data(WIDTH);
		for i in 0..WIDTH {
			let id = Id::from_bytes(&data[..i]);

			assert_eq!(&id[..i], &data[..i]);
			assert_eq!(&id[i..], vec![0u8; WIDTH - i]);
		}
	}

	#[test]
	fn to_hex_lower() {
		for _ in 0..WIDTH {
			let data = random_data(WIDTH);
			let id = Id::from_bytes(&data);
			let should = rs_to_hex_lower(&data);

			assert_eq!(id.to_hex_lower(), should);
			assert_eq!(format!("{:x}", id), should);
		}
	}

	#[test]
	fn to_hex_upper() {
		for _ in 0..WIDTH {
			let data = random_data(WIDTH);
			let id = Id::from_bytes(&data);
			let should = rs_to_hex_upper(&data);

			assert_eq!(id.to_hex_upper(), should);
			assert_eq!(format!("{:X}", id), should);
		}
	}

	#[test]
	fn idd_eq() {
		let id = Id([0xed; WIDTH]);

		let idd_one = Idd::new(id, ());
		let idd_two = Idd::new(id, 2);

		assert_eq!(idd_one, idd_two);
	}

	#[test]
	fn idd_ne() {
		let id_one = Id([0xed; WIDTH]);
		let id_two = Id([0xaa; WIDTH]);

		let idd_one = Idd::new(id_one, 2);
		let idd_two = Idd::new(id_two, 2);

		assert_ne!(idd_one, idd_two);
	}
}
