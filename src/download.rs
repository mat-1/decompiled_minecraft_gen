use std::{fs, path::Path};

pub mod authlib;
pub mod mojang;

pub struct DownloadResult {}

pub fn download_file_from_url_to_path(url: &str, path: &Path) -> Vec<u8> {
    println!("downloading {url} to {path:?}");
    let res = ureq::get(url)
        .call()
        .unwrap_or_else(|_| panic!("failed to download {url}"));
    let mut reader = res.into_reader();
    let mut buf = Vec::new();
    reader
        .read_to_end(&mut buf)
        .unwrap_or_else(|_| panic!("failed to read {url}"));
    let _ = fs::create_dir_all(path.parent().expect("path should have a parent"));
    fs::write(path, buf.clone()).unwrap_or_else(|_| panic!("failed to write to {path:?}"));
    buf
}

pub fn get_file_from_url_or_cache(url: &str, path: &Path) -> Vec<u8> {
    if let Ok(buf) = fs::read(path) {
        buf
    } else {
        download_file_from_url_to_path(url, path)
    }
}
