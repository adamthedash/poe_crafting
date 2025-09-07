use crate::parser::{Affix, ItemBase, Modifier, Root, Tier};

#[derive(Debug, PartialEq, Eq)]
pub enum Rarity {
    Normal,
    Magic,
    Rare,
}

#[derive(Debug)]
pub struct ItemState<'a> {
    pub base_type: &'a ItemBase,
    pub item_level: u32,
    pub rarity: Rarity,
    pub mods: Vec<(&'a Modifier, &'a Tier)>,
}

impl ItemState<'_> {
    pub fn num_prefixes(&self) -> usize {
        self.mods
            .iter()
            .filter(|(m, _)| m.affix == Affix::Prefix)
            .count()
    }

    pub fn num_suffixes(&self) -> usize {
        self.mods
            .iter()
            .filter(|(m, _)| m.affix == Affix::Suffix)
            .count()
    }
}

/// Get the pool of mods that could ever roll on this item, regardless of its current state
pub fn get_valid_mods_for_item<'a>(
    item: &ItemState,
    root: &'a Root,
) -> Vec<(&'a Modifier, Vec<&'a Tier>)> {
    root.basemods[&item.base_type.id_base]
        .iter()
        .filter_map(|mod_id| {
            let tiers = root.tiers[mod_id][&item.base_type.id_base]
                .iter()
                .filter(|t| item.item_level >= t.ilvl)
                .collect::<Vec<_>>();

            if tiers.is_empty() {
                None
            } else {
                Some((&root.modifiers.seq[root.modifiers.ind[mod_id]], tiers))
            }
        })
        .collect()
}
