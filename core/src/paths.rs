use color_eyre::eyre::bail;
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
use walkdir::WalkDir;

pub const GAME_BACKUP_EXTENSIONS: [&str; 3] = ["nsp", "xci", "nsz"];

pub(crate) fn is_game_backup(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| GAME_BACKUP_EXTENSIONS.contains(&ext))
}

pub(crate) fn read_game_paths(
    game_backup_path: &Path,
    recurse: bool,
    cancel: Option<&AtomicBool>,
) -> color_eyre::Result<Vec<String>> {
    if !game_backup_path.exists() {
        bail!("Given path ({}) does not exist", game_backup_path.display())
    }

    let mut game_paths = Vec::new();

    if game_backup_path.is_dir() {
        for entry_result in
            WalkDir::new(game_backup_path).max_depth(if recurse { usize::MAX } else { 1 })
        {
            if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) {
                return Ok(game_paths);
            }
            let Ok(entry) = entry_result else {
                continue;
            };
            let path = entry.path();
            if !is_game_backup(path) {
                continue;
            }
            let Some(path_str) = path.to_str() else {
                continue;
            };
            game_paths.push(path_str.to_string());
        }
    } else if is_game_backup(game_backup_path)
        && let Some(path_str) = game_backup_path.to_str()
    {
        if recurse {
            eprintln!("Warning: recurse has no effect when given path is a file, ignoring...");
        }
        game_paths.push(path_str.to_string());
    } else {
        bail!(
            "Given path ({}) is neither a directory nor a valid game backup file",
            game_backup_path.display()
        )
    };

    if game_paths.is_empty() {
        bail!(
            "No game backup files found in given directory ({})",
            game_backup_path.display()
        )
    }

    Ok(game_paths)
}
