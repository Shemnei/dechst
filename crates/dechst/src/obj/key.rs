use argon2::{Argon2, ParamsBuilder, PasswordHasher};
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::obj::{ObjectKind, RepoObject};
use crate::os::User;
use crate::process::encrypt::Encryption;
use crate::process::format::{Format, Formatter};

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct KeyBytes {
	#[serde(with = "serde_bytes")]
	encrypt_key: Vec<u8>,
	#[serde(with = "serde_bytes")]
	verify_key: Vec<u8>,
	#[serde(with = "serde_bytes")]
	identify_key: Vec<u8>,
	#[serde(with = "serde_bytes")]
	chunk_key: Vec<u8>,
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
			chunk_key: buf4,
		}
	}

	pub fn random(len: usize) -> KeyBytes {
		fn _random(len: usize) -> Vec<u8> {
			let mut buf = vec![0; len];
			rand::thread_rng().fill_bytes(&mut buf);
			buf
		}

		let buf1 = _random(len);
		let buf2 = _random(len);
		let buf3 = _random(len);
		let buf4 = _random(len);

		Self {
			encrypt_key: buf1,
			verify_key: buf2,
			identify_key: buf3,
			chunk_key: buf4,
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

	pub fn chunk_key(&self) -> &[u8] {
		&self.chunk_key
	}
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyMeta {
	#[serde(flatten)]
	pub user: User,
	pub created: DateTime<Utc>,
}

impl KeyMeta {
	pub fn new() -> Self {
		Self {
			user: User::default(),
			created: Utc::now(),
		}
	}
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
	#[serde(flatten)]
	meta: KeyMeta,
	#[serde(flatten)]
	bytes: KeyBytes,
}

impl Key {
	pub fn random() -> Self {
		Self {
			meta: KeyMeta::new(),
			bytes: KeyBytes::random(32),
		}
	}

	pub fn bytes(&self) -> &KeyBytes {
		&self.bytes
	}

	pub fn meta(&self) -> &KeyMeta {
		&self.meta
	}

	pub fn encrypt(
		&self,
		opts: EncryptOptions,
		encryption: Encryption,
		user_key: &[u8],
	) -> EncryptedKey {
		EncryptedKey::encrypt(self, opts, encryption, user_key)
	}
}

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
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

#[serde_with::apply(
	Option => #[serde(default, skip_serializing_if = "Option::is_none")],
	Vec => #[serde(default, skip_serializing_if = "Vec::is_empty")]
)]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedKey {
	#[serde(with = "serde_bytes")]
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

		let bytes = Formatter::Cbor.format(key).unwrap();

		let encrypted_bytes = encryption
			.encrypt_bytes(key_bytes.as_bytes(), &bytes)
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

		Formatter::Cbor.parse(&decrypted_bytes).unwrap()
	}

	pub fn try_unencrypted(&self) -> Key {
		Formatter::Cbor.parse(&self.encrypted_bytes).unwrap()
	}

	fn gen_key_bytes<'a, 'b>(
		opts: EncryptOptions,
		salt_hex: &'a str,
		user_key: &'b [u8],
		key_length: u32,
	) -> Result<argon2::password_hash::Output, argon2::password_hash::Error> {
		let mut params = opts.to_argon2_builder();
		params.output_len(key_length as usize).unwrap();
		let params = params.params().unwrap();

		let algo = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

		algo.hash_password(user_key, salt_hex)
			.map(|x| x.hash.unwrap())
	}
}

impl RepoObject for EncryptedKey {
	const KIND: ObjectKind = ObjectKind::Key;
}
