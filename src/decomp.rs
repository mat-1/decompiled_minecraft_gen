use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

use crate::config::Config;

const VINEFLOWER_PATH: &str = "./lib/vineflower-1.10.1+patched.jar";
// if you're updating it, download from https://repo1.maven.org/maven2/net/md-5/SpecialSource/
// make sure to use the -shaded jar! otherwise it won't work
const SPECIALSOURCE_PATH: &str = "./lib/SpecialSource-1.11.4-shaded.jar";

pub fn decompile_jar(config: &Config, jar_path: &Path, out_path: &Path) {
    let expected = expected_sources(jar_path).expect("could not inventory input JAR");
    for attempt_number in 0..3 {
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
        // Minecraft uses net/, authlib uses com/; partial output from either may need a retry.
        if !entries_in_out_path.is_empty()
            && !entries_in_out_path
                .iter()
                .any(|e| e.file_name() == "net" || e.file_name() == "com")
        {
            panic!("tried to decompile into a non-empty directory that doesn't look like a previous decompilation output (no 'net' or 'com' directory). delete it yourself if you're sure it's fine.")
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
        let status = Command::new("java")
            .args(config.java_args.iter().map(String::as_str))
            .arg("-jar")
            .arg(VINEFLOWER_PATH)
            .args(config.vineflower_args.iter().map(String::as_str))
            .arg(jar_path.to_str().unwrap())
            .arg(out_path.to_str().unwrap())
            .stdout(io::stdout())
            .stderr(io::stderr())
            .status()
            .expect("failed to execute vineflower");
        println!("Finished decompiling");

        if let Err(error) = validate_decompilation(status, out_path, &expected) {
            eprintln!(
                "Decompilation attempt {} rejected: {error}",
                attempt_number + 1
            );
        } else {
            println!(
                "Decompiled successfully after {} attempt{}",
                attempt_number + 1,
                if attempt_number == 0 { "" } else { "s" }
            );
            return;
        }
    }

    panic!("Decompilation failed!");
}

// Nested/anonymous classes are emitted inside their enclosing source, not as separate files.
// Compare against the input archive, never the previous release: legitimate removals are allowed.
fn expected_sources(jar_path: &Path) -> Result<Vec<PathBuf>, String> {
    let output = Command::new("jar")
        .arg("tf")
        .arg(jar_path)
        .output()
        .map_err(|e| format!("could not run jar tf: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "jar tf failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let listing = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    let mut expected = Vec::new();
    for entry in listing.lines() {
        let Some(class) = entry.strip_suffix(".class") else {
            continue;
        };
        if class.contains('$') || class == "module-info" || class.starts_with("META-INF/") {
            continue;
        }
        let path = Path::new(class);
        if path
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(format!("unsafe class path in JAR: {entry}"));
        }
        expected.push(path.with_extension("java"));
    }
    if expected.is_empty() {
        return Err("input JAR contains no top-level classes".into());
    }
    Ok(expected)
}

fn validate_decompilation(
    status: ExitStatus,
    out_path: &Path,
    expected: &[PathBuf],
) -> Result<(), String> {
    if !status.success() {
        return Err(format!("Vineflower exited with {status}"));
    }
    if expected.is_empty() {
        return Err("no expected sources to validate".into());
    }
    let missing: Vec<_> = expected
        .iter()
        .filter(|path| !fs::metadata(out_path.join(path)).is_ok_and(|m| m.is_file() && m.len() > 0))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "{} of {} expected sources missing or empty; first missing: {:?}",
            missing.len(),
            expected.len(),
            &missing[..missing.len().min(10)]
        ));
    }
    Ok(())
}

