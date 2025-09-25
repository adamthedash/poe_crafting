#![allow(non_snake_case)]
use std::{fs::File, io::BufReader, path::Path};

/**
stat_descriptions.csd - format strings for stats
https://repoe-fork.github.io/poe2/stat_translations/stat_descriptions.json
["ids"]: List of stat IDs this string applies to Eg. %_attack_damage_per_glory_consumed_for_6_seconds_up_to_100
["English"][n]["string"]: Format string with {i} and [either|or] placeholders
["English"][n]["condition"]: Conditions when this format string applies Eg. 0-99%, 100%
["English"][n]["format"]: How to format the value
["English"][n]["index_handlers"]: How the value should be pre-processed. Eg. "divide_by_100"
["trade_stats"][0]["text"]: Format strings as they appear on trade - this looks like the best
        bet for matching to CoE weights. Not present for all stats
*/
use serde::Deserialize;

use crate::types::StatFormatter;

pub type Root = Vec<Modifier>;

#[derive(Debug, Deserialize)]
pub struct Modifier {
    /// Eg. additional_strength
    pub ids: Vec<String>,
    pub English: Vec<StatFormatter>,
    pub trade_stats: Option<Vec<TradeStat>>,
}

#[derive(Debug, Deserialize)]
pub struct TradeStat {
    /// Eg. "desecrated.stat_123456"
    pub id: String,
    /// Eg. "# Life Regeneration per second"
    pub text: String,
    /// Eg. desecrated
    #[serde(rename = "type")]
    pub stat_type: String,
}

pub fn load(path: &Path) -> Root {
    serde_json::from_reader(BufReader::new(File::open(path).unwrap())).unwrap()
}

pub fn load_embedded() -> Root {
    serde_json::from_slice(include_bytes!("../../data/stat_descriptions.json")).unwrap()
}
