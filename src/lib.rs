use extism_pdk::{error, info, warn};
use nd_pdk::host::{config, library, scheduler, subsonicapi};
use nd_pdk::lifecycle::{Error as LifecycleError, InitProvider};
use nd_pdk::scheduler::{
    CallbackProvider,
    Error as SchedulerError,
    SchedulerCallbackRequest,
};
use serde_json::Value;
use std::fs;
use std::path::{Component, Path, PathBuf};

nd_pdk::register_lifecycle_init!(NavidromeDelete);
nd_pdk::register_scheduler_callback!(NavidromeDelete);

#[derive(Default)]
struct NavidromeDelete;

fn config_string(key: &str, default: &str) -> String {
    config::get(key)
        .ok()
        .flatten()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn config_i32(key: &str, default: i32) -> i32 {
    config::get_int(key)
        .ok()
        .flatten()
        .and_then(|v| i32::try_from(v).ok())
        .unwrap_or(default)
}

fn config_bool(key: &str, default: bool) -> bool {
    config::get(key)
        .ok()
        .flatten()
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(default)
}

fn safe_relative_path(path: &Path) -> Option<PathBuf> {
    let mut result = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Normal(part) => result.push(part),
            Component::CurDir => {}
            _ => return None,
        }
    }

    if result.as_os_str().is_empty() {
        return None;
    }

    Some(result)
}

fn resolve_library_path(
    library_path: &str,
    mount_point: &str,
    media_path: &str,
) -> Option<PathBuf> {
    let media = Path::new(media_path);
    let library_root = Path::new(library_path);

    let relative = if media.is_absolute() {
        media.strip_prefix(library_root).ok()?
    } else {
        media
    };

    let relative = safe_relative_path(relative)?;

    Some(Path::new(mount_point).join(relative))
}

fn process_playlist() -> Result<(), String> {
    let username = config_string("username", "Nikete9000");
    let playlist_name = config_string("playlist_name", "Delete from disk");
    let library_id = config_i32("library_id", 1);
    let dry_run = config_bool("dry_run", true);

    let library = library::get_library(library_id)
        .map_err(|e| format!("Could not read library: {}", e))?
        .ok_or_else(|| format!("Library {} does not exist", library_id))?;

    if library.mount_point.is_empty() {
        return Err(
            "Library filesystem access is not enabled for this plugin".to_string()
        );
    }

    let playlists_uri = format!(
        "getPlaylists?u={}",
        urlencoding::encode(&username)
    );

    let playlists_json = subsonicapi::call(&playlists_uri)
        .map_err(|e| format!("getPlaylists failed: {}", e))?;

    let root: Value = serde_json::from_str(&playlists_json)
        .map_err(|e| format!("Could not parse getPlaylists response: {}", e))?;

    let playlists = root
        .get("subsonic-response")
        .and_then(|v| v.get("playlists"))
        .and_then(|v| v.get("playlist"))
        .and_then(Value::as_array);

    let Some(playlists) = playlists else {
        info!(
            "[navidrome-delete] No playlists found for user {}",
            username
        );
        return Ok(());
    };

    let target_playlist = playlists.iter().find(|playlist| {
        playlist
            .get("name")
            .and_then(Value::as_str)
            .map(|name| name == playlist_name)
            .unwrap_or(false)
    });

    let Some(target_playlist) = target_playlist else {
        info!(
            "[navidrome-delete] Playlist '{}' does not exist yet",
            playlist_name
        );
        return Ok(());
    };

    let playlist_id = target_playlist
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| "Playlist has no ID".to_string())?;

    let playlist_uri = format!(
        "getPlaylist?id={}&u={}",
        urlencoding::encode(playlist_id),
        urlencoding::encode(&username)
    );

    let playlist_json = subsonicapi::call(&playlist_uri)
        .map_err(|e| format!("getPlaylist failed: {}", e))?;

    let detail: Value = serde_json::from_str(&playlist_json)
        .map_err(|e| format!("Could not parse playlist response: {}", e))?;

    let entries = detail
        .get("subsonic-response")
        .and_then(|v| v.get("playlist"))
        .and_then(|v| v.get("entry"))
        .and_then(Value::as_array);

    let Some(entries) = entries else {
        info!(
            "[navidrome-delete] Playlist '{}' is empty",
            playlist_name
        );
        return Ok(());
    };

    info!(
        "[navidrome-delete] Found {} candidate(s) in '{}'",
        entries.len(),
        playlist_name
    );

    for (index, entry) in entries.iter().enumerate() {
        let title = entry
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("(unknown title)");

        let artist = entry
            .get("artist")
            .and_then(Value::as_str)
            .unwrap_or("(unknown artist)");

        let Some(media_path) = entry.get("path").and_then(Value::as_str) else {
            warn!(
                "[navidrome-delete] Entry {} has no filesystem path: {} - {}",
                index,
                artist,
                title
            );
            continue;
        };

        let Some(physical_path) = resolve_library_path(
            &library.path,
            &library.mount_point,
            media_path,
        ) else {
            warn!(
                "[navidrome-delete] REJECTED path outside library: {}",
                media_path
            );
            continue;
        };

        if !physical_path.is_file() {
            warn!(
                "[navidrome-delete] File does not exist: {}",
                physical_path.display()
            );
            continue;
        }

        if dry_run {
            info!(
                "[navidrome-delete] DRY RUN | {} - {} | {} | exists=true",
                artist,
                title,
                physical_path.display()
            );
            continue;
        }

        match fs::remove_file(&physical_path) {
            Ok(_) => {
                info!(
                    "[navidrome-delete] DELETED | {} - {} | {}",
                    artist,
                    title,
                    physical_path.display()
                );
            }
            Err(e) => {
                error!(
                    "[navidrome-delete] DELETE FAILED | {} | {}",
                    physical_path.display(),
                    e
                );
            }
        }
    }

    Ok(())
}

impl InitProvider for NavidromeDelete {
    fn on_init(&self) -> Result<(), LifecycleError> {
        let cron = config_string("cron", "@every 30s");

        info!(
            "[navidrome-delete] Initializing in DRY-RUN mode"
        );

        match scheduler::schedule_recurring(
            &cron,
            "poll-delete-playlist",
            "navidrome-delete-poll",
        ) {
            Ok(id) => {
                info!(
                    "[navidrome-delete] Scheduled watcher: {} ({})",
                    id,
                    cron
                );
            }
            Err(e) => {
                warn!(
                    "[navidrome-delete] Could not register schedule: {}",
                    e
                );
            }
        }

        if let Err(e) = process_playlist() {
            error!(
                "[navidrome-delete] Initial check failed: {}",
                e
            );
        }

        Ok(())
    }
}

impl CallbackProvider for NavidromeDelete {
    fn on_callback(
        &self,
        req: SchedulerCallbackRequest,
    ) -> Result<(), SchedulerError> {
        if req.payload == "poll-delete-playlist" {
            if let Err(e) = process_playlist() {
                error!(
                    "[navidrome-delete] Playlist check failed: {}",
                    e
                );
            }
        }

        Ok(())
    }
}
