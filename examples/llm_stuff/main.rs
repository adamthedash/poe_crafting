use std::path::Path;

use itertools::Itertools;
use poe_crafting::{
    ESSENCES, MODS_HV, TIERS_HV,
    currency::CurrencyType,
    init,
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
};

fn main() {
    // let data_root = Path::new("/home/adam/repos/data/poe"); // laptop
    let data_root = Path::new("/mnt/nvme_4tb/programming/data/poe2"); // desktop
    init(data_root);

    let tiers = TIERS_HV.get().unwrap();
    let mods = MODS_HV.get().unwrap();

    let item = ItemState {
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
            for &tier_id in tier_ids {
                let tier = &tiers[tier_id];
                let modifier = &mods[tier.mod_id];
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
        .map(|&tier_id| &tiers[tier_id])
        .sorted_by_key(|tier| (&tier.affix, &tier.mod_id, tier.ilvl))
        .chunk_by(|tier| (tier.affix, &tier.mod_id));
    for ((affix, &mod_id), group) in &candidate_tiers {
        let tiers = group
            .filter(|tier| tier.ilvl >= 35)
            .map(|tier| tier.ilvl)
            .collect::<Vec<_>>();

        let modifier = &mods[mod_id];

        println!(
            "{} ({:?}), tags: {:?}, available levels: {:?}  ",
            modifier.group, affix, modifier.tags, tiers
        );
    }
}
