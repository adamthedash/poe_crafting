/**
Parser for Craft of Exile's mod data JSON
*/
use serde_with::DisplayFromStr;
use std::collections::HashMap;

use serde::{Deserialize, Deserializer};
use serde_with::serde_as;

pub type BaseItemId = u32;
pub type BaseGroupId = u32;
pub type ModId = u32;
pub type ModGroupId = u32;
pub type TagId = u32;

/// Mod tags, eg. "|1|2|3|4|"
fn deserialize_pipe_delimited_option<'de, D>(deserializer: D) -> Result<Option<Vec<u32>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            let values = s
                .trim_matches('|')
                .split('|')
                .filter(|s| !s.is_empty())
                .map(|s| s.parse::<u32>().map_err(serde::de::Error::custom))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Some(values))
        }
        None => Ok(None),
    }
}

/// Json-stringified list of strings, eg. "[\"hello\", \"world\"]"
fn deserialize_json_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

/// Json-stringified list of modifier intervals, eg. "[1, [2, 3]]"
fn deserialize_json_value_array<'de, D>(deserializer: D) -> Result<Vec<ModifierValues>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
pub struct ListLUT<T> {
    pub seq: Vec<T>,
    pub ind: HashMap<u32, usize>,
}

/// Values which a tier of mod can roll within. Eg. "(5 - 12) to (52 - 69) lightning damage"
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ModifierValues {
    ConstantU32(u32),
    IntervalU32(Vec<u32>),
    ConstantI32(i32),
    IntervalI32(Vec<i32>),
    Constantf32(f32),
    Intervalf32(Vec<f32>),
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Tier {
    #[serde_as(as = "DisplayFromStr")]
    pub ilvl: u32,
    #[serde_as(as = "DisplayFromStr")]
    pub weighting: u32,
    #[serde(deserialize_with = "deserialize_json_value_array")]
    pub nvalues: Vec<ModifierValues>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Affix {
    Prefix,
    Suffix,
    Socket,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Modifier {
    #[serde_as(as = "DisplayFromStr")]
    pub id_modifier: ModId,
    pub name_modifier: String,
    #[serde(deserialize_with = "deserialize_json_string")]
    pub modgroups: Vec<String>,
    pub affix: Affix,
    #[serde(deserialize_with = "deserialize_pipe_delimited_option")]
    pub mtypes: Option<Vec<u32>>,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Tag {
    #[serde_as(as = "DisplayFromStr")]
    pub id_mtype: TagId,
    pub name_mtype: String,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct ItemBase {
    #[serde_as(as = "DisplayFromStr")]
    pub id_bgroup: BaseGroupId,
    #[serde_as(as = "DisplayFromStr")]
    pub id_base: BaseItemId,
    pub name_base: String,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Root {
    /// Tiers of a given mod type on a given base item
    pub tiers: HashMap<ModId, HashMap<BaseItemId, Vec<Tier>>>,
    /// Mapping between base item IDs and the mods that can roll on them
    #[serde_as(as = "HashMap<_, Vec<DisplayFromStr>>")]
    pub basemods: HashMap<BaseItemId, Vec<ModId>>,
    /// Mod tags
    pub mtypes: ListLUT<Tag>,
    /// Modifier details
    pub modifiers: ListLUT<Modifier>,
    /// Item bases
    pub bases: ListLUT<ItemBase>,
}
