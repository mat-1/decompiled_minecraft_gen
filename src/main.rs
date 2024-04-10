use config::Config;

mod config;
pub mod decomp;
pub mod download;
pub mod git;
mod sources;

const REDOWNLOAD_MANIFEST: bool = !cfg!(debug_assertions);

fn main() {
    let config = Config::load();

    if REDOWNLOAD_MANIFEST {
        download::mojang::download_version_manifest();
    }

    git::ensure_repo_created();

    sources::mojmap::generate(&config);
    sources::authlib::generate(&config);

    git::push();
}
