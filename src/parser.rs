/**
Parser for Craft of Exile's mod data JSON
*/
use serde_with::DisplayFromStr;
use std::collections::HashMap;

use serde::Deserialize;
use serde_with::serde_as;

type BaseItemId = u32;
type ModId = u32;
type ModGroupId = u32;
type TagId = u32;

#[derive(Deserialize)]
pub struct ListLUT<T> {
    pub seq: Vec<T>,
    pub ind: HashMap<u32, usize>,
}

#[serde_as]
#[derive(Deserialize)]
pub struct Tier {
    #[serde_as(as = "DisplayFromStr")]
    pub ilvl: u32,
    #[serde_as(as = "DisplayFromStr")]
    pub weighting: u32,
    pub nvalues: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Affix {
    Prefix,
    Suffix,
    Socket,
}

#[serde_as]
#[derive(Deserialize)]
pub struct Modifier {
    #[serde_as(as = "DisplayFromStr")]
    pub id_modifier: ModId,
    pub modgroup: Option<u32>, // Option<String>?
    pub modgroups: String,
    pub affix: Affix,
    #[serde_as(as = "DisplayFromStr")]
    pub id_mgroup: ModGroupId,
    pub mtypes: Option<String>, // pip-delimited tag ids
}

#[serde_as]
#[derive(Deserialize)]
pub struct Tag {
    #[serde_as(as = "DisplayFromStr")]
    id_mtype: TagId,
    name_mtype: String,
}

#[serde_as]
#[derive(Deserialize)]
pub struct Root {
    /// Tiers of a given mod type on a given base item
    pub tiers: HashMap<ModId, HashMap<BaseItemId, Vec<Tier>>>,
    /// Mapping between base item IDs and the mods that can roll on them
    #[serde_as(as = "HashMap<_, Vec<DisplayFromStr>>")]
    pub basemods: HashMap<BaseItemId, Vec<ModId>>,
    /// Mod tags
    pub mtypes: ListLUT<Tag>,

    pub modifiers: ListLUT<Modifier>,
}
