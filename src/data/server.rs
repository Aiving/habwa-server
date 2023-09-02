use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::parsers;

#[derive(Serialize, Clone)]
pub(crate) struct Properties {
    pub(crate) ip: Option<String>,
    pub(crate) port: u32,
    pub(crate) max_players: u32,
    pub(crate) motd: Option<String>,
    pub(crate) online_mode: bool,
    pub(crate) pvp: bool,
    pub(crate) hardcore: bool,
    pub(crate) rcon: RCONProperty,
    pub(crate) whitelist: bool,
}

#[derive(Serialize, Clone)]
pub(crate) struct RCONProperty {
    pub(crate) enabled: bool,
    pub(crate) password: Option<String>,
    pub(crate) port: u32,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CachedUser {
    pub(crate) name: String,
    pub(crate) uuid: String,
    #[serde(with = "super::date_format::user")]
    pub(crate) expires_on: DateTime<FixedOffset>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub(crate) struct Player {
    pub(crate) name: String,
    pub(crate) uuid: String,
    #[serde(with = "super::date_format::user_optional")]
    pub(crate) last_join_time: Option<DateTime<FixedOffset>>,
}

impl Properties {
    pub(crate) fn new<T: Into<String>>(data: T) -> Properties {
        let properties = parsers::properties::parse(data);

        let ip = properties.get("server-ip").unwrap().as_string();
        let port = properties.get("server-port").unwrap().as_u32().unwrap();
        let max_players = properties.get("max-players").unwrap().as_u32().unwrap();
        let motd = properties.get("motd").unwrap().as_string();
        let online_mode = properties.get("online-mode").unwrap().as_bool().unwrap();
        let pvp = properties.get("pvp").unwrap().as_bool().unwrap();
        let hardcore = properties.get("hardcore").unwrap().as_bool().unwrap();
        let whitelist = properties.get("white-list").unwrap().as_bool().unwrap();

        let rcon_enabled = properties.get("enable-rcon").unwrap().as_bool().unwrap();
        let rcon_password = properties.get("rcon.password").unwrap().as_string();
        let rcon_port = properties.get("rcon.port").unwrap().as_u32().unwrap();

        Properties {
            ip,
            port,
            max_players,
            motd,
            online_mode,
            pvp,
            hardcore,
            rcon: RCONProperty::new(rcon_enabled, rcon_password, rcon_port),
            whitelist,
        }
    }
}

impl RCONProperty {
    pub(crate) fn new(enabled: bool, password: Option<String>, port: u32) -> RCONProperty {
        Self {
            enabled,
            password,
            port,
        }
    }
}
