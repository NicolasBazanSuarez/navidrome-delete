# Navidrome Delete

A Navidrome WebAssembly plugin that allows you to **physically delete music files from disk by adding tracks to a dedicated Navidrome playlist**.

The intended workflow is simple:

```text
Add track to "Delete from disk"
        ↓
Navidrome Delete detects it
        ↓
Resolves the real physical file path
        ↓
Deletes the audio file
        ↓
Navidrome's library watcher detects the change
        ↓
Scanner.PurgeMissing = always
        ↓
The missing entry is removed from the Navidrome library
```

> [!WARNING]
> **This plugin can permanently delete files from disk.**
>
> Files are not moved to a recycle bin or trash folder.
>
> Always perform the first test with **Dry run enabled** and with only one disposable track in the deletion playlist.

---

## Features

- Delete music files directly from Navidrome.
- Uses a normal Navidrome playlist as a deletion queue.
- Works without modifying the Navidrome web interface.
- Can also be used from Subsonic-compatible clients capable of editing playlists.
- Configurable playlist name.
- Configurable Navidrome user.
- Configurable library.
- Configurable polling interval.
- Safe **Dry run** mode.
- Validates that resolved files belong to the configured library.
- Runs inside Navidrome's WebAssembly plugin sandbox.
- Uses Navidrome's own library permissions.
- Supports automatic cleanup of deleted tracks through `Scanner.PurgeMissing`.

---

## Requirements

Navidrome Delete was developed and tested with:

```text
Navidrome 0.64.2
```

You will need:

- Navidrome with plugin support.
- Navidrome plugins enabled.
- A writable music library.
- A Navidrome user authorized for the plugin.
- The appropriate music library authorized for the plugin.
- **Allow Write Access** enabled for real deletion.
- The `navidrome-delete` player configured with **Report Real Path**.
- `Scanner.PurgeMissing = always` if you want deleted tracks to disappear automatically from Navidrome.

For Docker installations, the music volume must be mounted as `rw`.

---

# Installation

## 1. Download the plugin

Download the latest:

```text
navidrome-delete.ndp
```

from the repository's **Releases** page.

The `.ndp` package contains:

```text
manifest.json
plugin.wasm
```

---

## 2. Copy the plugin to Navidrome

Place `navidrome-delete.ndp` inside Navidrome's plugin directory.

For a typical Docker installation where `/data` is Navidrome's data directory:

```text
/data/plugins/navidrome-delete.ndp
```

On the Docker host this could look like:

```bash
cp navidrome-delete.ndp /path/to/navidrome/data/plugins/
```

For example, if your Navidrome data directory is:

```text
/home/user/navidrome/data
```

the plugin would be stored at:

```text
/home/user/navidrome/data/plugins/navidrome-delete.ndp
```

---

# Docker configuration

## Enable plugins

Navidrome plugins must be enabled.

For Docker Compose:

```yaml
services:
  navidrome:
    environment:
      ND_PLUGINS_ENABLED: "true"
```

Optionally, plugin auto-reload and debug logging can also be enabled:

```yaml
services:
  navidrome:
    environment:
      ND_PLUGINS_ENABLED: "true"
      ND_PLUGINS_AUTO_RELOAD: "true"
      ND_PLUGINS_LOG_LEVEL: "debug"
```

---

## Make the music library writable

The plugin cannot delete files if the music library is mounted read-only.

### Incorrect

```yaml
volumes:
  - /path/to/Music:/music:ro
```

### Correct

```yaml
volumes:
  - /path/to/Music:/music:rw
```

You can also simply omit the access mode because Docker mounts are writable by default:

```yaml
volumes:
  - /path/to/Music:/music
```

Explicit `:rw` is recommended because it makes the requirement obvious.

---

## Recommended Docker Compose configuration

A typical configuration could look like:

```yaml
services:
  navidrome:
    image: deluan/navidrome:latest
    container_name: navidrome

    environment:
      ND_PLUGINS_ENABLED: "true"
      ND_PLUGINS_AUTO_RELOAD: "true"
      ND_PLUGINS_LOG_LEVEL: "debug"
      ND_SCANNER_PURGEMISSING: "always"

    volumes:
      - /path/to/Music:/music:rw
      - /path/to/navidrome/data:/data

    ports:
      - "4533:4533"
```

Adapt the paths and other settings to your existing Navidrome installation.

---

## Apply Docker configuration changes

If you change volumes or environment variables, recreate the Navidrome container:

```bash
docker compose up -d
```

