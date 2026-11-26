# decompiled-minecraft gen

Generate a Git repo with commits and branches for the decompiled source code of different Minecraft versions and mappings.

Currently the only branches generated are mojmap and authlib.

## Usage

decompiled-minecraft-gen will create the full repository of decompiled Minecraft versions at `./repo` on run (with `cargo r -r`).
The first time you run it, it'll take a few hours.

If the repository already exists at that location, it'll update it to the latest snapshot (or if it's up-to-date, it won't do anything).
It'll run `git push --all` after it updates.

A recent JDK is required, with both `java` and `jar` on `PATH`.

Before replacing repository contents, the generator requires a successful Vineflower exit
and a nonempty `.java` file for every top-level class in the input JAR. Inner/anonymous
classes (names containing `$`), module descriptors, and `META-INF` versioned classes are
excluded from this file-presence check. This detects missing files, not incomplete method
bodies inside otherwise present sources. Failed attempts report missing paths and retry;
after three failures the generator stops before moving output or committing that version.
SpecialSource failures also stop the run.

This prevents new incomplete updates; it does not repair an existing incomplete commit.
The normal update loop skips versions already recorded in history.
