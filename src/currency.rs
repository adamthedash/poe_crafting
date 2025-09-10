use std::cell::OnceCell;
use std::collections::HashMap;
use std::sync::LazyLock;

use rand::random_range;
use random_choice::random_choice;

use crate::crafting::{filter_greater, filter_perfect};
use crate::types::{Affix, BaseItemId};
use crate::{
    MODS, TIERS,
    item_state::{ItemState, Rarity},
    types::TierId,
};

pub trait Currency {
    fn name(&self) -> &str;

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
    fn name(&self) -> &str {
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
    fn name(&self) -> &str {
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
                let modifier = &mods[&tier.mod_id];

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
    fn name(&self) -> &str {
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
    fn name(&self) -> &str {
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

        let existing_mod_families = item
            .mods
            .iter()
            .map(|tier_id| {
                let tier = &tiers[tier_id];
                &mods[&tier.mod_id].family
            })
            .collect::<Vec<_>>();

        candidate_tiers
            .iter()
            .filter(|tier_id| {
                let tier = &tiers[*tier_id];
                let modifier = &mods[&tier.mod_id];

                let has_space = modifier.affix == Affix::Prefix && num_prefixes < 3
                    || modifier.affix == Affix::Suffix && num_suffixes < 3;

                has_space && !existing_mod_families.contains(&&modifier.family)
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
    fn name(&self) -> &str {
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
    fn name(&self) -> &str {
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
    fn name(&self) -> &str {
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
    fn name(&self) -> &str {
        "Greater Chaos"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        Chaos.can_be_used(item)
    }

    fn possible_tiers(&self, _item: &ItemState, _candidate_tiers: &[TierId]) -> Vec<TierId> {
        vec![]
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        let candidate_tiers = filter_greater(candidate_tiers);
        Chaos.craft(item, &candidate_tiers);
    }
}

pub struct PerfectChaos;
impl Currency for PerfectChaos {
    fn name(&self) -> &str {
        "Perfect Chaos"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        Chaos.can_be_used(item)
    }

    fn possible_tiers(&self, _item: &ItemState, _candidate_tiers: &[TierId]) -> Vec<TierId> {
        vec![]
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        let candidate_tiers = filter_perfect(candidate_tiers);
        Chaos.craft(item, &candidate_tiers);
    }
}

pub struct GreaterExalt;
impl Currency for GreaterExalt {
    fn name(&self) -> &str {
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
    fn name(&self) -> &str {
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

pub struct GreaterTransmute;
impl Currency for GreaterTransmute {
    fn name(&self) -> &str {
        "Greater Transmute"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        Transmute.can_be_used(item)
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        let candidate_tiers = Transmute.possible_tiers(item, candidate_tiers);
        filter_greater(&candidate_tiers)
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        let candidate_tiers = self.possible_tiers(item, candidate_tiers);
        Transmute.craft(item, &candidate_tiers);
    }
}

pub struct PerfectTransmute;
impl Currency for PerfectTransmute {
    fn name(&self) -> &str {
        "Perfect Transmute"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        Transmute.can_be_used(item)
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        let candidate_tiers = Transmute.possible_tiers(item, candidate_tiers);
        filter_perfect(&candidate_tiers)
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        let candidate_tiers = self.possible_tiers(item, candidate_tiers);
        Transmute.craft(item, &candidate_tiers);
    }
}

pub struct GreaterAugmentation;
impl Currency for GreaterAugmentation {
    fn name(&self) -> &str {
        "Greater Augmentation"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        Augmentation.can_be_used(item)
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        let candidate_tiers = Augmentation.possible_tiers(item, candidate_tiers);
        filter_greater(&candidate_tiers)
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        let candidate_tiers = self.possible_tiers(item, candidate_tiers);
        Augmentation.craft(item, &candidate_tiers);
    }
}

pub struct PerfectAugmentation;
impl Currency for PerfectAugmentation {
    fn name(&self) -> &str {
        "Perfect Augmentation"
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        Augmentation.can_be_used(item)
    }

    fn possible_tiers(&self, item: &ItemState, candidate_tiers: &[TierId]) -> Vec<TierId> {
        let candidate_tiers = Augmentation.possible_tiers(item, candidate_tiers);
        filter_perfect(&candidate_tiers)
    }

    fn craft(&self, item: &mut ItemState, candidate_tiers: &[TierId]) {
        let candidate_tiers = self.possible_tiers(item, candidate_tiers);
        Augmentation.craft(item, &candidate_tiers);
    }
}

/// Lesser to Greater Essences
#[derive(Clone, Debug)]
pub struct Essence {
    name: String,
    tiers: HashMap<BaseItemId, TierId>,
}
impl Currency for Essence {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        let mods = MODS.get().unwrap();
        let tiers = TIERS.get().unwrap();

        // must be magic
        if item.rarity != Rarity::Magic {
            return false;
        }

        // base type must match
        let Some(new_tier_id) = self.tiers.get(&item.base_type) else {
            return false;
        };
        let new_tier = &tiers[new_tier_id];
        let new_mod = &mods[&new_tier.mod_id];

        // Must not have a mod of the same family already
        if item.mods.iter().any(|tier_id| {
            let tier = &tiers[tier_id];
            let modifier = &mods[&tier.mod_id];
            modifier.family == new_mod.family
        }) {
            return false;
        }

        // Must have space for the new mod
        match new_mod.affix {
            Affix::Prefix => item.num_prefixes() < 3,
            Affix::Suffix => item.num_suffixes() < 3,
            Affix::Corrupted => unreachable!(),
        }
    }

    fn possible_tiers(&self, item: &ItemState, _candidate_tiers: &[TierId]) -> Vec<TierId> {
        vec![self.tiers[&item.base_type].clone()]
    }

    fn craft(&self, item: &mut ItemState, _candidate_tiers: &[TierId]) {
        item.rarity = Rarity::Rare;
        item.mods.push(self.tiers[&item.base_type].clone());
    }
}

#[derive(Clone, Debug)]
pub struct PerfectEssence {
    name: String,
    tiers: HashMap<BaseItemId, TierId>,
}
impl Currency for PerfectEssence {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_be_used(&self, item: &ItemState) -> bool {
        let mods = MODS.get().unwrap();
        let tiers = TIERS.get().unwrap();

        // must be magic
        if item.rarity != Rarity::Rare {
            return false;
        }

        // base type must match
        let Some(new_tier_id) = self.tiers.get(&item.base_type) else {
            return false;
        };
        let new_tier = &tiers[new_tier_id];
        let new_mod = &mods[&new_tier.mod_id];

        // Must not have a mod of the same family already
        if item.mods.iter().any(|tier_id| {
            let tier = &tiers[tier_id];
            let modifier = &mods[&tier.mod_id];
            modifier.family == new_mod.family
        }) {
            return false;
        }

        true
    }

    fn possible_tiers(&self, item: &ItemState, _candidate_tiers: &[TierId]) -> Vec<TierId> {
        vec![self.tiers[&item.base_type].clone()]
    }

    fn craft(&self, item: &mut ItemState, _candidate_tiers: &[TierId]) {
        let mods = MODS.get().unwrap();
        let tiers = TIERS.get().unwrap();

        let new_tier_id = &self.tiers[&item.base_type];
        let new_tier = &tiers[new_tier_id];
        let new_mod = &mods[&new_tier.mod_id];

        let has_space = match new_mod.affix {
            Affix::Prefix => item.num_prefixes() < 3,
            Affix::Suffix => item.num_suffixes() < 3,
            Affix::Corrupted => unreachable!(),
        };

        // If there's not enough space for the mod, remove a mod with the same affix
        // Otherwise, remove a random mod
        let mut remove_candidates = (0..item.mods.len()).collect::<Vec<_>>();
        if !has_space {
            remove_candidates.retain(|index| {
                let tier_id = &item.mods[*index];
                let tier = &tiers[tier_id];
                let modifier = &mods[&tier.mod_id];
                modifier.affix == new_mod.affix
            });
        }

        let to_remove = random_range(0..item.mods.len());
        item.mods.remove(to_remove);

        // Add on the new mod
        item.mods.push(new_tier_id.clone());
    }
}

#[derive(Clone, Debug)]
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
    GreaterTransmute,
    PerfectTransmute,
    GreaterAugmentation,
    PerfectAugmentation,
    Essence(Essence),
    PerfectEssence(PerfectEssence),
}

impl Currency for CurrencyType {
    fn name(&self) -> &str {
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
            Self::GreaterTransmute => GreaterTransmute.name(),
            Self::PerfectTransmute => PerfectTransmute.name(),
            Self::GreaterAugmentation => GreaterAugmentation.name(),
            Self::PerfectAugmentation => PerfectAugmentation.name(),
            Self::Essence(essence) => essence.name(),
            Self::PerfectEssence(essence) => essence.name(),
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
            Self::GreaterTransmute => GreaterTransmute.can_be_used(item),
            Self::PerfectTransmute => PerfectTransmute.can_be_used(item),
            Self::GreaterAugmentation => GreaterAugmentation.can_be_used(item),
            Self::PerfectAugmentation => PerfectAugmentation.can_be_used(item),
            Self::Essence(essence) => essence.can_be_used(item),
            Self::PerfectEssence(essence) => essence.can_be_used(item),
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
            Self::GreaterTransmute => GreaterTransmute.possible_tiers(item, candidate_tiers),
            Self::PerfectTransmute => PerfectTransmute.possible_tiers(item, candidate_tiers),
            Self::GreaterAugmentation => GreaterAugmentation.possible_tiers(item, candidate_tiers),
            Self::PerfectAugmentation => PerfectAugmentation.possible_tiers(item, candidate_tiers),
            Self::Essence(essence) => essence.possible_tiers(item, candidate_tiers),
            Self::PerfectEssence(essence) => essence.possible_tiers(item, candidate_tiers),
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
            Self::GreaterTransmute => GreaterTransmute.craft(item, candidate_tiers),
            Self::PerfectTransmute => PerfectTransmute.craft(item, candidate_tiers),
            Self::GreaterAugmentation => GreaterAugmentation.craft(item, candidate_tiers),
            Self::PerfectAugmentation => PerfectAugmentation.craft(item, candidate_tiers),
            Self::Essence(essence) => essence.craft(item, candidate_tiers),
            Self::PerfectEssence(essence) => essence.craft(item, candidate_tiers),
        }
    }
}

pub static CURRENCIES: LazyLock<Vec<CurrencyType>> = LazyLock::new(|| {
    vec![
        CurrencyType::Transmute,
        CurrencyType::Augmentation,
        CurrencyType::Regal,
        CurrencyType::Exalt,
        CurrencyType::Annulment,
        CurrencyType::Alchemy,
        CurrencyType::Chaos,
        CurrencyType::GreaterChaos,
        CurrencyType::PerfectChaos,
        CurrencyType::GreaterExalt,
        CurrencyType::PerfectExalt,
        CurrencyType::GreaterTransmute,
        CurrencyType::PerfectTransmute,
        CurrencyType::GreaterAugmentation,
        CurrencyType::PerfectAugmentation,
        CurrencyType::Essence(Essence {
            name: "Lesser Essence of Mind".to_string(),
            tiers: {
                let mut tiers = HashMap::new();
                let bases = ["Belt", "Boots", "Gloves", "Helmet", "Ring", "Amulet"];
                for base in bases {
                    tiers.insert(base.to_string(), "IncreasedMana3".to_string());
                }

                tiers
            },
        }),
        CurrencyType::PerfectEssence(PerfectEssence {
            name: "Perfect Essence of Battle".to_string(),
            tiers: {
                let mut tiers = HashMap::new();
                let bases = ["Bow", "One Hand Mace", "Dagger", "Spear"];
                for base in bases {
                    tiers.insert(base.to_string(), "EssenceAttackSkillLevel1H1".to_string());
                }
                let bases = ["Crossbow", "Two Hand Mace", "Warstaff"];
                for base in bases {
                    tiers.insert(base.to_string(), "EssenceAttackSkillLevel2H1".to_string());
                }

                tiers
            },
        }),
    ]
});
