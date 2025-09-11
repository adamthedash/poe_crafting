#![allow(non_snake_case)]
use serde_with::DisplayFromStr;
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

use serde::Deserialize;
use serde_with::serde_as;

use crate::types::TierId;

pub type Root = HashMap<String, ItemRoot>;

#[derive(Debug, Deserialize)]
pub struct ItemRoot {
    pub opt: Opt,
    pub normal: Vec<Modifier>,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Opt {
    pub ItemClassesCode: String,
    #[serde_as(as = "DisplayFromStr")]
    pub ModDomainsID: u32,
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Modifier {
    pub Code: TierId,
    #[serde_as(as = "DisplayFromStr")]
    pub DropChance: u32,
}

pub fn load(path: &Path) -> Root {
    serde_json::from_reader(BufReader::new(File::open(path).unwrap())).unwrap()
}
