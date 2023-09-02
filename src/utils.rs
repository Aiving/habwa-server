use std::path::{Path, PathBuf};

use base64::{engine::general_purpose, Engine as _};
use rcon_client::{AuthRequest, RCONClient, RCONConfig};
use tokio::fs;

use crate::{
    data::server::{CachedUser, Player},
    loaders::{self, forge::Mod},
};

pub(crate) fn encode_password<T: Into<String>>(password: T, key: T) -> String {
    let password = password.into();
    let input = password.as_bytes();
    let key = key.into();
    let k = key.as_bytes();
    let hmac = hmac_sha256::HMAC::mac(input, k).to_vec();

    general_purpose::STANDARD.encode(hmac)
}

pub(crate) fn scan_folder<T: Into<String>>(path: T, files: Vec<&str>) -> bool {
    let string_path = path.into();
    let path = Path::new(&string_path);
    let exists = files.iter().all(|file| path.join(file).exists());

    exists
}

pub(crate) async fn load_mods(path: PathBuf) -> Vec<Mod> {
    let mut folder = fs::read_dir(path.join("mods"))
        .await
        .expect("failed to read mods folder");
    let mut entries = Vec::new();

    while let Some(entry) = folder.next_entry().await.expect("failed to get next entry") {
        println!("Loading mod: {:#?}", entry.file_name());

        let mut entry = loaders::forge::load_mod_by_path(entry.path()).await;

        entries.append(&mut entry);
    }

    entries
}

pub(crate) async fn get_players(path: PathBuf, online_uuids: Vec<String>) -> Vec<Player> {
    let cache_data = fs::read(path.join("usercache.json"))
        .await
        .expect("failed to read user cache file!");
    let cache: Vec<CachedUser> =
        serde_json::from_slice(&cache_data).expect("failed to deserialize user cache file!");
    let mut players = vec![];

    for user in cache {
        // let file = fs::read(path.join(format!("world/playerdata/{}.dat", user.uuid)))
        //     .await
        //     .expect("failed to read player.dat file");

        // let mut decoder = GzipDecoder::new(Cursor::new(file.clone()));
        // let mut data = vec![];

        // decoder
        //     .read_to_end(&mut data)
        //     .await
        //     .expect("failed to decode a player.dat file");

        // let player = fastnbt::from_bytes(&data);

        players.push(Player {
            name: user.name,
            uuid: user.uuid.clone(),
            last_join_time: if online_uuids.contains(&user.uuid) {
                None
            } else {
                Some(user.expires_on)
            },
        });
    }

    players
}

pub(crate) async fn connect_rcon<T: AsRef<str>>(
    ip: Option<String>,
    port: u32,
    password: Option<T>,
) -> Option<RCONClient> {
    let client = RCONClient::new(RCONConfig {
        url: format!("{}:{}", ip.unwrap_or("0.0.0.0".to_string()), port),
        write_timeout: None,
        read_timeout: None,
    });

    if let Ok(mut client) = client {
        if let Some(password) = password {
            let auth_result = client
                .auth(AuthRequest::new(password.as_ref().to_string()))
                .unwrap();

            if !auth_result.is_success() {
                panic!("Failed to authenticate rcon!");
            }
        }

        return Some(client);
    }

    None
}
