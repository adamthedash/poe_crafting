use std::cell::OnceCell;
use std::sync::LazyLock;

use rand::random_range;
use random_choice::random_choice;

use crate::crafting::{filter_greater, filter_perfect};
use crate::types::Affix;
use crate::{
    MODS, TIERS,
    item_state::{ItemState, Rarity},
    types::TierId,
};

pub trait Currency {
    fn name(&self) -> &'static str;

    /// Whether this currency can currently be used on the given item
    fn can_be_used(&self, item: &ItemState) -> bool;

    /// Gets the pool of mods that can roll if this currency is used.
    /// Assumes that it has been varified with Self::can_be_used
    /// candidate_tiers: The pool of mods that can possibly roll on this item
    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId>;

    /// Use this currency on the item.
    /// Assumes that it has been varified with Self::can_be_used
    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]);
}

pub struct Transmute;

impl Currency for Transmute {
    fn name(&self) -> &'static str {
        "Transmute"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        item.rarity == Rarity::Normal
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        Augmentation.possible_tiers(item, candidate_tiers)
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        Augmentation.craft(item, candidate_tiers);
        item.rarity = Rarity::Magic;
    }
}

pub struct Augmentation;

impl Currency for Augmentation {
    fn name(&self) -> &'static str {
        "Augmentation"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        item.rarity == Rarity::Magic && item.mods.len() < 2
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        // TODO: Filter out non-standard mods, such as essences or abyss
        let mods = MODS.get().unwrap();
        let tiers = TIERS.get().unwrap();

        let num_prefixes = item.num_prefixes();
        let num_suffixes = item.num_suffixes();

        candidate_tiers
            .iter()
            .filter(|tier_id| {
                let tier = &tiers[*tier_id];
                let modifier = &mods[&tier.modifier];

                modifier.affix == Affix::Prefix && num_prefixes < 1
                    || modifier.affix == Affix::Suffix && num_suffixes < 1
            })
            .cloned()
            .collect()
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        let tiers = TIERS.get().unwrap();

        let candidate_tiers = self.possible_tiers(item, candidate_tiers);

        let weights = candidate_tiers
            .iter()
            .map(|tier_id| tiers[tier_id].weight as f32)
            .collect::<Vec<_>>();

        let choice = random_choice().random_choice_f32(&candidate_tiers, &weights, 1)[0];

        item.mods.push(choice.clone());
    }
}

pub struct Regal;

impl Currency for Regal {
    fn name(&self) -> &'static str {
        "Regal"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        item.rarity == Rarity::Magic
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        Exalt.possible_tiers(item, candidate_tiers)
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        item.rarity = Rarity::Rare;
        Exalt.craft(item, candidate_tiers);
    }
}

pub struct Exalt;

impl Currency for Exalt {
    fn name(&self) -> &'static str {
        "Exalt"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        item.rarity == Rarity::Rare && item.mods.len() < 6
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        let mods = MODS.get().unwrap();
        let tiers = TIERS.get().unwrap();

        let num_prefixes = item.num_prefixes();
        let num_suffixes = item.num_suffixes();

        let existing_mod_groups = item
            .mods
            .iter()
            .map(|tier_id| {
                let tier = &tiers[tier_id];
                &mods[&tier.modifier].family
            })
            .collect::<Vec<_>>();

        candidate_tiers
            .iter()
            .filter(|tier_id| {
                let tier = &tiers[*tier_id];
                let modifier = &mods[&tier.modifier];

                let has_space = modifier.affix == Affix::Prefix && num_prefixes < 3
                    || modifier.affix == Affix::Suffix && num_suffixes < 3;

                has_space && !existing_mod_groups.contains(&&modifier.family)
            })
            .cloned()
            .collect()
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        let tiers = TIERS.get().unwrap();

        let candidate_tiers = self.possible_tiers(item, candidate_tiers);

        let weights = candidate_tiers
            .iter()
            .map(|tier_id| tiers[tier_id].weight as f32)
            .collect::<Vec<_>>();

        let choice = random_choice().random_choice_f32(&candidate_tiers, &weights, 1)[0];

        item.mods.push(choice.clone());
    }
}

pub struct Annulment;

impl Currency for Annulment {
    fn name(&self) -> &'static str {
        "Annulment"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        !item.mods.is_empty()
    }

    fn possible_tiers(&self, _item: &ItemState, _candidate_tiers: &[TierId]) -> Vec<TierId> {
        // TODO: Should this return the possible targets to remove?
        vec![]
    }

    fn craft(&self, item: &mut ItemState, _candidate_tiers: &[TierId]) {
        let to_remove = random_range(0..item.mods.len());
        item.mods.remove(to_remove);
    }
}

pub struct Alchemy;

impl Currency for Alchemy {
    fn name(&self) -> &'static str {
        "Alchemy"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        item.rarity == Rarity::Normal
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        Exalt.possible_tiers(item, candidate_tiers)
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        item.rarity = Rarity::Rare;
        for _ in 0..4 {
            Exalt.craft(item, candidate_tiers);
        }
    }
}

pub struct Chaos;

impl Currency for Chaos {
    fn name(&self) -> &'static str {
        "Chaos"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        item.rarity == Rarity::Rare && !item.mods.is_empty()
    }

    fn possible_tiers(&self, _item: &ItemState, _candidate_tiers: &[TierId]) -> Vec<TierId> {
        // TODO: Need to account for the mod being removed
        vec![]
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        // TODO: Need to understand if this logic is correct.
        //  It could be roll outcome first then roll remove
        Annulment.craft(item, candidate_tiers);
        Exalt.craft(item, candidate_tiers);
    }
}

