pub mod repo;
pub mod storage;

use std::fs::{create_dir_all, OpenOptions};
use std::io::{Read, Write};

use actix::prelude::*;
use bytes::Bytes;
use failure::{Error, ResultExt};
use serde_json;

use elba::remote::{resolution::DirectRes, TomlDep, TomlEntry};

use crate::model::packages::{DependencyReq, PackageVersion};
use crate::CONFIG;

use self::repo::IndexRepo;
use self::storage::Storage;

pub struct Index {
    pub repo: IndexRepo,
    pub storage: Storage,
}

impl Index {
    pub fn new() -> Self {
        Index {
            repo: IndexRepo::init().expect("Failed to init index repo"),
            storage: Storage::new(),
        }
    }
}

impl Actor for Index {
    type Context = SyncContext<Self>;
}

pub struct SaveAndUpdate {
    pub package: PackageVersion,
    pub dependencies: Vec<(DependencyReq)>,
    pub bytes: Bytes,
    pub readme: Option<String>,
}

pub struct YankAndUpdate {
    pub package: PackageVersion,
    pub yanked: bool,
}

impl Message for SaveAndUpdate {
    type Result = Result<(), Error>;
}

impl Message for YankAndUpdate {
    type Result = Result<(), Error>;
}

impl Handler<SaveAndUpdate> for Index {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: SaveAndUpdate, _: &mut Self::Context) -> Self::Result {
        self.storage
            .store_package(&msg.package, msg.bytes, msg.readme)?;

        info!(
            "Updating index for publishing `{} {}`",
            &msg.package.name.as_str(),
            &msg.package.semver
        );

        // git fetch
        self.repo.fetch_and_reset()?;

        // create metadata entry
        let mut metadata = TomlEntry::from(msg.package.clone());
        for dep in msg.dependencies {
            metadata.dependencies.push(TomlDep::from(dep));
        }

        let group_path = CONFIG.local_index_path.join(&msg.package.name.group());
        let meta_path = group_path.join(&msg.package.name.name());

        create_dir_all(&group_path)?;

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&meta_path)?;
        let mut buf = serde_json::to_string(&metadata)?;
        buf.push('\n');
        file.write_all(&buf.as_bytes())?;

        // git commit && git push
        self.repo
            .commit_and_push(
                &format!(
                    "Updating package `{}|{}`",
                    &msg.package.name.as_str(),
                    &msg.package.semver
                ),
                &meta_path,
            ).context("Failed to push index to remote repo")?;

        Ok(())
    }
}

impl Handler<YankAndUpdate> for Index {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: YankAndUpdate, _: &mut Self::Context) -> Self::Result {
        info!(
            "Updating index for yanking `{} {}` (yanked={})",
            &msg.package.name.as_str(),
            &msg.package.semver,
            &msg.yanked,
        );

        // git fetch
        self.repo.fetch_and_reset()?;

        let group_path = CONFIG.local_index_path.join(&msg.package.name.group());
        let meta_path = group_path.join(&msg.package.name.name());

        if !meta_path.exists() {
            return Err(format_err!("Metafile `{:?}` not found", &meta_path));
        }

        let mut file = OpenOptions::new().read(true).open(&meta_path)?;
        let mut buf = String::new();
        let mut new_buf = String::new();
        file.read_to_string(&mut buf)?;

        for line in buf.split("\n") {
            let mut metadata: TomlEntry = match serde_json::from_str(line) {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };

            if metadata.name == msg.package.name && metadata.version == msg.package.semver {
                metadata.yanked = msg.yanked;
            }

            let new_metadata = serde_json::to_string(&metadata)?;
            new_buf += &new_metadata;
            new_buf.push('\n');
        }

        let mut file = OpenOptions::new()
            .truncate(true)
            .write(true)
            .open(&meta_path)?;
        file.write_all(&new_buf.as_bytes())?;

        // git commit && git push
        self.repo
            .commit_and_push(
                &format!(
                    "Updating package `{}|{}`",
                    &msg.package.name.as_str(),
                    &msg.package.semver
                ),
                &meta_path,
            ).context("Failed to push index to remote repo")?;

        Ok(())
    }
}

impl From<PackageVersion> for TomlEntry {
    fn from(package: PackageVersion) -> Self {
        TomlEntry {
            name: package.name.clone(),
            version: package.semver.clone(),
            location: Some(DirectRes::Registry {
                registry: CONFIG.registry.clone(),
                name: package.name,
                version: package.semver,
            }),
            dependencies: Vec::new(),
            yanked: false,
        }
    }
}

impl From<DependencyReq> for TomlDep {
    fn from(req: DependencyReq) -> Self {
        TomlDep {
            name: req.name,
            index: None,
            req: req.version_req,
        }
    }
}
