use crate::{
    item_state::{ItemState, Rarity},
    parser::{Affix, Modifier, Tier},
};

pub trait Currency {
    /// Whether this currency can currently be used on the given item
    fn can_be_used(item: &ItemState) -> bool;

    /// Gets the pool of mods that can roll if this currency is used.
    /// all_mods: The pool of mods that can possibly roll on this item
    fn possible_mods<'a>(
        item: &ItemState,
        all_mods: &[(&'a Modifier, Vec<&'a Tier>)],
    ) -> Vec<(&'a Modifier, Vec<&'a Tier>)>;
}

pub struct Transmute;

impl Currency for Transmute {
    /// Whether this currency can currently be used on the given item
    fn can_be_used(item: &ItemState) -> bool {
        item.rarity == Rarity::Normal
    }

    /// Gets the pool of mods that can roll if this currency is used.
    /// all_mods: The pool of mods that can possibly roll on this item
    fn possible_mods<'a>(
        _item: &ItemState,
        all_mods: &[(&'a Modifier, Vec<&'a Tier>)],
    ) -> Vec<(&'a Modifier, Vec<&'a Tier>)> {
        // TODO: Filter out non-standard mods, such as essences or abyss
        all_mods.to_vec()
    }
}

pub struct Augmentation;

impl Currency for Augmentation {
    fn can_be_used(item: &ItemState) -> bool {
        item.rarity == Rarity::Magic && item.mods.len() < 2
    }

    fn possible_mods<'a>(
        item: &ItemState,
        all_mods: &[(&'a Modifier, Vec<&'a Tier>)],
    ) -> Vec<(&'a Modifier, Vec<&'a Tier>)> {
        let mut mods = all_mods.to_vec();

        if item.num_prefixes() == 1 {
            mods.retain(|(m, _)| m.affix != Affix::Prefix);
        }
        if item.num_suffixes() == 1 {
            mods.retain(|(m, _)| m.affix != Affix::Suffix);
        }

        mods
    }
}

pub struct Regal;

impl Currency for Regal {
    fn can_be_used(item: &ItemState) -> bool {
        item.rarity == Rarity::Magic
    }

    fn possible_mods<'a>(
        item: &ItemState,
        all_mods: &[(&'a Modifier, Vec<&'a Tier>)],
    ) -> Vec<(&'a Modifier, Vec<&'a Tier>)> {
        let existing_mod_groups = item
            .mods
            .iter()
            .flat_map(|(m, _)| &m.modgroups)
            .collect::<Vec<_>>();

        all_mods
            .iter()
            // Can't roll mods from the same mod group
            .filter(|(m, _)| {
                !m.modgroups
                    .iter()
                    .any(|group| existing_mod_groups.contains(&group))
            })
            .cloned()
            .collect()
    }
}

pub struct Exalt;

impl Currency for Exalt {
    fn can_be_used(item: &ItemState) -> bool {
        item.rarity == Rarity::Rare && item.mods.len() < 6
    }

    fn possible_mods<'a>(
        item: &ItemState,
        all_mods: &[(&'a Modifier, Vec<&'a Tier>)],
    ) -> Vec<(&'a Modifier, Vec<&'a Tier>)> {
        let mut mods = all_mods.to_vec();

        if item.num_prefixes() == 3 {
            mods.retain(|(m, _)| m.affix != Affix::Prefix);
        }
        if item.num_suffixes() == 3 {
            mods.retain(|(m, _)| m.affix != Affix::Suffix);
        }

        let existing_mod_groups = item
            .mods
            .iter()
            .flat_map(|(m, _)| &m.modgroups)
            .collect::<Vec<_>>();

        mods.iter()
            // Can't roll mods from the same mod group
            .filter(|(m, _)| {
                !m.modgroups
                    .iter()
                    .any(|group| existing_mod_groups.contains(&group))
            })
            .cloned()
            .collect()
    }
}
