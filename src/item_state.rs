use std::collections::HashSet;

use crate::{
    FORMATTERS, ITEM_TIERS, MODS, TIERS,
    types::{Affix, BaseItemId, ModFamily, ModTag, TierId, get_matching_formatter},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Rarity {
    Normal,
    Magic,
    Rare,
}

#[derive(Debug, Clone)]
pub struct ItemState {
    pub base_type: BaseItemId,
    pub item_level: u32,
    pub rarity: Rarity,
    pub mods: Vec<TierId>,
}

impl ItemState {
    pub fn num_prefixes(&self) -> usize {
        let tiers = TIERS.get().unwrap();
        self.mods
            .iter()
            .filter(|tier_id| tiers[*tier_id].affix == Affix::Prefix)
            .count()
    }

    pub fn num_suffixes(&self) -> usize {
        let tiers = TIERS.get().unwrap();
        self.mods
            .iter()
            .filter(|tier_id| tiers[*tier_id].affix == Affix::Suffix)
            .count()
    }

    /// Set of unique mod tags
    pub fn mod_tags(&self) -> HashSet<ModTag> {
        let tiers = TIERS.get().unwrap();
        let mods = MODS.get().unwrap();

        self.mods
            .iter()
            .fold(HashSet::new(), |mut all_tags, tier_id| {
                let tier = &tiers[tier_id];
                let modifier = &mods[&tier.mod_id];

                all_tags.extend(modifier.tags.iter().cloned());

                all_tags
            })
    }

    /// Mod familities for each tier
    pub fn mod_familities(&self) -> HashSet<ModFamily> {
        let tiers = TIERS.get().unwrap();
        let mods = MODS.get().unwrap();

        self.mods
            .iter()
            .map(|tier_id| {
                let tier = &tiers[tier_id];
                let modifier = &mods[&tier.mod_id];
                &modifier.family
            })
            .cloned()
            .collect()
    }

    pub fn print_item(&self) {
        let tiers = TIERS.get().unwrap();
        let mods = MODS.get().unwrap();
        let stat_formatters = FORMATTERS.get().unwrap();

        println!("{}", self.base_type);
        println!("ilvl: {}", self.item_level);
        println!("{:?}", self.rarity);
        println!("=====================");
        for tier_id in &self.mods {
            let tier = &tiers[tier_id];
            let modifier = &mods[&tier.mod_id];

            let formatters_key = modifier.stats.join("|");
            if let Some(formatters) = stat_formatters.get(&formatters_key) {
                let formatter = get_matching_formatter(
                    formatters,
                    &tier
                        .value_ranges
                        .iter()
                        .map(|[min, _]| *min)
                        .collect::<Vec<_>>(),
                );
                // Match on multi-stat formatter
                let string = formatter.format_value_range(&tier.value_ranges);
                println!("{}", string);
            } else {
                // Per-stat formatters
                for (stat_id, value_range) in modifier.stats.iter().zip(tier.value_ranges.chunks(1))
                {
                    let Some(formatters) = stat_formatters.get(stat_id) else {
                        println!("No formatter for stat: {:?}", stat_id);
                        continue;
                    };
                    let formatter = get_matching_formatter(
                        formatters,
                        &tier
                            .value_ranges
                            .iter()
                            .map(|[min, _]| *min)
                            .collect::<Vec<_>>(),
                    );
                    // TODO: select formatter based on tier
                    let string = formatter.format_value_range(value_range);

                    println!("{}", string);
                }
            }
        }
    }

    /// Checks whether the current state of the item is valid
    pub fn is_valid(&self) -> bool {
        let tiers = TIERS.get().unwrap();
        let mods = MODS.get().unwrap();

        let num_mods_ok = match self.rarity {
            Rarity::Normal => self.mods.is_empty(),
            Rarity::Magic => self.mods.len() <= 2,
            Rarity::Rare => self.mods.len() <= 6,
        };

        let num_affixes_ok = match self.rarity {
            Rarity::Normal => true,
            Rarity::Magic => self.num_prefixes() <= 1 && self.num_suffixes() <= 1,
            Rarity::Rare => self.num_prefixes() <= 3 && self.num_suffixes() <= 3,
        };

        let mod_ilvls_ok = self.mods.iter().all(|tier_id| {
            let tier = &tiers[tier_id];

            tier.ilvl <= self.item_level
        });

        let mod_families_ok = self
            .mods
            .iter()
            .map(|tier_id| {
                let tier = &tiers[tier_id];
                let modifier = &mods[&tier.mod_id];

                &modifier.family
            })
            .collect::<HashSet<_>>()
            .len()
            == self.mods.len();

        num_mods_ok && num_affixes_ok && mod_ilvls_ok && mod_families_ok
    }
}

/// Get the pool of mods that could ever roll on this item, regardless of its current state
pub fn get_valid_mods_for_item<'a>(item: &ItemState) -> Vec<TierId> {
    let tiers = TIERS.get().unwrap();
    let item_tiers = ITEM_TIERS.get().unwrap();

    item_tiers[&item.base_type]
        .iter()
        .filter(|tier_id| item.item_level >= tiers[*tier_id].ilvl)
        .cloned()
        .collect()
}
