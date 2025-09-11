use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use itertools::Itertools;
use poe_crafting::{
    FORMATTERS, ITEM_TIERS, MODS, TIERS,
    currency::{CURRENCIES, Currency},
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
    parser_dat::{load_mod_families, load_mod_groups, load_mod_tiers, load_stat_ids},
    parser_poe2db, parser_stat_desc,
    types::StatFormatters,
};
use random_choice::random_choice;

fn init() {
    // PATHS
    // data_root/
    //      tables/  - Extracted with poe_data_tools
    //      coe/     - From Prohibited Library discord
    //      stat_descriptions.json      - https://repoe-fork.github.io/poe2/stat_translations/stat_descriptions.json
    // let data_root = Path::new("/home/adam/repos/data/poe"); // laptop
    let data_root = Path::new("/mnt/nvme_4tb/programming/data/poe2"); // desktop
    let poe2_dat_root = data_root.join("tables");

    // Load weight data
    let poe2db_root = parser_poe2db::load(&data_root.join("coe/poe2db_data_altered_weights.json"));

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
    let mod_groups = load_mod_groups(&poe2_dat_root.join("data/modtype.csv"));

    let mod_families = load_mod_families(&poe2_dat_root.join("data/modfamily.csv"));

    // Load stat IDs from dat files
    let stat_ids = load_stat_ids(&poe2_dat_root.join("data/stats.csv"));

    // Load ModGroup -> [Tier] LUT from dat files
    // Load ModGroup -> [Stat] LUT
    let (mut tiers, mod_stats) = load_mod_tiers(
        &poe2_dat_root.join("data/mods.csv"),
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

fn main() {
    init();

    let tiers = TIERS.get().unwrap();
    let mods = MODS.get().unwrap();
    let item_tiers = ITEM_TIERS.get().unwrap();
    let formatters = FORMATTERS.get().unwrap();

    println!("{:?}", tiers["EvasionGrantsDeflection5"]);
    panic!();

    let bases = item_tiers.keys().collect::<Vec<_>>();
    let weights = vec![1.; bases.len()];

    for _ in 0..10 {
        let base_type = random_choice().random_choice_f32(&bases, &weights, 1)[0];
        let mut item = ItemState {
            base_type: (*base_type).clone(),
            item_level: 75,
            rarity: Rarity::Normal,
            mods: vec![],
        };
        let candidate_tiers = get_valid_mods_for_item(&item);

        for _ in 0..1000 {
            // Select a random currency
            let currencies = CURRENCIES
                .iter()
                .filter(|c| c.can_be_used(&item, &candidate_tiers, &HashSet::new()))
                .collect::<Vec<_>>();
            let weights = vec![1.; currencies.len()];
            let currency = random_choice().random_choice_f32(&currencies, &weights, 1)[0];

            // Select random omens
            let omens = currency
                .possible_omens()
                .into_iter()
                // Filter out non-implemented
                .filter(|omen| {
                    !["Whittling".to_string(), "Homogenising".to_string()].contains(omen)
                })
                // Only omens that can be used
                .filter(|omen| {
                    currency.can_be_used(
                        &item,
                        &candidate_tiers,
                        &HashSet::from_iter(std::iter::once(omen.clone())),
                    )
                })
                .collect::<Vec<_>>();
            let weights = vec![1.; omens.len()];
            let omens = HashSet::from_iter(
                random_choice()
                    .random_choice_f32(&omens, &weights, 1)
                    .into_iter()
                    .map(|o| (*o).clone()),
            );

            let before = item.clone();
            currency.craft(&mut item, &candidate_tiers, &omens);
            if !item.is_valid() {
                println!("invalid item");
                before.print_item();
                println!("- {:?} {:?} ->", omens, currency);
                item.print_item();
                panic!()
            }
        }

        item.print_item();
        println!();
    }
}
