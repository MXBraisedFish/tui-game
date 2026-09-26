use std::{
  fs::{self, OpenOptions},
  io::{self, Write},
  path::{Path, PathBuf},
};

pub fn atomic_write(path: &Path, bytes: &[u8], durable: bool) -> io::Result<()> {
  atomic_replace_with(path, durable, |temporary| {
    let mut file = OpenOptions::new()
      .create(true)
      .truncate(true)
      .write(true)
      .open(temporary)?;
    file.write_all(bytes)?;
    file.flush()
  })
}

pub fn atomic_replace_with(
  path: &Path,
  durable: bool,
  write: impl FnOnce(&Path) -> io::Result<()>,
) -> io::Result<()> {
  let temporary = temporary_path(path);
  let backup = backup_path(path);
  let _ = fs::remove_file(&temporary);
  if let Err(error) = write(&temporary) {
    let _ = fs::remove_file(&temporary);
    return Err(error);
  }
  if durable {
    let sync_result = OpenOptions::new()
      .read(true)
      .write(true)
      .open(&temporary)
      .and_then(|file| file.sync_all());
    if let Err(error) = sync_result {
      let _ = fs::remove_file(&temporary);
      return Err(error);
    }
  }

  let had_original = path.exists();
  if had_original {
    let _ = fs::remove_file(&backup);
    if let Err(error) = fs::rename(path, &backup) {
      let _ = fs::remove_file(&temporary);
      return Err(error);
    }
  }
  match fs::rename(&temporary, path) {
    Ok(()) => {
      if had_original {
        let _ = fs::remove_file(backup);
      }
      Ok(())
    }
    Err(error) => {
      if had_original {
        let _ = fs::rename(&backup, path);
      }
      let _ = fs::remove_file(temporary);
      Err(error)
    }
  }
}

/// Temporary file the atomic writers fill before replacing `path` (`name.ext` -> `name.ext.tmp`).
pub fn temporary_path(path: &Path) -> PathBuf {
  let extension = path
    .extension()
    .and_then(|value| value.to_str())
    .map(|value| format!("{value}.tmp"))
    .unwrap_or_else(|| "tmp".to_string());
  path.with_extension(extension)
}

fn backup_path(path: &Path) -> PathBuf {
  let extension = path
    .extension()
    .and_then(|value| value.to_str())
    .map(|value| format!("{value}.bak"))
    .unwrap_or_else(|| "bak".to_string());
  path.with_extension(extension)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn replaces_complete_file_and_removes_temporary_files() {
    let dir = std::env::temp_dir().join(format!("tg_atomic_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("profile.json");
    fs::write(&path, b"old").unwrap();
    atomic_write(&path, b"new", false).unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"new");
    assert!(!dir.join("profile.json.tmp").exists());
    assert!(!dir.join("profile.json.bak").exists());
    let _ = fs::remove_dir_all(dir);
  }

  #[test]
  fn failed_write_keeps_the_original_and_removes_temporary_file() {
    let dir = std::env::temp_dir().join(format!("tg_atomic_failure_{}", std::process::id()));
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("profile.json");
    fs::write(&path, b"old").unwrap();

    let error = atomic_replace_with(&path, false, |temporary| {
      fs::write(temporary, b"incomplete")?;
      Err(io::Error::other("interrupted"))
    })
    .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(fs::read(&path).unwrap(), b"old");
    assert!(!dir.join("profile.json.tmp").exists());
    let _ = fs::remove_dir_all(dir);
  }
}
