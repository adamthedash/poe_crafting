use std::{collections::HashMap, sync::OnceLock};

use crate::types::{ItemMods, ModGroup, Modifier, StatFormatters, Tier, TierId};

pub mod crafting;
pub mod currency;
pub mod item_state;
pub mod omen;
pub mod parser_coe;
pub mod parser_dat;
pub mod parser_poe2db;
pub mod parser_stat_desc;
pub mod types;

pub static FORMATTERS: OnceLock<StatFormatters> = OnceLock::new();
pub static TIERS: OnceLock<HashMap<TierId, Tier>> = OnceLock::new();
pub static MODS: OnceLock<HashMap<ModGroup, Modifier>> = OnceLock::new();
pub static ITEM_TIERS: OnceLock<ItemMods> = OnceLock::new();
