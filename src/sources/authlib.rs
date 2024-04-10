use std::{collections::HashSet, path::PathBuf};

use crate::{config::Config, decomp, download, git};

const FIRST_VERSION_WITH_AUTHLIB: &str = "13w39a";
const BRANCH_NAME: &str = "authlib";

pub fn generate(config: &Config) {
    git::checkout_branch(BRANCH_NAME);

    let manifest = download::mojang::get_version_manifest();

    let mut authlib_versions_decompiled = HashSet::new();
    let mut latest_mc_version_that_updated_authlib = None;

    for commit_message in git::commit_messages() {
        // authlib commit messages look like "Update to 1.1 (13w39a)"
        if let Some(authlib_and_mc_version_ids) = commit_message.strip_prefix("Update to ") {
            let mut authlib_and_mc_version_ids_split =
                authlib_and_mc_version_ids.split_whitespace();
            let authlib_version = authlib_and_mc_version_ids_split
                .next()
                .expect("commit message should have authlib version");
            let mc_version = authlib_and_mc_version_ids_split
                .next()
                .expect("authlib commit message should have Minecraft version")
                .trim_matches(|c| c == '(' || c == ')');

            authlib_versions_decompiled.insert(authlib_version.to_owned());
            if latest_mc_version_that_updated_authlib.is_none() {
                latest_mc_version_that_updated_authlib = Some(mc_version.to_owned());
            }
        }
    }

    let mc_versions_to_check = if let Some(latest_mc_version_decompiled) =
        latest_mc_version_that_updated_authlib
    {
        println!("Latest Minecraft version that updated authlib that we decompiled: {latest_mc_version_decompiled}");

        manifest
            .versions
            .iter()
            .rev()
            .skip_while(|v| v.id != latest_mc_version_decompiled)
            .skip(1)
            .collect::<Vec<_>>()
    } else {
        println!("No authlib versions decompiled yet, starting at {FIRST_VERSION_WITH_AUTHLIB}");

        manifest
            .versions
            .iter()
            .rev()
            .skip_while(|v| v.id != FIRST_VERSION_WITH_AUTHLIB)
            .collect::<Vec<_>>()
    };

    for version in mc_versions_to_check {
        let mc_version_id = version.id.to_owned();
        println!("Checking version {mc_version_id}");

        let authlib_version_id =
            download::authlib::get_authlib_version_for_minecraft_version(&mc_version_id);
        if authlib_versions_decompiled.contains(&authlib_version_id) {
            continue;
        }
        authlib_versions_decompiled.insert(authlib_version_id.clone());

        let jar_path = download::authlib::get_version_jar(&authlib_version_id);

        let out_path = PathBuf::from("./tmp/out");

        decomp::decompile_jar(config, &jar_path, &out_path);

        git::move_decomp_output_into_repo(&out_path, &[]);

        git::commit(
            &format!("Update to {authlib_version_id} ({mc_version_id})"),
            "Mojang",
            "support@mojang.com",
            version.release_time,
        );
    }
}
