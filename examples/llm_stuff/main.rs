use std::path::Path;

use itertools::Itertools;
use poe_crafting::{
    CURRENCIES, MODS, TIERS,
    currency::CurrencyType,
    init,
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
};

fn main() {
    // let data_root = Path::new("/home/adam/repos/data/poe"); // laptop
    let data_root = Path::new("/mnt/nvme_4tb/programming/data/poe2"); // desktop
    init(data_root);

    let item = ItemState {
        base_type: "Bow".to_string(),
        item_level: 75,
        rarity: Rarity::Normal,
        mods: vec![],
    };

    CURRENCIES
        .iter()
        .filter(|c| {
            matches!(
                c,
                CurrencyType::Essence(_) | CurrencyType::PerfectEssence(_)
            )
        })
        .for_each(|e| {
            let (name, tier_ids) = match e {
                CurrencyType::Essence(essence) => {
                    (&essence.name, essence.tiers.get(&item.base_type))
                }
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
                for &tier_id in tier_ids {
                    let tier = &TIERS[tier_id];
                    let modifier = &MODS[tier.mod_id];
                    println!(
                        "\t{} ({:?}), tags: {:?}, available levels: {:?}  ",
                        modifier.group,
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
        .map(|&tier_id| &TIERS[tier_id])
        .sorted_by_key(|tier| (&tier.affix, &tier.mod_id, tier.ilvl))
        .chunk_by(|tier| (tier.affix, &tier.mod_id));
    for ((affix, &mod_id), group) in &candidate_tiers {
        let tiers = group
            .filter(|tier| tier.ilvl >= 35)
            .map(|tier| tier.ilvl)
            .collect::<Vec<_>>();

        let modifier = &MODS[mod_id];

        println!(
            "{} ({:?}), tags: {:?}, available levels: {:?}  ",
            modifier.group, affix, modifier.tags, tiers
        );
    }
}
