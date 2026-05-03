use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

pub type Cell = (usize, usize);

pub const DEFAULT_SAVE_PATH: &str = "saves.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Saves {
    pub seeds: Vec<Vec<Cell>>,
}

#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::Io(err) => write!(f, "I/O error while saving seed: {err}"),
            SaveError::Json(err) => write!(f, "JSON error while saving seed: {err}"),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<std::io::Error> for SaveError {
    fn from(err: std::io::Error) -> Self {
        SaveError::Io(err)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(err: serde_json::Error) -> Self {
        SaveError::Json(err)
    }
}

pub fn load<P: AsRef<Path>>(path: P) -> Result<Saves, SaveError> {
    let path = path.as_ref();
    match File::open(path) {
        Ok(file) => Ok(serde_json::from_reader(BufReader::new(file))?),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(Saves::default()),
        Err(err) => Err(err.into()),
    }
}

pub fn append_seed<P: AsRef<Path>>(path: P, seed: Vec<Cell>) -> Result<usize, SaveError> {
    let path = path.as_ref();
    let mut saves = load(path)?;
    saves.seeds.push(seed);
    let slot = saves.seeds.len();
    write(path, &saves)?;
    Ok(slot)
}

pub fn delete_seed<P: AsRef<Path>>(path: P, index: usize) -> Result<Option<Vec<Cell>>, SaveError> {
    let path = path.as_ref();
    let mut saves = load(path)?;
    if index >= saves.seeds.len() {
        return Ok(None);
    }

    let removed = saves.seeds.remove(index);
    write(path, &saves)?;
    Ok(Some(removed))
}

fn write(path: &Path, saves: &Saves) -> Result<(), SaveError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    let temp_path = temp_path(path);
    let file = File::create(&temp_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, saves)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    fs::rename(temp_path, path)?;
    Ok(())
}

fn temp_path(path: &Path) -> PathBuf {
    match path.extension() {
        Some(ext) => {
            let mut ext = ext.to_os_string();
            ext.push(".tmp");
            path.with_extension(ext)
        }
        None => path.with_extension("tmp"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn load_returns_default_when_file_is_missing() {
        let path = unique_path("missing");
        let saves = load(&path).expect("missing save file should return default");
        assert_eq!(saves, Saves::default());
    }

    #[test]
    fn append_seed_preserves_existing_seeds() {
        let path = unique_path("append");

        let first_slot = append_seed(&path, vec![(1, 2), (3, 4)]).expect("first save should work");
        let second_slot = append_seed(&path, vec![(5, 6)]).expect("second save should work");

        assert_eq!(first_slot, 1);
        assert_eq!(second_slot, 2);
        assert_eq!(
            load(&path).expect("save file should be readable"),
            Saves {
                seeds: vec![vec![(1, 2), (3, 4)], vec![(5, 6)]],
            }
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn delete_seed_removes_selected_seed_and_keeps_order() {
        let path = unique_path("delete");
        append_seed(&path, vec![(1, 2)]).expect("first save should work");
        append_seed(&path, vec![(3, 4)]).expect("second save should work");
        append_seed(&path, vec![(5, 6)]).expect("third save should work");

        let removed = delete_seed(&path, 1)
            .expect("delete should succeed")
            .expect("second seed should exist");

        assert_eq!(removed, vec![(3, 4)]);
        assert_eq!(
            load(&path).expect("save file should be readable"),
            Saves {
                seeds: vec![vec![(1, 2)], vec![(5, 6)]],
            }
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn delete_seed_returns_none_for_missing_index() {
        let path = unique_path("delete-missing");
        append_seed(&path, vec![(1, 2)]).expect("save should work");

        let removed = delete_seed(&path, 99).expect("delete should not fail");
        assert_eq!(removed, None);

        let _ = fs::remove_file(path);
    }

    fn unique_path(label: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        env::temp_dir().join(format!(
            "conways-game-of-life-{label}-{}-{timestamp}.json",
            std::process::id()
        ))
    }
}
