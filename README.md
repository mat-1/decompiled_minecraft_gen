# decompiled-minecraft gen

Generate a Git repo with commits and branches for the decompiled source code of different Minecraft versions and mappings.

Currently the only branches generated are mojmap and authlib.

## Usage

decompiled-minecraft-gen will create the full repository of decompiled Minecraft versions at `./repo` on run (with `cargo r -r`).
The first time you run it, it'll take a few hours.

If the repository already exists at that location, it'll update it to the latest snapshot (or if it's up-to-date, it won't do anything).
It'll run `git push --all` after it updates.

Having a recent version of Java installed is required.
