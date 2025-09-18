use std::{collections::HashSet, path::Path};

use poe_crafting::{
    MODS_HV, TIERS_HV,
    currency::{Currency, CurrencyType},
    init,
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
    strategy::{Condition, ConditionGroup, ModifierCondition, Strategy},
};

fn main() {
    let data_root = Path::new("/home/adam/repos/data/poe"); // laptop
    // let data_root = Path::new("/mnt/nvme_4tb/programming/data/poe2"); // desktop
    init(data_root);

    let tiers = TIERS_HV.get().unwrap();
    let mods = MODS_HV.get().unwrap();

    let item = ItemState {
        base_type: "Bow".to_string(),
        item_level: 100,
        rarity: Rarity::Normal,
        mods: vec![],
    };
    let candidate_tiers = get_valid_mods_for_item(&item);

    let desirable_magic_mods = vec![
        ModifierCondition {
            mod_group: mods.get_opaque("LocalPhysicalDamage"),
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
