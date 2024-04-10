pub mod decomp;
pub mod download;
pub mod git;
mod sources;

const REDOWNLOAD_MANIFEST: bool = cfg!(debug_assertions);

fn main() {
    if REDOWNLOAD_MANIFEST {
        download::mojang::download_version_manifest();
    }
    git::ensure_repo_created();

    sources::mojmap::generate();
    sources::authlib::generate();
}
