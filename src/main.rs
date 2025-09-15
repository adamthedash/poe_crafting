use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use itertools::Itertools;
use poe_crafting::{
    ESSENCES, FORMATTERS, ITEM_TIERS, MODS, TIERS,
    currency::{CURRENCIES, Currency, CurrencyType},
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
    parser_dat::{Dats, load_essences, load_mod_tiers},
    parser_poe2db, parser_stat_desc,
    strategy::{Condition, ConditionGroup, ModifierCondition, Strategy},
    types::{BaseItemId, StatFormatters},
};
use random_choice::random_choice;

fn init() {
    // PATHS
    // data_root/
    //      tables/  - Extracted with poe_data_tools
    //      coe/     - From Prohibited Library discord
    //      stat_descriptions.json      - https://repoe-fork.github.io/poe2/stat_translations/stat_descriptions.json
    let data_root = Path::new("/home/adam/repos/data/poe"); // laptop
    // let data_root = Path::new("/mnt/nvme_4tb/programming/data/poe2"); // desktop
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

fn random_crafts() {
    let tiers = TIERS.get().unwrap();
    let mods = MODS.get().unwrap();
    let item_tiers = ITEM_TIERS.get().unwrap();
    let formatters = FORMATTERS.get().unwrap();
    // println!("{:#?}", ESSENCES.get().unwrap());
    // panic!();

    let bases = item_tiers.keys().collect::<Vec<_>>();
    let weights = vec![1.; bases.len()];

    for _ in 0..100 {
        let base_type = random_choice().random_choice_f32(&bases, &weights, 1)[0];
        let mut item = ItemState {
            base_type: (*base_type).clone(),
            item_level: 75,
            rarity: Rarity::Normal,
            mods: vec![],
        };
        let candidate_tiers = get_valid_mods_for_item(&item);

        for _ in 0..100 {
            // Select a random currency
            let currencies = CURRENCIES
                .iter()
                .chain(ESSENCES.get().unwrap())
                .filter(|c| c.can_be_used(&item, &candidate_tiers, &HashSet::new()))
                .collect::<Vec<_>>();
            let weights = vec![1.; currencies.len()];
            let currency = random_choice().random_choice_f32(&currencies, &weights, 1)[0];

            // Select random omens
            let omens = currency
                .possible_omens()
                .into_iter()
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

            println!("{:?} {:?}", omens, currency);
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

fn print_llm_stuff() {
    let item_tiers = ITEM_TIERS.get().unwrap();
    let tiers = TIERS.get().unwrap();
    let mods = MODS.get().unwrap();

    let mut item = ItemState {
        base_type: "Bow".to_string(),
        item_level: 75,
        rarity: Rarity::Normal,
        mods: vec![],
    };

    ESSENCES.get().unwrap().iter().for_each(|e| {
        let (name, tier_ids) = match e {
            CurrencyType::Essence(essence) => (&essence.name, essence.tiers.get(&item.base_type)),
            CurrencyType::PerfectEssence(essence) => {
                (&essence.name, essence.tiers.get(&item.base_type))
            }
            _ => unreachable!(),
        };

        if ["Greater", "Perfect"]
            .into_iter()
            .any(|pre| name.contains(pre))
            && let Some(tier_ids) = tier_ids
        {
            println!("{}", name);
            for tier_id in tier_ids {
                let tier = &tiers[tier_id];
                let modifier = &mods[&tier.mod_id];
                println!(
                    "\t{} ({:?}), tags: {:?}, available levels: {:?}  ",
                    tier.mod_id,
                    tier.affix,
                    modifier.tags,
                    [tier.ilvl]
                );
            }
        }
    });

    let candidate_tiers = get_valid_mods_for_item(&item);

    let candidate_tiers = candidate_tiers
        .iter()
        .map(|tier_id| &tiers[tier_id])
        .sorted_by_key(|tier| (&tier.affix, &tier.mod_id, tier.ilvl))
        .chunk_by(|tier| (tier.affix, &tier.mod_id));
    for ((affix, mod_id), group) in &candidate_tiers {
        let tiers = group
            .filter(|tier| tier.ilvl >= 35)
            .map(|tier| tier.ilvl)
            .collect::<Vec<_>>();

        println!(
            "{} ({:?}), tags: {:?}, available levels: {:?}  ",
            mod_id, affix, mods[mod_id].tags, tiers
        );
    }
}

fn main() {
    init();
    print_llm_stuff();

    let tiers = TIERS.get().unwrap();
    let mods = MODS.get().unwrap();

    let mut item = ItemState {
        base_type: "Bow".to_string(),
        item_level: 100,
        rarity: Rarity::Normal,
        mods: vec![],
    };
    let candidate_tiers = get_valid_mods_for_item(&item);

    let desirable_magic_mods = vec![
        ModifierCondition {
            mod_group: "LocalPhysicalDamage".to_string(),
            levels: vec![65, 75],
        },
        ModifierCondition {
            mod_group: "LocalFireDamage".to_string(),
            levels: vec![75, 81],
        },
        ModifierCondition {
            mod_group: "LocalColdDamage".to_string(),
            levels: vec![75, 81],
        },
        ModifierCondition {
            mod_group: "LocalLightningDamage".to_string(),
            levels: vec![75, 81],
        },
        ModifierCondition {
            mod_group: "LocalPhysicalDamagePercent".to_string(),
            levels: vec![60, 75, 82],
        },
        ModifierCondition {
            mod_group: "IncreasedWeaponElementalDamagePercent".to_string(),
            levels: vec![81],
        },
        ModifierCondition {
            mod_group: "AdditionalArrows".to_string(),
            levels: vec![82],
        },
        ModifierCondition {
            mod_group: "LocalIncreasedAttackSpeed".to_string(),
            levels: vec![37],
        },
        ModifierCondition {
            mod_group: "LocalBaseCriticalStrikeChance".to_string(),
            levels: vec![59, 73],
        },
        ModifierCondition {
            mod_group: "LocalCriticalStrikeMultiplier".to_string(),
            levels: vec![59, 73],
        },
    ];

    let strategy = Strategy(vec![
        (
            Condition {
                rarity: Rarity::Normal,
                groups: vec![],
            },
            Some((HashSet::new(), CurrencyType::PerfectTransmute)),
        ),
        (
            Condition {
                rarity: Rarity::Magic,
                groups: vec![
                    ConditionGroup::AffixCount {
                        suffixes: 0..=1,
                        prefixes: 0..=1,
                        affixes: 1..=1,
                    },
                    // T2+ of one of the mods we want
                    ConditionGroup::Count {
                        count: 1..=1,
                        mods: desirable_magic_mods.clone(),
                    },
                ],
            },
            Some((HashSet::new(), CurrencyType::PerfectAugmentation)),
        ),
        // Catchall - failed transmute
        (
            Condition {
                rarity: Rarity::Magic,
                groups: vec![ConditionGroup::AffixCount {
                    suffixes: 0..=1,
                    prefixes: 0..=1,
                    affixes: 1..=1,
                }],
            },
            None,
        ),
        (
            Condition {
                rarity: Rarity::Magic,
                groups: vec![
                    ConditionGroup::AffixCount {
                        suffixes: (0..=1),
                        prefixes: (0..=1),
                        affixes: (2..=2),
                    },
                    ConditionGroup::Count {
                        count: 2..=2,
                        mods: desirable_magic_mods.clone(),
                    },
                ],
            },
            Some((HashSet::new(), CurrencyType::GreaterRegal)),
        ),
        // Catchall - failed aug
        (
            Condition {
                rarity: Rarity::Magic,
                groups: vec![ConditionGroup::AffixCount {
                    suffixes: 1..=1,
                    prefixes: 1..=1,
                    affixes: 2..=2,
                }],
            },
            None,
        ),
    ]);

    for _ in 0..100 {
        println!("------------------------------------------------------------");
        let mut item = item.clone();

        while let Some((omens, currency)) = strategy.get_craft(&item) {
            assert!(
                currency.can_be_used(&item, &candidate_tiers, omens),
                "Currency cannot be used in current state!"
            );

            currency.craft(&mut item, &candidate_tiers, omens);
            println!("{:?} {}", omens, currency.name());
            // item.print_item();
            // println!();
        }
        item.print_item();
    }
}
