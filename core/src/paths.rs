use color_eyre::eyre::bail;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub const GAME_BACKUP_EXTENSIONS: [&str; 3] = ["nsp", "xci", "nsz"];
pub const RCM_PAYLOAD_EXTENSIONS: [&str; 1] = ["bin"];

fn is_game_backup(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| GAME_BACKUP_EXTENSIONS.contains(&ext))
}

pub fn read_game_paths(game_backup_path: &Path, recurse: bool) -> color_eyre::Result<Vec<PathBuf>> {
    if !game_backup_path.exists() {
        bail!("Given path ({}) does not exist", game_backup_path.display())
    }

    let mut game_paths = Vec::new();

    if game_backup_path.is_dir() {
        for entry_result in
            WalkDir::new(game_backup_path).max_depth(if recurse { usize::MAX } else { 1 })
        {
            let Ok(entry) = entry_result else {
                continue;
            };
            let path = entry.path();
            if !is_game_backup(path) {
                continue;
            }
            game_paths.push(path.to_path_buf());
        }
    } else if is_game_backup(game_backup_path) {
        if recurse {
            eprintln!("Warning: recurse has no effect when given path is a file, ignoring...");
        }
        game_paths.push(game_backup_path.to_path_buf());
    } else {
        bail!(
            "Given path ({}) is neither a directory nor a valid game backup file",
            game_backup_path.display()
        )
    }

    if game_paths.is_empty() {
        bail!(
            "No game backup files found in given directory ({})\nDid you forget 'recurse'?",
            game_backup_path.display()
        )
    }

    Ok(game_paths)
}
