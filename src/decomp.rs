use std::{fs, io, path::Path, process::Command};

const VINEFLOWER_PATH: &str = "./lib/vineflower-1.10.0.jar";
// if you're updating it, download from https://repo1.maven.org/maven2/net/md-5/SpecialSource/
// make sure to use the -shaded jar! otherwise it won't work
const SPECIALSOURCE_PATH: &str = "./lib/SpecialSource-1.11.4-shaded.jar";

pub fn decompile_jar(jar_path: &Path, out_path: &Path) {
    // make sure we don't delete /
    assert!(
        out_path.parent().is_some(),
        "decompile out path ({out_path:?}) can't be the root"
    );

    if !out_path
        .try_exists()
        .expect("out path directory ({out_path:?}) must be readable")
    {
        fs::create_dir_all(out_path).expect("couldn't create out");
    }
    let entries_in_out_path = out_path
        .read_dir()
        .expect("out path must be readable")
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    // if it's not empty then make sure there's a META-INF here JUST to be safe
    if !entries_in_out_path.is_empty()
        && !entries_in_out_path
            .iter()
            .any(|e| e.file_name() == "META-INF")
    {
        panic!("tried to decompile into a non-empty directory that doesn't look like a previous decompilation output (no META-INF). delete it yourself if you're sure it's fine.")
    }

    println!("deleting everything in {out_path:?}");
    for entry in entries_in_out_path {
        if entry.path().is_dir() {
            if let Err(e) = fs::remove_dir_all(entry.path()) {
                println!("couldn't remove directory {entry:?}! {e}");
            }
        } else if let Err(e) = fs::remove_file(entry.path()) {
            println!("couldn't remove file {entry:?}! {e}");
        }
    }

    // make sure it's empty
    assert!(
        out_path.read_dir().unwrap().next().is_none(),
        "out_path ({out_path:?}) should be empty"
    );

    println!("Decompiling {jar_path:?}");
    Command::new("java")
        .args([
            "-Xmx4G",
            "-Xms1G",
            "-jar",
            VINEFLOWER_PATH,
            jar_path.to_str().unwrap(),
            out_path.to_str().unwrap(),
        ])
        .stdout(io::stdout())
        .stderr(io::stderr())
        .output()
        .expect("failed to execute vineflower");
    println!("Finished decompiling");
}

pub fn remap_jar_with_srg_mappings(
    jar_input_path: &Path,
    jar_output_path: &Path,
    mappings_path: &Path,
) {
    println!("Remapping {jar_input_path:?} with mappings {mappings_path:?}");
    Command::new("java")
        .args([
            "-Xmx4G",
            "-Xms1G",
            "-jar",
            SPECIALSOURCE_PATH,
            "--in-jar",
            jar_input_path.to_str().unwrap(),
            "--out-jar",
            jar_output_path.to_str().unwrap(),
            "--srg-in",
            mappings_path.to_str().unwrap(),
        ])
        // .stdout(io::stdout())
        // .stderr(io::stderr())
        .output()
        .expect("failed to execute vineflower");
}
