use std::{collections::HashSet, fmt::Display};

use serde::{Deserialize, Serialize};

use crate::{
    FORMATTERS, ITEM_TIERS, MODS_HV, TIERS_HV,
    hashvec::OpaqueIndex,
    types::{Affix, BaseItemId, ModFamily, ModTag, Tier, get_matching_formatter},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Rarity {
    Normal,
    Magic,
    Rare,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemState {
    pub base_type: BaseItemId,
    pub item_level: u32,
    pub rarity: Rarity,
    pub mods: Vec<OpaqueIndex<Tier>>,
}

impl ItemState {
    pub fn num_prefixes(&self) -> usize {
        let tiers = TIERS_HV.get().unwrap();
        self.mods
            .iter()
            .filter(|tier_id| tiers[**tier_id].affix == Affix::Prefix)
            .count()
    }

    pub fn num_suffixes(&self) -> usize {
        let tiers = TIERS_HV.get().unwrap();
        self.mods
            .iter()
            .filter(|tier_id| tiers[**tier_id].affix == Affix::Suffix)
            .count()
    }

    /// Whether the item has room for a mod of the given type
    pub fn has_room(&self, affix: Affix) -> bool {
        let max_affixes = match self.rarity {
            Rarity::Normal => 0,
            Rarity::Magic => 1,
            Rarity::Rare => 3,
        };
        match affix {
            Affix::Prefix => self.num_prefixes() < max_affixes,
            Affix::Suffix => self.num_suffixes() < max_affixes,
            Affix::Corrupted => unreachable!(),
        }
    }

    /// Set of unique mod tags
    pub fn mod_tags(&self) -> HashSet<ModTag> {
        let tiers = TIERS_HV.get().unwrap();
        let mods = MODS_HV.get().unwrap();

        self.mods
            .iter()
            .fold(HashSet::new(), |mut all_tags, tier_id| {
                let tier = &tiers[*tier_id];
                let modifier = &mods[tier.mod_id];

                all_tags.extend(modifier.tags.iter().cloned());

                all_tags
            })
    }

    /// Mod familities for each tier
    pub fn mod_familities(&self) -> HashSet<ModFamily> {
        let tiers = TIERS_HV.get().unwrap();
        let mods = MODS_HV.get().unwrap();

        self.mods
            .iter()
            .map(|tier_id| {
                let tier = &tiers[*tier_id];
                let modifier = &mods[tier.mod_id];
                &modifier.family
            })
            .cloned()
            .collect()
    }

    pub fn print_item(&self) {
        println!("{self}");
    }

    /// Checks whether the current state of the item is valid
    pub fn is_valid(&self) -> bool {
        let tiers = TIERS_HV.get().unwrap();
        let mods = MODS_HV.get().unwrap();

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
            let tier = &tiers[*tier_id];

            tier.ilvl <= self.item_level
        });

        let mod_families_ok = self
            .mods
            .iter()
            .map(|tier_id| {
                let tier = &tiers[*tier_id];
                let modifier = &mods[tier.mod_id];

                &modifier.family
            })
            .collect::<HashSet<_>>()
            .len()
            == self.mods.len();

        num_mods_ok && num_affixes_ok && mod_ilvls_ok && mod_families_ok
    }
}

impl Display for ItemState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let tiers = TIERS_HV.get().unwrap();
        let mods = MODS_HV.get().unwrap();
        let stat_formatters = FORMATTERS.get().unwrap();

        writeln!(f, "{}", self.base_type)?;
        writeln!(f, "ilvl: {}", self.item_level)?;
        writeln!(f, "{:?}", self.rarity)?;
        writeln!(f, "=====================")?;
        for tier_id in &self.mods {
            let tier = &tiers[*tier_id];
            let modifier = &mods[tier.mod_id];

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
                writeln!(f, "{}", string)?;
            } else {
                // Per-stat formatters
                for (stat_id, value_range) in modifier.stats.iter().zip(tier.value_ranges.chunks(1))
                {
                    let Some(formatters) = stat_formatters.get(stat_id) else {
                        panic!("No formatter for stat: {:?}", stat_id);
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

                    writeln!(f, "{}", string)?;
                }
            }
        }

        Ok(())
    }
}

/// Get the pool of mods that could ever roll on this item, regardless of its current state
pub fn get_valid_mods_for_item(item: &ItemState) -> Vec<OpaqueIndex<Tier>> {
    let tiers = TIERS_HV.get().unwrap();
    let item_tiers = ITEM_TIERS.get().unwrap();

    item_tiers[&item.base_type]
        .iter()
        .map(|tier_id| tiers.get_opaque(tier_id))
        .filter(|&tier_id| item.item_level >= tiers[tier_id].ilvl)
        .collect()
}
