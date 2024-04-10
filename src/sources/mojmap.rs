use std::path::PathBuf;

use crate::{config::Config, decomp, download, git};

const FIRST_VERSION_WITH_MAPPINGS: &str = "19w36a";
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

    let versions_to_decompile = if let Some(latest_version_decompiled) = latest_version_decompiled {
        println!("Latest version decompiled: {latest_version_decompiled}");

        manifest
            .versions
            .iter()
            .rev()
            .skip_while(|v| v.id != latest_version_decompiled)
            .skip(1)
            .collect::<Vec<_>>()
    } else {
        println!("No versions decompiled yet, starting at {FIRST_VERSION_WITH_MAPPINGS}");

        manifest
            .versions
            .iter()
            .rev()
            .skip_while(|v| v.id != FIRST_VERSION_WITH_MAPPINGS)
            .collect::<Vec<_>>()
    };

    for version in versions_to_decompile {
        let version_id = version.id.to_owned();
        println!("Decompiling version {version_id}");

        let jar_path = download::mojang::get_version_jar(&version_id);
        let remapped_jar_path = jar_path.with_file_name(format!(
            "{}-remapped.jar",
            jar_path
                .file_stem()
                .unwrap_or_else(|| panic!("jar path {jar_path:?} must have a file extension"))
                .to_str()
                .unwrap()
        ));
        let mappings_path = download::mojang::get_version_mappings(&version_id);

        let out_path = PathBuf::from("./tmp/out");

        decomp::remap_jar_with_srg_mappings(config, &jar_path, &remapped_jar_path, &mappings_path);
        decomp::decompile_jar(config, &remapped_jar_path, &out_path);

        git::move_decomp_output_into_repo(&out_path, &["assets", "data"]);

        git::commit(
            &format!("Update to {version_id}"),
            "Mojang",
            "support@mojang.com",
            version.release_time,
        );
    }
}
