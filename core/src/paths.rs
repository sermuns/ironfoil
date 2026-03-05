use color_eyre::eyre::bail;
use std::path::Path;
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
) -> color_eyre::Result<Vec<String>> {
    if !game_backup_path.exists() {
        bail!("Given path ({}) does not exist", game_backup_path.display())
    }

    let game_paths: Vec<_> = if game_backup_path.is_dir() {
        WalkDir::new(game_backup_path)
            .max_depth(if recurse { usize::MAX } else { 1 })
            .into_iter()
            .filter_map(|entry_result| {
                let entry = entry_result.ok()?;
                let path = entry.path();
                is_game_backup(path).then_some(path.to_str()?.to_string())
            })
            .collect()
    } else if is_game_backup(game_backup_path)
        && let Some(path_str) = game_backup_path.to_str()
    {
        if recurse {
            eprintln!("Warning: recurse has no effect when given path is a file, ignoring...");
        }
        vec![path_str.to_string()]
    } else {
        bail!(
            "Given path ({}) is not a directory or a valid game backup file",
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