pub struct GreaterChaos;
impl Currency for GreaterChaos {
    fn name(&self) -> &'static str {
        "Greater Chaos"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        Chaos.can_be_used(item)
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        let candidate_tiers = Chaos.possible_tiers(item, candidate_tiers);
        filter_greater(&candidate_tiers)
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        let candidate_tiers = self.possible_tiers(item, candidate_tiers);
        Chaos.craft(item, &candidate_tiers);
    }
}

pub struct PerfectChaos;
impl Currency for PerfectChaos {
    fn name(&self) -> &'static str {
        "Perfect Chaos"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        Chaos.can_be_used(item)
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        let candidate_tiers = Chaos.possible_tiers(item, candidate_tiers);
        filter_perfect(&candidate_tiers)
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        let candidate_tiers = self.possible_tiers(item, candidate_tiers);
        Chaos.craft(item, &candidate_tiers);
    }
}

pub struct GreaterExalt;
impl Currency for GreaterExalt {
    fn name(&self) -> &'static str {
        "Greater Exalt"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        Exalt.can_be_used(item)
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        let candidate_tiers = Exalt.possible_tiers(item, candidate_tiers);
        filter_greater(&candidate_tiers)
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        let candidate_tiers = self.possible_tiers(item, candidate_tiers);
        Exalt.craft(item, &candidate_tiers);
    }
}

pub struct PerfectExalt;
impl Currency for PerfectExalt {
    fn name(&self) -> &'static str {
        "Perfect Exalt"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        Exalt.can_be_used(item)
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        let candidate_tiers = Exalt.possible_tiers(item, candidate_tiers);
        filter_perfect(&candidate_tiers)
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        let candidate_tiers = self.possible_tiers(item, candidate_tiers);
        Exalt.craft(item, &candidate_tiers);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrencyType {
    Transmute,
    Augmentation,
    Regal,
    Exalt,
    Annulment,
    Alchemy,
    Chaos,
    GreaterChaos,
    PerfectChaos,
    GreaterExalt,
    PerfectExalt,
}

impl Currency for CurrencyType {
    fn name(&self) -> &'static str {
        match self {
            Self::Transmute => Transmute.name(),
            Self::Augmentation => Augmentation.name(),
            Self::Regal => Regal.name(),
            Self::Exalt => Exalt.name(),
            Self::Annulment => Annulment.name(),
            Self::Alchemy => Alchemy.name(),
            Self::Chaos => Chaos.name(),
            Self::GreaterChaos => GreaterChaos.name(),
            Self::PerfectChaos => PerfectChaos.name(),
            Self::GreaterExalt => GreaterExalt.name(),
            Self::PerfectExalt => PerfectExalt.name(),
        }
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        match self {
            Self::Transmute => Transmute.can_be_used(item),
            Self::Augmentation => Augmentation.can_be_used(item),
            Self::Regal => Regal.can_be_used(item),
            Self::Exalt => Exalt.can_be_used(item),
            Self::Annulment => Annulment.can_be_used(item),
            Self::Alchemy => Alchemy.can_be_used(item),
            Self::Chaos => Chaos.can_be_used(item),
            Self::GreaterChaos => GreaterChaos.can_be_used(item),
            Self::PerfectChaos => PerfectChaos.can_be_used(item),
            Self::GreaterExalt => GreaterExalt.can_be_used(item),
            Self::PerfectExalt => PerfectExalt.can_be_used(item),
        }
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        match self {
            Self::Transmute => Transmute.possible_tiers(item, candidate_tiers),
            Self::Augmentation => Augmentation.possible_tiers(item, candidate_tiers),
            Self::Regal => Regal.possible_tiers(item, candidate_tiers),
            Self::Exalt => Exalt.possible_tiers(item, candidate_tiers),
            Self::Annulment => Annulment.possible_tiers(item, candidate_tiers),
            Self::Alchemy => Alchemy.possible_tiers(item, candidate_tiers),
            Self::Chaos => Chaos.possible_tiers(item, candidate_tiers),
            Self::GreaterChaos => GreaterChaos.possible_tiers(item, candidate_tiers),
            Self::PerfectChaos => PerfectChaos.possible_tiers(item, candidate_tiers),
            Self::GreaterExalt => GreaterExalt.possible_tiers(item, candidate_tiers),
            Self::PerfectExalt => PerfectExalt.possible_tiers(item, candidate_tiers),
        }
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        match self {
            Self::Transmute => Transmute.craft(item, candidate_tiers),
            Self::Augmentation => Augmentation.craft(item, candidate_tiers),
            Self::Regal => Regal.craft(item, candidate_tiers),
            Self::Exalt => Exalt.craft(item, candidate_tiers),
            Self::Annulment => Annulment.craft(item, candidate_tiers),
            Self::Alchemy => Alchemy.craft(item, candidate_tiers),
            Self::Chaos => Chaos.craft(item, candidate_tiers),
            Self::GreaterChaos => GreaterChaos.craft(item, candidate_tiers),
            Self::PerfectChaos => PerfectChaos.craft(item, candidate_tiers),
            Self::GreaterExalt => GreaterExalt.craft(item, candidate_tiers),
            Self::PerfectExalt => PerfectExalt.craft(item, candidate_tiers),
        }
    }
}

impl CurrencyType {
    /// Get all available currency types
    pub const fn all() -> &'static [CurrencyType] {
        &[
            Self::Transmute,
            Self::Augmentation,
            Self::Regal,
            Self::Exalt,
            Self::Annulment,
            Self::Alchemy,
            Self::Chaos,
            Self::GreaterChaos,
            Self::PerfectChaos,
            Self::GreaterExalt,
            Self::PerfectExalt,
        ]
    }
}
pub const CURRENCIES: &[CurrencyType] = CurrencyType::all();
