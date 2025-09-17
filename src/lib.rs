use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::OnceLock,
};

use crate::{
    currency::CurrencyType,
    parser_dat::{Dats, load_essences, load_mod_tiers},
    types::{BaseItemId, ItemMods, ModGroup, Modifier, StatFormatters, Tier, TierId},
};

pub mod crafting;
pub mod currency;
pub mod item_state;
pub mod parser_coe;
pub mod parser_dat;
pub mod parser_poe2db;
pub mod parser_stat_desc;
pub mod strategy;
pub mod types;

pub static FORMATTERS: OnceLock<StatFormatters> = OnceLock::new();
pub static TIERS: OnceLock<HashMap<TierId, Tier>> = OnceLock::new();
pub static MODS: OnceLock<HashMap<ModGroup, Modifier>> = OnceLock::new();
pub static ITEM_TIERS: OnceLock<ItemMods> = OnceLock::new();
pub static ESSENCES: OnceLock<Vec<CurrencyType>> = OnceLock::new();

/// Load all of the data
pub fn init(data_root: &Path) {
    // PATHS
    // data_root/
    //      tables/  - Extracted with poe_data_tools
    //      coe/     - From Prohibited Library discord
    //      stat_descriptions.json      - https://repoe-fork.github.io/poe2/stat_translations/stat_descriptions.json
    // Load weight data
    let poe2db_root = parser_poe2db::load(&data_root.join("coe/poe2db_data_altered_weights.json"));

    // Create TierId -> weight LUT
    let mut tier_weights = HashMap::new();
    // Create BaseItemId -> [TierId] LUT
    let mut base_tiers = HashMap::new();
    // Gloves -> {Gloves_StrDex, Gloves_DexInt, ...}
    let mut specific_bases = HashMap::<String, HashSet<BaseItemId>>::new();
    for (item_name, item_root) in poe2db_root {
        specific_bases
            .entry(item_root.opt.ItemClassesCode)
            .or_default()
            .insert(item_name.clone());

        for tier in &item_root.normal {
            tier_weights.insert(tier.Code.clone(), tier.DropChance);
        }

        base_tiers.insert(
            item_name.clone(),
            item_root
                .normal
                .iter()
                .map(|m| m.Code.clone())
                .collect::<Vec<_>>(),
        );
    }
    ITEM_TIERS.set(base_tiers).unwrap();

    // Load mod groups from dat files
    let dat_tables = Dats::load_tables(&data_root.join("tables"));

    // Load ModGroup -> [Tier] LUT from dat files
    // Load ModGroup -> [Stat] LUT
    let (mut tiers, mod_stats) = load_mod_tiers(&dat_tables);
    MODS.set(mod_stats).unwrap();

    // Apply mod weights
    for (mod_group, tier) in &mut tiers {
        tier.weight = *tier_weights.get(mod_group).unwrap_or(&0);
    }
    TIERS.set(tiers).unwrap();
    ESSENCES.set(load_essences(&dat_tables)).unwrap();

    // Load stat descriptions
    let stat_desc_root = parser_stat_desc::load(&data_root.join("stat_descriptions.json"));

    // Create StatID -> [Formatter] LUT
    let mut stat_formatters = StatFormatters::new();
    for m in &stat_desc_root {
        // Add a multi-stat formatter
        let key = m.ids.join("|");
        stat_formatters.insert(key, m.English.clone());

        // Also add as per-stat formatters
        for key in &m.ids {
            stat_formatters.insert(key.clone(), m.English.clone());
        }
    }
    FORMATTERS.set(stat_formatters).unwrap();
}