This is different from simply restarting the existing container because changes to the Compose configuration may require the container to be recreated.

---

## Verify write access

You can check whether `/music` is writable from Docker with:

```bash
docker inspect navidrome \
  --format '{{range .Mounts}}{{if eq .Destination "/music"}}RW={{.RW}} Source={{.Source}}{{end}}{{end}}'
```

Expected output:

```text
RW=true Source=/path/to/Music
```

If you see:

```text
RW=false
```

the plugin will not be able to physically delete files.

---

# Navidrome plugin configuration

Open the Navidrome administration interface and go to:

```text
Settings
→ Plugins
→ Navidrome Delete
```

The plugin exposes the following settings:

| Setting | Example | Description |
|---|---|---|
| Navidrome username | `myuser` | User whose playlist is monitored |
| Delete playlist | `Delete from disk` | Playlist used as the deletion queue |
| Library ID | `1` | Navidrome library containing the files |
| Check interval | `@every 30s` | How often the playlist is checked |
| Dry run | `true` | Simulates deletion without touching files |

You must also authorize:

- the Navidrome user configured above;
- the music library that contains the files.

For real file deletion you must additionally enable:

```text
Allow Write Access
```

> [!IMPORTANT]
> Write access should only be enabled after confirming that Dry run resolves the correct files.

---

# Create the deletion playlist

Create a normal Navidrome playlist named:

```text
Delete from disk
```

The name can be changed through the plugin settings if you prefer another name.

Do **not** use an existing playlist containing music that you intend to keep.

Every track found in this playlist is considered a deletion candidate.

---

# Report Real Path

This configuration is required.

Navidrome's Subsonic API may normally report a virtual path generated from metadata such as:

```text
Artist/Album/Track.opus
```

That path does not necessarily correspond to the actual location of the file on disk.

Navidrome Delete needs the real physical path.

---

## Enable the plugin player

Enable Navidrome Delete at least once.

When the plugin performs its first internal Subsonic request, Navidrome should create a player called:

```text
navidrome-delete
```

Go to:

```text
Settings
→ Players
```

Open:

```text
navidrome-delete
```

Enable:

```text
Report Real Path
```

and save the player configuration.

---

## Why this is necessary

Without **Report Real Path**, Navidrome may return something similar to:

```text
Artist/Album/Track.opus
```

even if the actual file is:

```text
/music/Track.opus
```

With Report Real Path enabled, the plugin can resolve it to its sandbox library path:

```text
/libraries/1/Track.opus
```

This allows the plugin to verify that the correct physical file exists before attempting deletion.

---

# First safe test

Before allowing real deletion, keep:

```text
Dry run = true
```

Add **one disposable track only** to:

```text
Delete from disk
```

Then inspect the Navidrome logs:

```bash
docker logs -f navidrome 2>&1 | grep --line-buffered navidrome-delete
```

A successful test should look similar to:

```text
[navidrome-delete] Found 1 candidate(s) in 'Delete from disk'
[navidrome-delete] DRY RUN | Artist - Track | /libraries/1/Track.opus | exists=true
```

The most important part is:

```text
exists=true
```

This confirms that the playlist entry has been resolved to an existing physical file.

If it says:

```text
exists=false
```

do **not** disable Dry run.

Check the **Report Real Path** configuration first.

---

# Dry run

Dry run is the plugin's safety mode.

When:

```text
Dry run = true
```

Navidrome Delete:

1. Reads the configured playlist.
2. Finds the tracks inside it.
3. Requests their real paths from Navidrome.
4. Validates that the paths belong to the configured library.
5. Checks whether the files exist.
6. Logs what would be deleted.
7. Does **not** delete anything.

Example:

```text
DRY RUN | Artist - Track | /libraries/1/Track.opus | exists=true
```

Dry run is strongly recommended when:

- installing the plugin for the first time;
- changing music library paths;
- changing Docker mounts;
- moving Navidrome to another machine;
- changing the configured library;
- upgrading Navidrome;
- upgrading Navidrome Delete;
- testing changes to the plugin.

---

# Enable real deletion

Only continue after Dry run successfully reports:

```text
exists=true
```

for the intended file.

Make sure all of the following are true:

```text
Dry run = false
+
Music volume mounted as RW
+
Correct library authorized
+
Correct user authorized
+
Allow Write Access enabled
+
Report Real Path enabled
```

Then set:

```text
Dry run = false
```

and save the plugin configuration.

When the plugin next processes the playlist, a successful deletion appears as:

