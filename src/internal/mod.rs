use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{LazyLock, OnceLock},
};

use itertools::Itertools;

use crate::{
    currency::{Currency, CurrencyType},
    hashvec::HashVec,
    parsers::{
        dat::{Dats, load_essences, load_mod_tiers},
        poe2db, stat_desc,
    },
    types::{BaseItemId, ModGroup, Modifier, StatFormatter, StatFormatters, Tier, TierId},
};

/// 2-stage initialisation of global data
/// 1) init() loads and sets the data in the OnceLock as it needs the data path passed to it
/// 2) LazyLock pulls the initialised data on request so we can just use it like a normal reference
///    rather than doing MODS.get().unwrap() everywhere.
static TIERS_INTERNAL: OnceLock<HashVec<TierId, Tier>> = OnceLock::new();
pub static TIERS: LazyLock<&HashVec<TierId, Tier>> =
    LazyLock::new(|| TIERS_INTERNAL.get().expect("init() has not been called."));

static MODS_INTERNAL: OnceLock<HashVec<ModGroup, Modifier>> = OnceLock::new();
pub static MODS: LazyLock<&HashVec<ModGroup, Modifier>> =
    LazyLock::new(|| MODS_INTERNAL.get().expect("init() has not been called."));

static ESSENCES_INTERNAL: OnceLock<Vec<CurrencyType>> = OnceLock::new();
pub static CURRENCIES: LazyLock<Vec<&CurrencyType>> = LazyLock::new(|| {
    ESSENCES_INTERNAL
        .get()
        .expect("init() has not been called.")
        .iter()
        .sorted_unstable_by_key(|e| {
            let name = e.name();
            let sort = match name.split(" ").next().unwrap() {
                "Lesser" => 0,
                "Essence" => 1,
                "Greater" => 2,
                "Perfect" => 3,
                _ => 4,
            };
            (sort, name)
        })
        .chain(CurrencyType::all().iter())
        .collect()
});

static ITEM_TIERS_INTERNAL: OnceLock<HashMap<BaseItemId, Vec<TierId>>> = OnceLock::new();
pub static ITEM_TIERS: LazyLock<&HashMap<BaseItemId, Vec<TierId>>> = LazyLock::new(|| {
    ITEM_TIERS_INTERNAL
        .get()
        .expect("init() has not been called.")
});

static FORMATTERS_INTERNAL: OnceLock<HashMap<String, Vec<StatFormatter>>> = OnceLock::new();
pub static FORMATTERS: LazyLock<&HashMap<String, Vec<StatFormatter>>> = LazyLock::new(|| {
    FORMATTERS_INTERNAL
        .get()
        .expect("init() has not been called.")
});

/// Load all of the data
/// PATHS
/// data_root/
/// -> tables/  - Extracted with poe_data_tools
/// -> coe/     - From Prohibited Library discord
/// -> stat_descriptions.json      - https://repoe-fork.github.io/poe2/stat_translations/stat_descriptions.json
pub fn init(data_root: &Path) {
    // Load weight data
    // let poe2db_root = poe2db::load(&data_root.join("coe/poe2db_data_altered_weights.json"));
    let poe2db_root = poe2db::load_embedded();

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
    ITEM_TIERS_INTERNAL.set(base_tiers).unwrap();

    // Load mod groups from dat files
    // let dat_tables = Dats::load_tables(&data_root.join("tables"));
    let dat_tables = Dats::load_tables_embedded();

    // Load ModGroup -> [Tier] LUT from dat files
    // Load ModGroup -> [Stat] LUT
    let (mut tiers, mod_stats) = load_mod_tiers(&dat_tables);
    MODS_INTERNAL.set(mod_stats).unwrap();

    // Apply mod weights
    tiers.values_mut().for_each(|tier| {
        tier.weight = *tier_weights.get(&tier.id).unwrap_or(&0);
    });
    TIERS_INTERNAL.set(tiers).unwrap();
    ESSENCES_INTERNAL.set(load_essences(&dat_tables)).unwrap();

    // Load stat descriptions
    // let stat_desc_root = stat_desc::load(&data_root.join("stat_descriptions.json"));
    let stat_desc_root = stat_desc::load_embedded();

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
    FORMATTERS_INTERNAL.set(stat_formatters).unwrap();
}
