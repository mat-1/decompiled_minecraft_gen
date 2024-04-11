use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use serde::Deserialize;

use super::{download_file_from_url_to_path, get_file_from_url_or_cache};

const VERSION_MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest.json";
const VERSION_MANIFEST_SAVE_PATH: &str = "tmp/mojang/version_manifest.json";

#[derive(Deserialize, Debug)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<ManifestVersion>,
}

#[derive(Deserialize, Debug)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ManifestVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub time: DateTime<Utc>,
    pub release_time: DateTime<Utc>,
}

pub fn get_version_manifest() -> VersionManifest {
    let version_manifest =
        get_file_from_url_or_cache(VERSION_MANIFEST_URL, Path::new(VERSION_MANIFEST_SAVE_PATH));
    serde_json::from_slice::<VersionManifest>(&version_manifest)
        .expect("failed to parse version_manifest.json")
}

pub fn download_version_manifest() -> VersionManifest {
    let version_manifest =
        download_file_from_url_to_path(VERSION_MANIFEST_URL, Path::new(VERSION_MANIFEST_SAVE_PATH));
    serde_json::from_slice::<VersionManifest>(&version_manifest)
        .expect("failed to parse version_manifest.json")
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VersionData {
    pub downloads: HashMap<String, VersionDownload>,
    pub id: String,
    pub java_version: JavaVersion,
    pub libraries: Vec<Library>,
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VersionDownload {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub downloads: LibraryDownloads,
    pub name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryArtifact>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LibraryArtifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

pub fn get_version_data(id: &str) -> VersionData {
    let manifest = get_version_manifest();
    let manifest_version = manifest
        .versions
        .iter()
        .find(|v| v.id == id)
        .unwrap_or_else(|| panic!("couldn't find version {id} in version_manifest.json"));
    let version_data = get_file_from_url_or_cache(
        &manifest_version.url,
        Path::new(&format!("tmp/mojang/{id}.json")),
    );
    serde_json::from_slice::<VersionData>(&version_data)
        .unwrap_or_else(|e| panic!("failed to parse version data for {id}: {e}"))
}

pub fn get_version_jar(id: &str) -> PathBuf {
    let version_data = get_version_data(id);
    let jar_url = &version_data
        .downloads
        .get("client")
        .expect("version has no jar")
        .url;
    let jar_path = PathBuf::from(format!("tmp/mojang/{id}.jar"));
    get_file_from_url_or_cache(jar_url, &jar_path);
    jar_path
}

pub fn get_version_mappings(id: &str) -> Option<PathBuf> {
    let version_data = get_version_data(id);
    let jar_url = &version_data.downloads.get("client_mappings")?.url;
    let jar_path = PathBuf::from(format!("tmp/mojang/{id}-mappings.txt"));
    get_file_from_url_or_cache(jar_url, &jar_path);
    Some(jar_path)
}
