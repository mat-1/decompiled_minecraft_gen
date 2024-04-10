use std::path::PathBuf;

use crate::download;

pub fn get_authlib_version_for_minecraft_version(mc_id: &str) -> String {
    let version_data = download::mojang::get_version_data(mc_id);
    let authlib = &version_data
        .libraries
        .into_iter()
        .find(|l| l.name.starts_with("com.mojang:authlib:"))
        .expect("version has no authlib jar");
    let authlib_version = authlib.name.strip_prefix("com.mojang:authlib:").unwrap();
    authlib_version.to_owned()
}

pub fn get_version_jar(id: &str) -> PathBuf {
    let jar_url =
        format!("https://libraries.minecraft.net/com/mojang/authlib/{id}/authlib-{id}.jar");
    let jar_path = PathBuf::from(format!("tmp/authlib/{id}.jar"));
    download::get_file_from_url_or_cache(&jar_url, &jar_path);
    jar_path
}
