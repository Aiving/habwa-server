use std::path::PathBuf;

use rcon_client::RCONClient;
use tokio::sync::Mutex;

use crate::loaders::forge::Mod;

use super::server;

pub(crate) struct State {
    pub(crate) rcon: Mutex<Option<RCONClient>>,
    pub(crate) properties: server::Properties,
    pub(crate) path: PathBuf,
    pub(crate) access_token: Option<String>,
    pub(crate) encryption_key: Option<String>,
    pub(crate) mods: Mutex<Vec<Mod>>,
}

impl State {
    pub(crate) fn new<T: Into<String>>(
        rcon: Option<RCONClient>,
        path: PathBuf,
        properties: server::Properties,
        token: Option<T>,
        key: Option<T>,
        mods: Vec<Mod>,
    ) -> State {
        let mut access_token = None;

        if let Some(token) = token {
            access_token = Some(token.into());
        }

        let mut encryption_key = None;

        if let Some(key) = key {
            encryption_key = Some(key.into());
        }

        Self {
            rcon: Mutex::new(rcon),
            properties,
            path,
            access_token,
            encryption_key,
            mods: Mutex::new(mods),
        }
    }
}
