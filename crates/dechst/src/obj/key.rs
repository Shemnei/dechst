use argon2::{Argon2, ParamsBuilder, PasswordHasher};
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::id::Id;
use crate::obj::{ObjectKind, RepoObject};
use crate::process::encrypt::{Encrypt, Encryption};

#[serde_with::apply(Option => #[serde(default, skip_serializing_if = "Option::is_none")])]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct KeyBytes {
	encrypt_key: Vec<u8>,
	verify_key: Vec<u8>,
	identify_key: Vec<u8>,
	chunker_key: Vec<u8>,
}

impl KeyBytes {
	pub fn from_bytes(bytes: &[u8]) -> Self {
		assert!(bytes.len() / 4 > 16, "Weak key bytes");

		let mut buf1 = Vec::with_capacity(bytes.len() / 4);
		let mut buf2 = Vec::with_capacity(bytes.len() / 4);
		let mut buf3 = Vec::with_capacity(bytes.len() / 4);
		let mut buf4 = Vec::with_capacity(bytes.len() / 4);

		for (i, &b) in bytes.iter().enumerate() {
			match i % 4 {
				0 => buf1.push(b),
				1 => buf2.push(b),
				2 => buf3.push(b),
				3 => buf4.push(b),
				_ => unreachable!(),
			}
		}

		Self {
			encrypt_key: buf1,
			verify_key: buf2,
			identify_key: buf3,
			chunker_key: buf4,
		}
	}

	pub fn random(len: usize) -> KeyBytes {
		fn _random(len: usize) -> Vec<u8> {
			let mut buf1 = vec![0; len];
			rand::thread_rng().fill_bytes(&mut buf1);
			buf1
		}

		let buf1 = _random(len);
		let buf2 = _random(len);
		let buf3 = _random(len);
		let buf4 = _random(len);

		Self {
			encrypt_key: buf1,
			verify_key: buf2,
			identify_key: buf3,
			chunker_key: buf4,
		}
	}

	pub fn encrypt_key(&self) -> &[u8] {
		&self.encrypt_key
	}

	pub fn verify_key(&self) -> &[u8] {
		&self.verify_key
	}

	pub fn identify_key(&self) -> &[u8] {
		&self.identify_key
	}

	pub fn chunker_key(&self) -> &[u8] {
		&self.chunker_key
	}
}

#[serde_with::apply(Option => #[serde(default, skip_serializing_if = "Option::is_none")])]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyMeta {
	pub hostname: Option<String>,
	pub username: Option<String>,
	pub created: DateTime<Utc>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
	#[serde(flatten)]
	meta: KeyMeta,
	#[serde(flatten)]
	bytes: KeyBytes,
}

impl Key {
	pub fn bytes(&self) -> &KeyBytes {
		&self.bytes
	}
}

impl RepoObject for Key {
	const KIND: ObjectKind = ObjectKind::Key;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptOptions {
	mem_cost: u32,
	time_cost: u32,
	parallel_cost: u32,
}

impl Default for EncryptOptions {
	fn default() -> Self {
		Self {
			mem_cost: argon2::Params::DEFAULT_M_COST,
			time_cost: argon2::Params::DEFAULT_T_COST,
			parallel_cost: argon2::Params::DEFAULT_P_COST,
		}
	}
}

impl EncryptOptions {
	pub fn to_argon2_builder(self) -> argon2::ParamsBuilder {
		let mut params = ParamsBuilder::new();
		params
			.m_cost(self.mem_cost)
			.unwrap()
			.t_cost(self.time_cost)
			.unwrap()
			.p_cost(self.parallel_cost)
			.unwrap();

		params
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedKey {
	encrypted_bytes: Vec<u8>,
	salt: [u8; 32],
	#[serde(flatten)]
	opts: EncryptOptions,
	encryption: Encryption,
}

impl EncryptedKey {
	pub fn encrypt(
		key: &Key,
		opts: EncryptOptions,
		encryption: Encryption,
		user_key: &[u8],
	) -> Self {
		let mut salt = [0; 32];
		rand::thread_rng().fill_bytes(&mut salt);
		let salt_hex = Zeroizing::new(hex::encode(salt));

		let key_bytes =
			Self::gen_key_bytes(opts, &salt_hex, user_key, encryption.key_length()).unwrap();

		let mut buf = Vec::new();

		ciborium::ser::into_writer(key, &mut buf).unwrap();

		let encrypted_bytes = encryption
			.encrypt_bytes(key_bytes.as_bytes(), &buf)
			.unwrap();

		Self {
			encrypted_bytes,
			salt,
			opts,
			encryption,
		}
	}

	pub fn decrypt(&self, user_key: &[u8]) -> Key {
		let salt_hex = Zeroizing::new(hex::encode(self.salt));

		let key_bytes =
			Self::gen_key_bytes(self.opts, &salt_hex, user_key, self.encryption.key_length())
				.unwrap();

		let decrypted_bytes = self
			.encryption
			.decrypt_bytes(key_bytes.as_bytes(), &self.encrypted_bytes)
			.unwrap();

		ciborium::de::from_reader(decrypted_bytes.as_slice()).unwrap()
	}

	fn gen_key_bytes<'a, 'b>(
		opts: EncryptOptions,
		salt_hex: &'a str,
		user_key: &'b [u8],
		key_length: u32,
	) -> Result<argon2::password_hash::PasswordHashString, argon2::password_hash::Error> {
		let mut params = opts.to_argon2_builder();
		params.output_len(key_length as usize).unwrap();
		let params = params.params().unwrap();

		let algo = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

		algo.hash_password(user_key, salt_hex)
			.map(|h| h.serialize())
	}
}
