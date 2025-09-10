use core::prelude;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use itertools::Itertools;
use poe_crafting::{
    FORMATTERS, ITEM_TIERS, MODS, TIERS,
    crafting::roll_mod,
    currency::{Augmentation, Currency, Exalt, Regal, Transmute},
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
    parser_dat::{load_mod_families, load_mod_groups, load_mod_tiers, load_stat_ids},
    parser_poe2db, parser_stat_desc,
    types::StatFormatters,
};

fn init() {
    let poe2db_root = parser_poe2db::load(Path::new(
        "/home/adam/repos/data/poe/coe/poe2db_data_altered_weights.json",
    ));

    // Create TierId -> weight LUT
    // Create BaseItemId -> [TierId] LUT
    let mut tier_weights = HashMap::new();
    let mut base_tiers = HashMap::new();
    for (item_name, item_root) in poe2db_root {
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
    let mod_groups = load_mod_groups(Path::new(
        "/home/adam/repos/data/poe/tables/data/modtype.csv",
    ));

    let mod_families = load_mod_families(Path::new(
        "/home/adam/repos/data/poe/tables/data/modfamily.csv",
    ));

    // Load stat IDs from dat files
    let stat_ids = load_stat_ids(Path::new("/home/adam/repos/data/poe/tables/data/stats.csv"));

    // Load ModGroup -> [Tier] LUT from dat files
    // Load ModGroup -> [Stat] LUT
    let (mut tiers, mod_stats) = load_mod_tiers(
        Path::new("/home/adam/repos/data/poe/tables/data/mods.csv"),
        &stat_ids,
        &mod_groups,
        &mod_families,
    );
    MODS.set(mod_stats).unwrap();

    // Apply mod weights
    for (mod_group, tier) in &mut tiers {
        tier.weight = *tier_weights.get(mod_group).unwrap_or(&0);
    }
    TIERS.set(tiers).unwrap();

    // Load stat descriptions
    let stat_desc_root = parser_stat_desc::load(Path::new(
        "/home/adam/repos/data/poe/stat_descriptions.json",
    ));

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

fn main() {
    init();

    let tiers = TIERS.get().unwrap();
    let mods = MODS.get().unwrap();
    let item_tiers = ITEM_TIERS.get().unwrap();
    let formatters = FORMATTERS.get().unwrap();

    // for (item_id, tier_ids) in item_tiers {
    //     println!("{:?}", item_id);
    //     for tier_id in tier_ids {
    //         let tier = &tiers[tier_id];
    //         let modifier = &mods[&tier.modifier];
    //
    //         let trade_string = modifier
    //             .stats
    //             .iter()
    //             .map(|stat_id| formatters[stat_id].first().unwrap().trade_string())
    //             .join(", ");
    //
    //         println!(
    //             "\t{}\t{}\t{:?}\t{}",
    //             tier.ilvl, tier.weight, tier.value_ranges, trade_string
    //         );
    //     }
    // }

    let mut item = ItemState {
        base_type: "Bow".to_string(),
        item_level: 75,
        rarity: Rarity::Normal,
        mods: vec![],
    };

    let candidate_mods = get_valid_mods_for_item(&item);

    let transmute_mods = Transmute::possible_tiers(&item, &candidate_mods);
    item.mods.push(roll_mod(&transmute_mods));
    item.rarity = Rarity::Magic;

    let aug_mods = Augmentation::possible_tiers(&item, &candidate_mods);
    item.mods.push(roll_mod(&aug_mods));

    let regal_mods = Regal::possible_tiers(&item, &candidate_mods);
    item.mods.push(roll_mod(&regal_mods));
    item.rarity = Rarity::Rare;

    for _ in 0..3 {
        let exalt_mods = Exalt::possible_tiers(&item, &candidate_mods);
        item.mods.push(roll_mod(&exalt_mods));
    }

    println!("{:?}", item);
    item.print_item();
}
