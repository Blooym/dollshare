use crate::cryptography::{self, Cryptography};
use anyhow::Result;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct StorageHandler {
    uploads_path: PathBuf,
    expire_after: Duration,
}

impl StorageHandler {
    pub fn new(uploads_base_path: &Path, expire_after: Duration) -> Result<Self> {
        fs::create_dir_all(uploads_base_path)?;
        Ok(Self {
            uploads_path: uploads_base_path.to_path_buf(),
            expire_after,
        })
    }

    fn is_upload_expired(&self, filename: &str) -> Result<bool> {
        let metadata = fs::metadata(self.uploads_path.join(filename))?;
        let last_access = metadata.accessed()?;

        if last_access + self.expire_after <= SystemTime::now() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn remove_expired_files(&self) -> Result<()> {
        fs::read_dir(&self.uploads_path)
            .unwrap()
            .filter_map(|f| f.ok())
            .for_each(|file| {
                let Ok(file_name) = file.file_name().into_string() else {
                    return;
                };
                if self.is_upload_expired(&file_name).unwrap() {
                    info!("'{}' has expired - deleting from storage.", file_name);
                    self.delete_upload(&file_name).unwrap();
                }
            });
        Ok(())
    }

    pub fn upload_exists(&self, filename: &str) -> Result<bool> {
        debug!("Checking if {filename} exists in storage");
        Ok(fs::exists(
            self.uploads_path.join(self.uploads_path.join(filename)),
        )?)
    }

    pub fn get_upload(&self, filename: &str, key: &str) -> Result<Vec<u8>> {
        debug!("Decrypting and fetching {filename} from storage");
        cryptography::Cryptography::decrypt(&fs::read(self.uploads_path.join(filename))?, key)
    }

    pub fn store_upload(&self, filename: &str, bytes: &[u8]) -> Result<String> {
        debug!("Encrypting and saving {filename} to storage");
        let (key, bytes) = Cryptography::encrypt(bytes)?;
        fs::write(self.uploads_path.join(filename), bytes)?;
        Ok(key)
    }

    pub fn delete_upload(&self, filename: &str) -> Result<()> {
        debug!("Deleting {filename} from storage");
        fs::remove_file(self.uploads_path.join(filename))?;
        Ok(())
    }
}
