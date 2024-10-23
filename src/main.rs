use std::path::{self, Path};

use config::Config;

mod config;
pub mod decomp;
pub mod download;
pub mod git;
mod sources;

const REDOWNLOAD_MANIFEST: bool = !cfg!(debug_assertions);

pub const REPO_PATH: &str = "repo";
pub const LOCKFILE_PATH: &str = "lockfile";

fn main() {
    let lockfile_path = path::absolute(Path::new(LOCKFILE_PATH)).unwrap();
    let _lock = match lockfile::Lockfile::create(&lockfile_path) {
        Ok(lock) => lock,
        Err(err) => match err {
            lockfile::Error::Io(error) => panic!("Failed to create lockfile: {}", error),
            lockfile::Error::LockTaken => panic!("Another instance of decompiled_minecraft_gen is already running (lockfile exists at {})", lockfile_path.display()),
            _ => panic!("{err}")
        },
    };

    let config = Config::load();

    if REDOWNLOAD_MANIFEST {
        download::mojang::download_version_manifest();
    }

    git::ensure_repo_created();

    sources::mojmap::generate(&config);
    sources::authlib::generate(&config);

    git::push();
}
