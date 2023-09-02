use lazy_regex::{Lazy, lazy_regex};
use regex::Regex;

pub(crate) mod app;
pub(crate) mod server;
pub(crate) mod date_format;

pub(crate) static UUID_REX: Lazy<Regex> = lazy_regex!("([A-f0-9]{8}-[A-f0-9]{4}-[A-f0-9]{4}-[A-f0-9]{4}-[A-f0-9]{12})");