```text
[navidrome-delete] DELETED | Artist - Track | /libraries/1/Track.opus
```

The audio file has now been physically deleted.

> [!CAUTION]
> The deletion is permanent.
>
> Navidrome Delete does not move files to a recycle bin or trash directory.

---

# Automatically remove deleted tracks from Navidrome

Deleting the physical audio file does not necessarily remove its database entry immediately.

By default, Navidrome can keep these tracks as **Missing Files**.

For a deletion workflow, it is usually preferable to automatically purge those records.

Configure:

```yaml
environment:
  ND_SCANNER_PURGEMISSING: "always"
```

Then apply the Compose change:

```bash
docker compose up -d
```

Verify the setting:

```bash
docker exec navidrome sh -c 'echo $ND_SCANNER_PURGEMISSING'
```

Expected output:

```text
always
```

The resulting workflow is:

```text
Track added to deletion playlist
        ↓
Navidrome Delete resolves the physical path
        ↓
File is deleted
        ↓
Navidrome watcher notices the filesystem change
        ↓
Scanner detects the missing media file
        ↓
PurgeMissing removes the missing database entry
```

> [!NOTE]
> Purging a missing track also means that Navidrome no longer keeps the normal library record associated with that missing file.

---

# Normal usage

Once installation and testing are complete, normal usage is very simple.

### Delete a track

Add it to:

```text
Delete from disk
```

The plugin checks the playlist periodically.

With the default configuration:

```text
@every 30s
```

it will process the queue approximately every 30 seconds.

No manual filesystem access is required.

---

# Using Subsonic clients

Because the deletion queue is a normal Navidrome playlist, it can also be manipulated by Subsonic-compatible applications that support playlist editing.

For example:

```text
Client
   ↓
Add track to "Delete from disk"
   ↓
Navidrome playlist
   ↓
Navidrome Delete
   ↓
Physical file deletion
```

The client itself does not need direct filesystem access.

---

# Safety model

Several independent protections must all allow deletion.

```text
Track exists in configured playlist
        +
Correct Navidrome user
        +
Correct Navidrome library
        +
Path belongs to that library
        +
Dry run disabled
        +
Plugin Write Access enabled
        +
Underlying filesystem writable
        =
Physical deletion
```

This is intentional.

Giving the plugin Write Access alone is not enough if Docker exposes the music directory as read-only.

Similarly, mounting the library as writable does not allow the plugin to modify it unless Navidrome grants plugin Write Access.

---

# Path safety

The plugin does not blindly delete arbitrary paths received from the API.

Before operating on a file it verifies that the path belongs to the selected Navidrome library.

Paths that escape or do not belong to the configured library are rejected.

Example log:

```text
[navidrome-delete] REJECTED path outside library: ...
```

The plugin never intentionally deletes directories.

---

# Updating Navidrome Delete

For the safest update process, first disable the plugin:

```bash
docker exec navidrome navidrome plugin disable navidrome-delete
```

Replace the old:

```text
navidrome-delete.ndp
```

with the new release inside the plugin directory.

Then restart the main Navidrome process:

```bash
docker restart navidrome
```

Finally, enable the plugin again.

---

## Why restart Navidrome after an update?

Navidrome may detect a new `.ndp` package during a plugin rescan while the main server process still has the previous WebAssembly module loaded.

A restart guarantees that the new:

```text
plugin.wasm
```

is loaded.

Therefore, after replacing the `.ndp`, this is recommended:

```bash
docker restart navidrome
```

---

# When do I need to restart Navidrome?

| Change | Required action |
|---|---|
| Add tracks to `Delete from disk` | Nothing |
| Remove tracks from `Delete from disk` | Nothing |
| Change plugin configuration | Save plugin settings |
| Enable or disable Dry run | Save plugin settings |
| Change polling interval | Save plugin settings |
| Enable Report Real Path | Save player settings |
| Enable plugin Write Access | Restart recommended before real deletion |
| Change `/music:ro` to `/music:rw` | `docker compose up -d` |
| Change Navidrome environment variables | `docker compose up -d` |
| Add `ND_SCANNER_PURGEMISSING` | `docker compose up -d` |
| Install a new `.ndp` | Plugin rescan or restart |
| Replace an existing `.ndp` | `docker restart navidrome` recommended |
| Plugin appears to run old code | `docker restart navidrome` |

---

# Troubleshooting

## `exists=false`

Example:

```text
DRY RUN | Artist - Track | ... | exists=false
```

Check:

```text
Settings
→ Players
→ navidrome-delete
→ Report Real Path
```

