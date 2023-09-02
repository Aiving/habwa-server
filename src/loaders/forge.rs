use std::{collections::HashMap, io::Cursor, path::PathBuf};

use async_zip::{base::read::seek::ZipFileReader, ZipFile};
use serde::{Deserialize, Serialize};
use tokio::fs;
use toml::Value;

use crate::parsers;

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct ModManifest {
    #[serde(rename = "modLoader")]
    pub(crate) mod_loader: String,
    #[serde(rename = "loaderVersion")]
    pub(crate) loader_version: String,
    pub(crate) license: String,
    #[serde(rename = "showAsResourcePack")]
    pub(crate) show_as_resource_pack: Option<bool>,

    pub(crate) services: Option<Vec<String>>,
    pub(crate) properties: Option<HashMap<String, Value>>,

    #[serde(rename = "issueTrackerURL")]
    pub(crate) issue_tracker_url: Option<String>,

    pub(crate) mods: Vec<Mod>,
    pub(crate) dependencies: Option<HashMap<String, Vec<ModDependency>>>,
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct Mod {
    #[serde(rename = "modId")]
    pub(crate) mod_id: String,
    pub(crate) namespace: Option<String>,
    #[serde(default = "default_version")]
    pub(crate) version: String,
    #[serde(rename = "displayName")]
    pub(crate) display_name: Option<String>,
    #[serde(default = "default_description")]
    pub(crate) description: String,
    #[serde(rename = "logoFile")]
    pub(crate) logo_file: Option<String>,
    #[serde(rename = "logoBlur", default = "default_logo_blur")]
    pub(crate) logo_blur: bool,
    #[serde(rename = "updateJSONURL")]
    pub(crate) update_json_url: Option<String>,
    #[serde(default = "default_features")]
    pub(crate) features: ModFeatures,
    #[serde(default = "default_modproperties")]
    pub(crate) modproperties: HashMap<String, Value>,
    #[serde(rename = "modUrl")]
    pub(crate) mod_url: Option<String>,
    pub(crate) credits: Option<String>,
    pub(crate) authors: Option<String>,
    #[serde(rename = "displayURL")]
    pub(crate) display_url: Option<String>,
    #[serde(default = "default_display_test")]
    pub(crate) display_test: ModDisplayTest,
    pub(crate) dependencies: Option<Vec<ModDependency>>,
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct ModDependency {
    #[serde(rename = "modId")]
    pub(crate) mod_id: String,
    pub(crate) mandatory: bool,
    #[serde(rename = "versionRange", default = "default_version_range")]
    pub(crate) version_range: String,
    #[serde(default = "default_ordering")]
    pub(crate) ordering: DependencyOrdering,
    #[serde(default = "default_side")]
    pub(crate) side: DependencySide,
    #[serde(rename = "referralUrl")]
    pub(crate) referral_url: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum DependencyOrdering {
    None,
    Before,
    After,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum DependencySide {
    Client,
    Server,
    Both,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum ModDisplayTest {
    None,
    MatchVersion,
    IgnoreServerVersion,
    IgnoreAllVersion,
}

#[derive(Deserialize, Serialize, Clone)]
pub(crate) struct ModFeatures {
    pub(crate) java_version: Option<String>,
}

fn default_side() -> DependencySide {
    DependencySide::Both
}

fn default_ordering() -> DependencyOrdering {
    DependencyOrdering::None
}

fn default_version_range() -> String {
    "".to_string()
}

fn default_display_test() -> ModDisplayTest {
    ModDisplayTest::MatchVersion
}

fn default_modproperties() -> HashMap<String, Value> {
    HashMap::new()
}

fn default_features() -> ModFeatures {
    ModFeatures { java_version: None }
}

fn default_logo_blur() -> bool {
    true
}

fn default_description() -> String {
    "MISSING DESCRIPTION".to_string()
}

fn default_version() -> String {
    "1".to_string()
}

fn archive_contains(archive: &ZipFile, files: Vec<&str>) -> bool {
    let entries = archive.entries();
    let mut entries_iterator = entries.iter();

    let exists = files.iter().all(|file| {
        entries_iterator.any(|stored_entry| {
            stored_entry
                .entry()
                .filename()
                .clone()
                .as_str()
                .is_ok_and(|filename| filename.ends_with(file))
        })
    });

    exists
}

pub(crate) async fn is_mod(data: &[u8]) -> bool {
    let reader = ZipFileReader::with_tokio(Cursor::new(data))
        .await
        .expect("failed to read mod file");
    let archive = reader.file();

    archive_contains(
        archive,
        vec!["META-INF/MANIFEST.MF", "META-INF/mods.toml"],
    )
}

pub(crate) async fn load_mod_by_path(path: PathBuf) -> Vec<Mod> {
    let file = fs::File::open(path).await.expect("failed to open mod file");

    load_mod(file).await
}

pub(crate) async fn load_mod<R: tokio::io::AsyncRead + tokio::io::AsyncSeek + Unpin>(
    data: R,
) -> Vec<Mod> {
    let mut reader = ZipFileReader::with_tokio(data)
        .await
        .expect("failed to read mod file");
    let archive = reader.file();
    let files = archive.entries();
    let mut mods = Vec::new();

    let manifest_index = files.iter().position(|file| {
        file.entry()
            .filename()
            .clone()
            .into_string()
            .is_ok_and(|filename| filename.ends_with("MANIFEST.MF"))
    });

    let toml_index = files.iter().position(|file| {
        file.entry()
            .filename()
            .clone()
            .into_string()
            .is_ok_and(|filename| filename.ends_with("mods.toml"))
    });

    if let Some(index) = toml_index {
        let mut data = reader.reader_with_entry(index).await.unwrap();
        let mut buffer = String::new();

        data.read_to_string_checked(&mut buffer)
            .await
            .expect("failed to read a mods.toml inside mod file");

        let manifest: ModManifest =
            toml::from_str(&buffer).expect("failed to deserialize mods.toml buffer");

        if let Some(dependencies) = manifest.dependencies {
            for mut r#mod in manifest.mods {
                r#mod.dependencies = dependencies.get(&r#mod.mod_id).cloned();

                if let Some(index) = manifest_index {
                    let mut data = reader.reader_with_entry(index).await.unwrap();
                    let mut buffer = String::new();

                    data.read_to_string_checked(&mut buffer)
                        .await
                        .expect("failed to read a MANIFEST.MF inside mod file");

                    let manifest = parsers::manifest::parse(buffer);

                    if r#mod.version == "${file.jarVersion}"
                        && manifest.contains_key("Implementation-Version")
                    {
                        r#mod.version = manifest.get("Implementation-Version").unwrap().clone();
                    }
                }

                mods.push(r#mod);
            }
        } else {
            for mut r#mod in manifest.mods {
                if let Some(index) = manifest_index {
                    let mut data = reader.reader_with_entry(index).await.unwrap();
                    let mut buffer = String::new();

                    data.read_to_string_checked(&mut buffer)
                        .await
                        .expect("failed to read a MANIFEST.MF inside mod file");

                    let manifest = parsers::manifest::parse(buffer);

                    if r#mod.version == "${file.jarVersion}"
                        && manifest.contains_key("Implementation-Version")
                    {
                        r#mod.version = manifest.get("Implementation-Version").unwrap().clone();
                    }
                }

                mods.push(r#mod);
            }
        }
    }

    mods
}
