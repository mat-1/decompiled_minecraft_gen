use std::path::PathBuf;

use crate::{config::Config, decomp, download, git};

const FIRST_VERSION_WITH_MAPPINGS: &str = "1.14.4";
const FIRST_VERSION_WITHOUT_OBF: &str = "26.1-snapshot-1";
const BRANCH_NAME: &str = "mojmap";

pub fn generate(config: &Config) {
    git::checkout_branch(BRANCH_NAME);

    let manifest = download::mojang::get_version_manifest();

    let mut latest_version_decompiled = None;
    for commit_message in git::commit_messages() {
        if let Some(version_id) = commit_message.strip_prefix("Update to ") {
            latest_version_decompiled = Some(version_id.to_owned());
            break;
        }
    }

    let mut has_obfuscation = true;

    let mut versions_to_decompile = Vec::new();
    if let Some(latest_version_decompiled) = latest_version_decompiled {
        println!("Latest version decompiled: {latest_version_decompiled}");

        let mut it = manifest.versions.iter().rev();
        for version in &mut it {
            if version.id == FIRST_VERSION_WITHOUT_OBF {
                has_obfuscation = false;
            }
            if version.id == latest_version_decompiled {
                break;
            }
        }
        versions_to_decompile.extend(it);
    } else {
        println!("No versions decompiled yet, starting at {FIRST_VERSION_WITH_MAPPINGS}");

        versions_to_decompile.extend(
            manifest
                .versions
                .iter()
                .rev()
                .skip_while(|v| v.id != FIRST_VERSION_WITH_MAPPINGS),
        )
    };

    println!("versions_to_decompile: {versions_to_decompile:?}");

    for version in versions_to_decompile {
        let version_id = version.id.to_owned();
        println!("Decompiling version {version_id}");

        if version_id == FIRST_VERSION_WITHOUT_OBF {
            has_obfuscation = false;
        }

        let jar_path = download::mojang::get_version_jar(&version_id);
        let mut remapped_jar_path = jar_path.with_file_name(format!(
            "{}-remapped.jar",
            jar_path
                .file_stem()
                .unwrap_or_else(|| panic!("jar path {jar_path:?} must have a file extension"))
                .to_str()
                .unwrap()
        ));

        let out_path = PathBuf::from("./tmp/out");

        if has_obfuscation {
            let Some(mappings_path) = download::mojang::get_version_mappings(&version_id) else {
                println!("{version_id} has no mappings");
                continue;
            };

            decomp::remap_jar_with_srg_mappings(
                config,
                &jar_path,
                &remapped_jar_path,
                &mappings_path,
            );
        } else {
            remapped_jar_path = jar_path;
        }
        decomp::decompile_jar(config, &remapped_jar_path, &out_path);

        git::move_decomp_output_into_repo(&out_path, &["assets", "data", "META-INF"]);

        git::commit(
            &format!("Update to {version_id}"),
            "Mojang",
            "support@mojang.com",
            version.release_time,
        );
    }
}