It must be enabled.

---

## `DELETE FAILED`

If the log contains:

```text
DELETE FAILED
```

check the two independent write permission layers.

### Docker filesystem permission

Run:

```bash
docker inspect navidrome \
  --format '{{range .Mounts}}{{if eq .Destination "/music"}}RW={{.RW}}{{end}}{{end}}'
```

Expected:

```text
RW=true
```

### Navidrome plugin permission

Open:

```text
Settings
→ Plugins
→ Navidrome Delete
```

Make sure:

```text
Allow Write Access = enabled
```

and make sure the correct library is authorized.

---

## `REJECTED path outside library`

Example:

```text
[navidrome-delete] REJECTED path outside library: ...
```

The path reported by Navidrome does not belong to the configured library.

The plugin refuses to delete it.

Check:

- Library ID.
- Authorized library.
- Report Real Path.
- Navidrome music paths.

---

## File is deleted but remains in Missing Files

Configure:

```text
ND_SCANNER_PURGEMISSING=always
```

For Docker Compose:

```yaml
environment:
  ND_SCANNER_PURGEMISSING: "always"
```

Then:

```bash
docker compose up -d
```

---

## Plugin still behaves like the previous version

Restart Navidrome:

```bash
docker restart navidrome
```

A plugin rescan alone may not replace the WebAssembly module already loaded by the running server process.

---

## Plugin does not appear

Verify that plugins are enabled:

```yaml
environment:
  ND_PLUGINS_ENABLED: "true"
```

Verify that the package exists in the configured plugin directory:

```text
/data/plugins/navidrome-delete.ndp
```

Then restart Navidrome:

```bash
docker restart navidrome
```

---

# Building from source

## Requirements

You need:

- Docker
- Git
- `zip`

Clone the repository:

```bash
git clone https://github.com/OWNER/navidrome-delete.git
cd navidrome-delete
```

Run:

```bash
./scripts/build.sh
```

The script:

1. Downloads the required Navidrome source/PDK into `.deps/`.
2. Installs the `wasm32-wasip1` Rust target inside the build container.
3. Compiles Navidrome Delete as WebAssembly.
4. Renames the resulting WASM module to `plugin.wasm`.
5. Packages `manifest.json` and `plugin.wasm` into an `.ndp`.

The resulting package is:

```text
dist/navidrome-delete.ndp
```

You can inspect it with:

```bash
unzip -l dist/navidrome-delete.ndp
```

It should contain:

```text
manifest.json
plugin.wasm
```

---

# Project structure

```text
navidrome-delete/
├── .github/
│   └── workflows/
│       └── build.yml
├── scripts/
│   └── build.sh
├── src/
│   └── lib.rs
├── .gitignore
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── manifest.json
└── README.md
```

Generated files are intentionally excluded from Git:

```text
.deps/
target/
dist/
*.ndp
```

---

# GitHub Releases

The repository includes a GitHub Actions workflow.

Normal pushes and pull requests compile the plugin to verify that the project still builds.

Tags matching:

```text
v*
```

also create a GitHub Release containing:

```text
navidrome-delete.ndp
```

For example:

```bash
git tag v0.1.0
git push origin v0.1.0
```

will trigger the release workflow for:

```text
v0.1.0
```

---

# Plugin package format

Navidrome plugins use the `.ndp` format.

An `.ndp` file is a ZIP archive containing:

```text
manifest.json
plugin.wasm
```

The plugin ID comes from the `.ndp` filename.

Therefore:

```text
navidrome-delete.ndp
```

is loaded with the plugin ID:

```text
navidrome-delete
```

Keeping this filename stable is recommended because Navidrome stores plugin configuration against the plugin ID.

---

# Development

The plugin is written in Rust and compiled to:

```text
wasm32-wasip1
```

The current build is based on the Navidrome plugin SDK corresponding to:

```text
Navidrome v0.64.2
```

The SDK source is downloaded automatically by:

```text
scripts/build.sh
```

into:

```text
.deps/navidrome
```

This directory is ignored by Git.

---

# Disclaimer

This project performs destructive filesystem operations.

Although multiple safety checks are implemented, you are responsible for:

- maintaining backups;
- testing configuration before enabling deletion;
- ensuring the correct library is authorized;
- ensuring the deletion playlist does not contain tracks you want to keep.

Use **Dry run** whenever changing your installation or configuration.

---

# License

Navidrome Delete is released under the MIT License.

See:

```text
LICENSE
```

for the full license text.