pub fn remap_jar_with_srg_mappings(
    config: &Config,
    jar_input_path: &Path,
    jar_output_path: &Path,
    mappings_path: &Path,
) {
    println!("Remapping {jar_input_path:?} with mappings {mappings_path:?}");
    let output = Command::new("java")
        .args(config.java_args.iter().map(String::as_str))
        .args([
            "-jar",
            SPECIALSOURCE_PATH,
            "--in-jar",
            jar_input_path.to_str().unwrap(),
            "--out-jar",
            jar_output_path.to_str().unwrap(),
            "--srg-in",
            mappings_path.to_str().unwrap(),
            // remove the local variable table so their names are generated by our decompiler
            "--kill-lvt",
        ])
        // .stdout(io::stdout())
        // .stderr(io::stderr())
        .output()
        .expect("failed to execute specialsource");
    assert!(
        output.status.success(),
        "SpecialSource failed ({}): {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "decomp-test-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn write(&self, name: &str, contents: &str) {
            let path = self.0.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn status(code: i32) -> ExitStatus {
        Command::new("sh")
            .args(["-c", &format!("exit {code}")])
            .status()
            .unwrap()
    }
    #[test]
    fn empty_source_is_not_a_successful_decompilation() {
        let f = Fixture::new();
        f.write("Broken.java", "");
        assert!(validate_decompilation(status(0), &f.0, &["Broken.java".into()]).is_err());
    }
    #[test]
    fn partial_output_is_rejected_even_after_successful_exit() {
        let f = Fixture::new();
        f.write("net/example/Present.java", "class Present {}");
        let result = validate_decompilation(
            status(0),
            &f.0,
            &[
                "net/example/Present.java".into(),
                "net/example/Missing.java".into(),
            ],
        );
        assert!(result.unwrap_err().contains("Missing.java"));
    }
    #[test]
    fn failed_process_is_rejected_even_with_all_sources() {
        let f = Fixture::new();
        f.write("Present.java", "class Present {}");
        assert!(validate_decompilation(status(1), &f.0, &["Present.java".into()]).is_err());
    }
    #[test]
    fn complete_output_is_accepted() {
        let f = Fixture::new();
        f.write(
            "com/example/Present.java",
            "class Present { class Nested {} }",
        );
        assert!(
            validate_decompilation(status(0), &f.0, &["com/example/Present.java".into()]).is_ok()
        );
    }
    #[test]
    fn inventory_uses_top_level_classes_from_real_jar() {
        let f = Fixture::new();
        let input = f.0.join("input");
        fs::create_dir(&input).unwrap();
        for name in [
            "Present.class",
            "Present$Nested.class",
            "package-info.class",
            "resource.txt",
        ] {
            fs::write(input.join(name), "fixture").unwrap();
        }
        let jar = f.0.join("input.jar");
        assert!(Command::new("jar")
            .arg("cf")
            .arg(&jar)
            .arg("-C")
            .arg(&input)
            .arg(".")
            .status()
            .unwrap()
            .success());
        let mut expected = expected_sources(&jar).unwrap();
        expected.sort();
        assert_eq!(
            expected,
            vec![
                PathBuf::from("Present.java"),
                PathBuf::from("package-info.java")
            ]
        );
    }
    #[test]
    fn invalid_jar_is_rejected() {
        let f = Fixture::new();
        f.write("bad.jar", "not a zip");
        assert!(expected_sources(&f.0.join("bad.jar")).is_err());
    }
    #[test]
    fn incomplete_run_never_returns_to_publishing_caller() {
        let f = Fixture::new();
        f.write("input/net/example/Present.class", "fixture");
        let jar = f.0.join("input.jar");
        assert!(Command::new("jar")
            .arg("cf")
            .arg(&jar)
            .arg("-C")
            .arg(f.0.join("input"))
            .arg(".")
            .status()
            .unwrap()
            .success());
        // java -version exits successfully without invoking Vineflower or emitting any sources.
        let config = Config {
            java_args: vec!["-version".into()],
            vineflower_args: vec![],
        };
        assert!(
            std::panic::catch_unwind(|| decompile_jar(&config, &jar, &f.0.join("out"))).is_err()
        );
    }
}
