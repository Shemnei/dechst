use dechst::backend::ext::Find;
use dechst::backend::BackendWrite;
use dechst::id::Id;
use dechst::repo::{DecryptedRepo, Repo};

use crate::opts::RepoOpts;
use crate::password::Password;

// TODO: Interactive key chooser
pub fn unlock_repo<B: BackendWrite>(
	repo: Repo<B>,
	opts: &RepoOpts,
) -> Result<DecryptedRepo<B>, (Repo<B>, anyhow::Error)> {
	let key = opts.key.as_ref();
	let key: Option<&str> = if let Some(s) = key { Some(&s) } else { None };

	if opts.no_password {
		log::debug!("Option `no-password` given");

		try_unlock(repo, key, None)
	} else {
		log::debug!("Getting password");

		let password = match Password::get(opts) {
			Ok(pw) => pw,
			Err(err) => return Err((repo, err)),
		};

		try_unlock(repo, key, password)
	}
}

pub fn try_unlock<B: BackendWrite>(
	repo: Repo<B>,
	key: Option<&str>,
	password: Option<Password>,
) -> Result<DecryptedRepo<B>, (Repo<B>, anyhow::Error)> {
	if let Some(password) = password {
		try_unlock_encrypted(repo, key, password)
	} else {
		try_unlock_unencrypted(repo, key)
	}
}

pub fn try_unlock_encrypted<B: BackendWrite>(
	repo: Repo<B>,
	key: Option<&str>,
	password: Password,
) -> Result<DecryptedRepo<B>, (Repo<B>, anyhow::Error)> {
	if let Some(key) = key {
		try_unlock_encrypted_single(repo, key, password)
	} else {
		try_unlock_encrypted_all(repo, password)
	}
}

pub fn try_unlock_encrypted_single<B: BackendWrite>(
	repo: Repo<B>,
	key: &str,
	password: Password,
) -> Result<DecryptedRepo<B>, (Repo<B>, anyhow::Error)> {
	let key_id = match get_key_id(&repo, key) {
		Ok(id) => id,
		Err(err) => return Err((repo, err)),
	};

	repo.decrypt(key_id, password.as_bytes())
		.map_err(|(repo, _)| (repo, anyhow::anyhow!("Failed to decrypt the repository")))
}

pub fn try_unlock_encrypted_all<B: BackendWrite>(
	repo: Repo<B>,
	password: Password,
) -> Result<DecryptedRepo<B>, (Repo<B>, anyhow::Error)> {
	let keys = match repo.keys() {
		Ok(keys) => keys,
		Err(_) => return Err((repo, anyhow::anyhow!("Failed to list keys"))),
	};

	let mut r = repo;

	for key in keys {
		let Ok(key) =  key else {continue};

		match r.decrypt(key, password.as_bytes()) {
			Ok(repo) => return Ok(repo),
			Err((repo, _)) => {
				r = repo;
			}
		}
	}

	Err((r, anyhow::anyhow!("Failed to find key")))
}

pub fn try_unlock_unencrypted<B: BackendWrite>(
	repo: Repo<B>,
	key: Option<&str>,
) -> Result<DecryptedRepo<B>, (Repo<B>, anyhow::Error)> {
	if let Some(key) = key {
		try_unlock_unencrypted_single(repo, key)
	} else {
		try_unlock_unencrypted_all(repo)
	}
}

pub fn try_unlock_unencrypted_single<B: BackendWrite>(
	repo: Repo<B>,
	key: &str,
) -> Result<DecryptedRepo<B>, (Repo<B>, anyhow::Error)> {
	let key_id = match get_key_id(&repo, key) {
		Ok(id) => id,
		Err(err) => return Err((repo, err)),
	};

	repo.try_unencrypted(key_id)
		.map_err(|(repo, _)| (repo, anyhow::anyhow!("Failed to decrypt the repository")))
}

pub fn try_unlock_unencrypted_all<B: BackendWrite>(
	repo: Repo<B>,
) -> Result<DecryptedRepo<B>, (Repo<B>, anyhow::Error)> {
	let keys = match repo.keys() {
		Ok(keys) => keys,
		Err(_) => return Err((repo, anyhow::anyhow!("Failed to list keys"))),
	};

	let mut r = repo;

	for key in keys {
		let Ok(key) =  key else {continue};

		match r.try_unencrypted(key) {
			Ok(repo) => return Ok(repo),
			Err((repo, _)) => {
				r = repo;
			}
		}
	}

	Err((r, anyhow::anyhow!("Failed to find key")))
}

fn get_key_id<B: BackendWrite>(repo: &Repo<B>, key: &str) -> anyhow::Result<Id> {
	match repo.find_key_id(key) {
		Ok(Some(Find::Unique(id))) => Ok(id),
		Ok(Some(Find::NonUnique)) => anyhow::bail!("Multiple matching keys found"),
		Ok(_) => anyhow::bail!("No matching key found"),
		Err(err) => anyhow::bail!("Failed to retrive keys"),
	}
}
