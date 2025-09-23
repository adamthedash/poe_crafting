use std::collections::{HashMap, HashSet};

use random_choice::random_choice;

use crate::{
    MODS, TIERS,
    crafting::{
        filter_affix, filter_better_currency, filter_lowest_tier, filter_out_families, filter_tags,
    },
    hashvec::OpaqueIndex,
    item_state::{ItemState, Rarity},
    types::{Affix, BaseItemId, Omen, Tier},
};

pub trait Currency {
    fn name(&self) -> &str;

    /// Whether this currency can currently be used on the given item
    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool;

    /// Use this currency on the item.
    /// Assumes that it has been verified with Self::can_be_used
    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    );
}

pub struct Transmute;
impl Currency for Transmute {
    fn name(&self) -> &str {
        "Transmute"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        _candidate_tiers: &[OpaqueIndex<Tier>],
        _omens: &HashSet<Omen>,
    ) -> bool {
        item.rarity == Rarity::Normal
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        _omens: &HashSet<Omen>,
    ) {
        // Transmute doesn't care about omens
        Augmentation.craft(item, candidate_tiers, &HashSet::new());
        item.rarity = Rarity::Magic;
    }
}

pub struct GreaterTransmute;
impl Currency for GreaterTransmute {
    fn name(&self) -> &str {
        "Greater Transmute"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        Transmute.can_be_used(item, candidate_tiers, omens)
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        let candidate_tiers = filter_better_currency(candidate_tiers, 55);
        Transmute.craft(item, &candidate_tiers, omens);
    }
}

pub struct PerfectTransmute;
impl Currency for PerfectTransmute {
    fn name(&self) -> &str {
        "Perfect Transmute"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        Transmute.can_be_used(item, candidate_tiers, omens)
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        let candidate_tiers = filter_better_currency(candidate_tiers, 70);
        Transmute.craft(item, &candidate_tiers, omens);
    }
}

pub struct Augmentation;
impl Currency for Augmentation {
    fn name(&self) -> &str {
        "Augmentation"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        _candidate_tiers: &[OpaqueIndex<Tier>],
        _omens: &HashSet<Omen>,
    ) -> bool {
        item.rarity == Rarity::Magic && item.mods.len() < 2
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        _omens: &HashSet<Omen>,
    ) {
        let mut candidate_tiers: Box<dyn Iterator<Item = OpaqueIndex<Tier>>> =
            Box::new(candidate_tiers.iter().copied());

        if item.num_prefixes() == 1 {
            // Filter out prefixes
            candidate_tiers = Box::new(filter_affix(candidate_tiers, Affix::Suffix))
        }
        if item.num_suffixes() == 1 {
            // Filter out suffixes
            candidate_tiers = Box::new(filter_affix(candidate_tiers, Affix::Prefix))
        }

        let candidate_tiers = candidate_tiers.collect::<Vec<_>>();

        // Roll a mod
        let weights = candidate_tiers
            .iter()
            .map(|&tier_id| TIERS[tier_id].weight as f32)
            .collect::<Vec<_>>();

        let choice = *random_choice().random_choice_f32(&candidate_tiers, &weights, 1)[0];

        item.mods.push(choice);
    }
}

pub struct GreaterAugmentation;
impl Currency for GreaterAugmentation {
    fn name(&self) -> &str {
        "Greater Augmentation"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        Augmentation.can_be_used(item, candidate_tiers, omens)
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        let candidate_tiers = filter_better_currency(candidate_tiers, 55);
        Augmentation.craft(item, &candidate_tiers, omens);
    }
}

pub struct PerfectAugmentation;
impl Currency for PerfectAugmentation {
    fn name(&self) -> &str {
        "Perfect Augmentation"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        Augmentation.can_be_used(item, candidate_tiers, omens)
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        let candidate_tiers = filter_better_currency(candidate_tiers, 70);
        Augmentation.craft(item, &candidate_tiers, omens);
    }
}

pub struct Regal;
impl Currency for Regal {
    fn name(&self) -> &str {
        "Regal"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        item.rarity == Rarity::Magic && {
            // TODO: see if we can do this check without copying
            let mut item = item.clone();
            item.rarity = Rarity::Rare;
            Exalt.can_be_used(&item, candidate_tiers, omens)
        }
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        item.rarity = Rarity::Rare;
        Exalt.craft(item, candidate_tiers, omens);
    }
}

pub struct GreaterRegal;
impl Currency for GreaterRegal {
    fn name(&self) -> &str {
        "Greater Regal"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        Regal.can_be_used(item, candidate_tiers, omens)
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        let candidate_tiers = filter_better_currency(candidate_tiers, 35);
        Regal.craft(item, &candidate_tiers, omens);
    }
}

pub struct PerfectRegal;
impl Currency for PerfectRegal {
    fn name(&self) -> &str {
        "Perfect Regal"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        Regal.can_be_used(item, candidate_tiers, omens)
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        let candidate_tiers = filter_better_currency(candidate_tiers, 50);
        Regal.craft(item, &candidate_tiers, omens);
    }
}

pub struct Exalt;
impl Currency for Exalt {
    fn name(&self) -> &str {
        "Exalt"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        item.rarity == Rarity::Rare && item.mods.len() < 6 && {
            // Omens
            let mut candidate_tiers: Box<dyn Iterator<Item = OpaqueIndex<Tier>>> =
                Box::new(candidate_tiers.iter().copied());

            if omens.contains(&Omen::Dextral) {
                // filter suffixes
                candidate_tiers = Box::new(filter_affix(candidate_tiers, Affix::Suffix));
            }
            if omens.contains(&Omen::Sinistral) {
                // filter prefixes
                candidate_tiers = Box::new(filter_affix(candidate_tiers, Affix::Prefix));
            }
            if omens.contains(&Omen::Homogenous) {
                // filter tags
                let existing_tags = item.mod_tags();
                // If there are no tags, homogenizing has no effect
                if !existing_tags.is_empty() {
                    candidate_tiers = Box::new(filter_tags(candidate_tiers, existing_tags));
                }
            }

            // Filter out based on current item state
            let candidate_tiers = self.filter_slammable(item, &candidate_tiers.collect::<Vec<_>>());
            if candidate_tiers.is_empty() {
                return false;
            }

            // Which affixes can be slammed
            if omens.contains(&Omen::Greater) {
                let unique_affixes = candidate_tiers
                    .iter()
                    .map(|&tier_id| {
                        let tier = &TIERS[tier_id];

                        tier.affix
                    })
                    .collect::<HashSet<_>>();

                if unique_affixes.len() == 1 {
                    if unique_affixes.contains(&Affix::Suffix) {
                        item.num_suffixes() <= 1
                    } else {
                        item.num_prefixes() <= 1
                    }
                } else {
                    item.mods.len() <= 4
                }
            } else {
                // By now we've checked if anything is slammable for 1 mod
                true
            }
        }
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        let mut candidate_tiers: Box<dyn Iterator<Item = OpaqueIndex<Tier>>> =
            Box::new(candidate_tiers.iter().copied());

        // Apply omens
        if omens.contains(&Omen::Dextral) {
            // filter suffixes
            candidate_tiers = Box::new(filter_affix(candidate_tiers, Affix::Suffix));
        }
        if omens.contains(&Omen::Sinistral) {
            // filter prefixes
            candidate_tiers = Box::new(filter_affix(candidate_tiers, Affix::Prefix));
        }
        if omens.contains(&Omen::Homogenous) {
            // filter tags
            let existing_tags = item.mod_tags();
            // If there are no tags, homogenizing has no effect
            if !existing_tags.is_empty() {
                candidate_tiers = Box::new(filter_tags(candidate_tiers, existing_tags));
            }
        }

        let candidate_tiers = candidate_tiers.collect::<Vec<_>>();

        // TODO: Check validity of 2nd slam
        let num_slams = if omens.contains(&Omen::Greater) { 2 } else { 1 };
        for _ in 0..num_slams {
            let candidate_tiers = self.filter_slammable(item, &candidate_tiers);

            let weights = candidate_tiers
                .iter()
                .map(|&tier_id| TIERS[tier_id].weight as f32)
                .collect::<Vec<_>>();

            let choice = *random_choice()
                .random_choice_f32(&candidate_tiers, &weights, 1)
                .first()
                .unwrap_or_else(|| {
                    item.print_item();
                    panic!(
                        "No canidates to slam! omens: {:?}, {:?}",
                        omens,
                        std::any::type_name::<Self>()
                    );
                });

            item.mods.push(*choice);
        }
    }
}

impl Exalt {
    fn filter_slammable(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
    ) -> Vec<OpaqueIndex<Tier>> {
        let mut candidate_tiers: Box<dyn Iterator<Item = OpaqueIndex<Tier>>> =
            Box::new(candidate_tiers.iter().copied());

        candidate_tiers = Box::new(filter_out_families(candidate_tiers, item.mod_familities()));

        if item.num_prefixes() == 3 {
            candidate_tiers = Box::new(filter_affix(candidate_tiers, Affix::Suffix));
        }

        if item.num_suffixes() == 3 {
            candidate_tiers = Box::new(filter_affix(candidate_tiers, Affix::Prefix));
        }

        candidate_tiers.collect()
    }
}

pub struct GreaterExalt;
impl Currency for GreaterExalt {
    fn name(&self) -> &str {
        "Greater Exalt"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        Exalt.can_be_used(item, candidate_tiers, omens)
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        let candidate_tiers = filter_better_currency(candidate_tiers, 35);
        Exalt.craft(item, &candidate_tiers, omens);
    }
}

pub struct PerfectExalt;
impl Currency for PerfectExalt {
    fn name(&self) -> &str {
        "Perfect Exalt"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        Exalt.can_be_used(item, candidate_tiers, omens)
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        let candidate_tiers = filter_better_currency(candidate_tiers, 50);
        Exalt.craft(item, &candidate_tiers, omens);
    }
}

pub struct Annulment;
impl Currency for Annulment {
    fn name(&self) -> &str {
        "Annulment"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        _candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        !item.mods.is_empty() && {
            // Omens
            let mut candidate_removes: Box<dyn Iterator<Item = OpaqueIndex<Tier>>> =
                Box::new(item.mods.iter().copied());

            // Apply omens
            if omens.contains(&Omen::Dextral) {
                // filter suffixes
                candidate_removes = Box::new(filter_affix(candidate_removes, Affix::Suffix));
            }
            if omens.contains(&Omen::Sinistral) {
                // filter prefixes
                candidate_removes = Box::new(filter_affix(candidate_removes, Affix::Prefix));
            }
            // For chaos orb
            if omens.contains(&Omen::Whittling) {
                // filter lowest tier
                candidate_removes = Box::new(filter_lowest_tier(candidate_removes));
            }
            let num_removes = if omens.contains(&Omen::Greater) { 2 } else { 1 };

            candidate_removes.count() >= num_removes
        }
    }

    fn craft(
        &self,
        item: &mut ItemState,
        _candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        // Omens
        let mut candidate_removes: Box<dyn Iterator<Item = OpaqueIndex<Tier>>> =
            Box::new(item.mods.iter().copied());

        // Apply omens
        if omens.contains(&Omen::Dextral) {
            // filter suffixes
            candidate_removes = Box::new(filter_affix(candidate_removes, Affix::Suffix));
        }
        if omens.contains(&Omen::Sinistral) {
            // filter prefixes
            candidate_removes = Box::new(filter_affix(candidate_removes, Affix::Prefix));
        }
        // For chaos orb
        if omens.contains(&Omen::Whittling) {
            // filter lowest tier
            candidate_removes = Box::new(filter_lowest_tier(candidate_removes));
        }

        let mut candidate_removes = candidate_removes.collect::<Vec<_>>();

        // TODO: Check validity of 2nd remove
        let num_removes = if omens.contains(&Omen::Greater) { 2 } else { 1 };
        for _ in 0..num_removes {
            let weights = vec![1.; candidate_removes.len()];
            let to_remove = *(*random_choice()
                .random_choice_f32(&candidate_removes, &weights, 1)
                .first()
                .expect("No candidates to remove!"));

            item.mods.retain(|tier_id| *tier_id != to_remove);
            candidate_removes.retain(|tier_id| *tier_id != to_remove);
        }
    }
}

pub struct Alchemy;
impl Currency for Alchemy {
    fn name(&self) -> &str {
        "Alchemy"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        _candidate_tiers: &[OpaqueIndex<Tier>],
        _omens: &HashSet<Omen>,
    ) -> bool {
        item.rarity == Rarity::Normal
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        item.rarity = Rarity::Rare;

        let omens = if omens.contains(&Omen::Dextral) {
            HashSet::from_iter(std::iter::once(Omen::Dextral))
        } else if omens.contains(&Omen::Sinistral) {
            HashSet::from_iter(std::iter::once(Omen::Sinistral))
        } else {
            HashSet::new()
        };

        for _ in 0..3 {
            Exalt.craft(item, candidate_tiers, &omens);
        }
        Exalt.craft(item, candidate_tiers, &HashSet::new());
    }
}

pub struct Chaos;
impl Currency for Chaos {
    fn name(&self) -> &str {
        "Chaos"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        item.rarity == Rarity::Rare && Annulment.can_be_used(item, candidate_tiers, omens)
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        Annulment.craft(item, candidate_tiers, omens);
        Exalt.craft(item, candidate_tiers, &HashSet::new());
    }
}

pub struct GreaterChaos;
impl Currency for GreaterChaos {
    fn name(&self) -> &str {
        "Greater Chaos"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        Chaos.can_be_used(item, candidate_tiers, omens)
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        let candidate_tiers = filter_better_currency(candidate_tiers, 35);
        Chaos.craft(item, &candidate_tiers, omens);
    }
}

pub struct PerfectChaos;
impl Currency for PerfectChaos {
    fn name(&self) -> &str {
        "Perfect Chaos"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        Chaos.can_be_used(item, candidate_tiers, omens)
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        let candidate_tiers = filter_better_currency(candidate_tiers, 50);
        Chaos.craft(item, &candidate_tiers, omens);
    }
}

/// Lesser to Greater Essences
#[derive(Clone, Debug)]
pub struct Essence {
    pub name: String,
    pub tiers: HashMap<BaseItemId, Vec<OpaqueIndex<Tier>>>,
}
impl Currency for Essence {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        _candidate_tiers: &[OpaqueIndex<Tier>],
        _omens: &HashSet<Omen>,
    ) -> bool {
        // must be magic
        if item.rarity != Rarity::Magic {
            return false;
        }

        // base type must match
        let Some(new_tier_ids) = self.tiers.get(&item.base_type) else {
            return false;
        };
        let new_tiers = new_tier_ids
            .iter()
            .map(|&tier_id| &TIERS[tier_id])
            .collect::<Vec<_>>();

        // Must not have a mod of the same family already
        // Assumption: all mods added have the same family
        let new_mod_family = &MODS[new_tiers.first().unwrap().mod_id].family;
        if item.mod_familities().contains(new_mod_family) {
            return false;
        }

        // Must have space for the new mod
        let new_mod_affixes = new_tiers.iter().map(|tier| tier.affix).collect::<Vec<_>>();
        match (
            new_mod_affixes.contains(&Affix::Prefix),
            new_mod_affixes.contains(&Affix::Suffix),
        ) {
            // Can add either, so doesn't matter which is removed
            (true, true) => true,
            // Adds one affix, so space must be made if full
            (true, false) => item.num_prefixes() < 3,
            (false, true) => item.num_suffixes() < 3,
            (false, false) => unreachable!(),
        }
    }

    fn craft(
        &self,
        item: &mut ItemState,
        _candidate_tiers: &[OpaqueIndex<Tier>],
        _omens: &HashSet<Omen>,
    ) {
        item.rarity = Rarity::Rare;
        Exalt.craft(item, &self.tiers[&item.base_type], &HashSet::new());
    }
}

impl PartialEq for Essence {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Clone, Debug)]
pub struct PerfectEssence {
    pub name: String,
    pub tiers: HashMap<BaseItemId, Vec<OpaqueIndex<Tier>>>,
}
impl Currency for PerfectEssence {
    fn name(&self) -> &str {
        &self.name
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        _candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        // must be rare
        if item.rarity != Rarity::Rare {
            return false;
        }

        // base type must match
        let Some(new_tier_ids) = self.tiers.get(&item.base_type) else {
            return false;
        };
        let new_tiers = new_tier_ids
            .iter()
            .map(|&tier_id| &TIERS[tier_id])
            .collect::<Vec<_>>();

        // Must not have a mod of the same family already
        // Assumption: all mods added have the same family
        let new_mod_family = &MODS[new_tiers.first().unwrap().mod_id].family;
        if item.mod_familities().contains(new_mod_family) {
            return false;
        }

        // Must have room for it
        // If there's not enough space for the mod, remove a mod with the same affix
        // Otherwise, remove a random mod
        let mut candidate_removes: Box<dyn Iterator<Item = OpaqueIndex<Tier>>> =
            Box::new(item.mods.iter().copied());

        let new_mod_affixes = new_tiers.iter().map(|tier| tier.affix).collect::<Vec<_>>();
        let need_remove_affix = match (
            new_mod_affixes.contains(&Affix::Prefix),
            new_mod_affixes.contains(&Affix::Suffix),
        ) {
            // Can add either, so doesn't matter which is removed
            (true, true) => None,
            // Adds one affix, so space must be made if full
            (true, false) => (item.num_prefixes() == 3).then_some(Affix::Prefix),
            (false, true) => (item.num_suffixes() == 3).then_some(Affix::Suffix),
            (false, false) => unreachable!(),
        };
        if let Some(affix) = need_remove_affix {
            // filter same affix as essence adds
            candidate_removes = Box::new(filter_affix(candidate_removes, affix));
        }

        // Apply omens
        if omens.contains(&Omen::Dextral) {
            // filter suffixes
            candidate_removes = Box::new(filter_affix(candidate_removes, Affix::Suffix));
        }
        if omens.contains(&Omen::Sinistral) {
            // filter prefixes
            candidate_removes = Box::new(filter_affix(candidate_removes, Affix::Prefix));
        }

        candidate_removes.count() > 0
    }

    fn craft(
        &self,
        item: &mut ItemState,
        _candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        let new_tier_ids = &self.tiers[&item.base_type];
        let new_tiers = new_tier_ids
            .iter()
            .map(|&tier_id| &TIERS[tier_id])
            .collect::<Vec<_>>();

        // Must have room for it
        // If there's not enough space for the mod, remove a mod with the same affix
        // Otherwise, remove a random mod
        let mut candidate_removes: Box<dyn Iterator<Item = OpaqueIndex<Tier>>> =
            Box::new(item.mods.iter().copied());

        let new_mod_affixes = new_tiers.iter().map(|tier| tier.affix).collect::<Vec<_>>();
        let need_remove_affix = match (
            new_mod_affixes.contains(&Affix::Prefix),
            new_mod_affixes.contains(&Affix::Suffix),
        ) {
            // Can add either, so doesn't matter which is removed
            (true, true) => None,
            // Adds one affix, so space must be made if full
            (true, false) => (item.num_prefixes() == 3).then_some(Affix::Prefix),
            (false, true) => (item.num_suffixes() == 3).then_some(Affix::Suffix),
            (false, false) => unreachable!(),
        };
        if let Some(affix) = need_remove_affix {
            // filter same affix as essence adds
            candidate_removes = Box::new(filter_affix(candidate_removes, affix));
        }

        // Apply omens
        if omens.contains(&Omen::Dextral) {
            // filter suffixes
            candidate_removes = Box::new(filter_affix(candidate_removes, Affix::Suffix));
        }
        if omens.contains(&Omen::Sinistral) {
            // filter prefixes
            candidate_removes = Box::new(filter_affix(candidate_removes, Affix::Prefix));
        }

        let candidate_removes = candidate_removes.collect::<Vec<_>>();

        // Remove a mod
        let weights = vec![1.; candidate_removes.len()];
        let to_remove = **random_choice()
            .random_choice_f32(&candidate_removes, &weights, 1)
            .first()
            .expect("No candidates to remove!");

        item.mods.retain(|tier_id| *tier_id != to_remove);

        // Add on a new mod
        // TODO: Check if this plays well with weights for essence mods
        Exalt.craft(item, new_tier_ids, &HashSet::new());
    }
}

impl PartialEq for PerfectEssence {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

pub struct Desecrate;
impl Currency for Desecrate {
    fn name(&self) -> &str {
        "Descrate Item"
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        _candidate_tiers: &[OpaqueIndex<Tier>],
        _omens: &HashSet<Omen>,
    ) -> bool {
        // TOOD: Check for existing veiled/desecrated mod
        item.rarity == Rarity::Rare
    }

    fn craft(
        &self,
        item: &mut ItemState,
        _candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        // TODO: Clean up this monstrosity
        let mut remove = 0; // 0 == neither, 1 == prefix, 2 == suffix, 3 == either

        if omens.contains(&Omen::Sinistral) && !item.has_room(Affix::Prefix) {
            remove = 1;
        } else if omens.contains(&Omen::Dextral) && !item.has_room(Affix::Suffix) {
            remove = 2;
        } else if item.mods.len() == 6 {
            remove = 3;
        }

        let mut annul_hm = HashSet::new();
        match remove {
            1 => {
                annul_hm.insert(Omen::Sinistral);
            }
            2 => {
                annul_hm.insert(Omen::Dextral);
            }
            _ => (),
        }
        if remove != 0 {
            Annulment.craft(item, &[], &annul_hm);
        }

        let mut candidate_tiers: Box<dyn Iterator<Item = OpaqueIndex<Tier>>> =
            Box::new([TIERS.opaque("VeiledPrefix"), TIERS.opaque("VeiledSuffix")].into_iter());

        // Apply omens
        if omens.contains(&Omen::Dextral) || !item.has_room(Affix::Prefix) {
            // filter suffixes
            candidate_tiers = Box::new(filter_affix(candidate_tiers, Affix::Suffix));
        }
        if omens.contains(&Omen::Sinistral) || !item.has_room(Affix::Suffix) {
            // filter prefixes
            candidate_tiers = Box::new(filter_affix(candidate_tiers, Affix::Prefix));
        }

        let candidate_tiers = candidate_tiers.collect::<Vec<_>>();

        let weights = vec![1.; candidate_tiers.len()];
        let choice = *random_choice().random_choice_f32(&candidate_tiers, &weights, 1)[0];

        item.mods.push(choice);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum CurrencyType {
    Transmute,
    GreaterTransmute,
    PerfectTransmute,
    Augmentation,
    GreaterAugmentation,
    PerfectAugmentation,
    Regal,
    GreaterRegal,
    PerfectRegal,
    Exalt,
    GreaterExalt,
    PerfectExalt,
    Annulment,
    Alchemy,
    Chaos,
    GreaterChaos,
    PerfectChaos,
    Essence(Essence),
    PerfectEssence(PerfectEssence),
    Desecrate,
}

impl Currency for CurrencyType {
    fn name(&self) -> &str {
        match self {
            Self::Transmute => Transmute.name(),
            Self::GreaterTransmute => GreaterTransmute.name(),
            Self::PerfectTransmute => PerfectTransmute.name(),
            Self::Augmentation => Augmentation.name(),
            Self::GreaterAugmentation => GreaterAugmentation.name(),
            Self::PerfectAugmentation => PerfectAugmentation.name(),
            Self::Regal => Regal.name(),
            Self::GreaterRegal => GreaterRegal.name(),
            Self::PerfectRegal => PerfectRegal.name(),
            Self::Exalt => Exalt.name(),
            Self::GreaterExalt => GreaterExalt.name(),
            Self::PerfectExalt => PerfectExalt.name(),
            Self::Annulment => Annulment.name(),
            Self::Alchemy => Alchemy.name(),
            Self::Chaos => Chaos.name(),
            Self::GreaterChaos => GreaterChaos.name(),
            Self::PerfectChaos => PerfectChaos.name(),
            Self::Essence(essence) => essence.name(),
            Self::PerfectEssence(essence) => essence.name(),
            Self::Desecrate => Desecrate.name(),
        }
    }

    fn can_be_used(
        &self,
        item: &ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) -> bool {
        match self {
            Self::Transmute => Transmute.can_be_used(item, candidate_tiers, omens),
            Self::GreaterTransmute => GreaterTransmute.can_be_used(item, candidate_tiers, omens),
            Self::PerfectTransmute => PerfectTransmute.can_be_used(item, candidate_tiers, omens),
            Self::Augmentation => Augmentation.can_be_used(item, candidate_tiers, omens),
            Self::GreaterAugmentation => {
                GreaterAugmentation.can_be_used(item, candidate_tiers, omens)
            }
            Self::PerfectAugmentation => {
                PerfectAugmentation.can_be_used(item, candidate_tiers, omens)
            }
            Self::Regal => Regal.can_be_used(item, candidate_tiers, omens),
            Self::GreaterRegal => GreaterRegal.can_be_used(item, candidate_tiers, omens),
            Self::PerfectRegal => PerfectRegal.can_be_used(item, candidate_tiers, omens),
            Self::Exalt => Exalt.can_be_used(item, candidate_tiers, omens),
            Self::GreaterExalt => GreaterExalt.can_be_used(item, candidate_tiers, omens),
            Self::PerfectExalt => PerfectExalt.can_be_used(item, candidate_tiers, omens),
            Self::Annulment => Annulment.can_be_used(item, candidate_tiers, omens),
            Self::Alchemy => Alchemy.can_be_used(item, candidate_tiers, omens),
            Self::Chaos => Chaos.can_be_used(item, candidate_tiers, omens),
            Self::GreaterChaos => GreaterChaos.can_be_used(item, candidate_tiers, omens),
            Self::PerfectChaos => PerfectChaos.can_be_used(item, candidate_tiers, omens),
            Self::Essence(essence) => essence.can_be_used(item, candidate_tiers, omens),
            Self::PerfectEssence(essence) => essence.can_be_used(item, candidate_tiers, omens),
            Self::Desecrate => Desecrate.can_be_used(item, candidate_tiers, omens),
        }
    }

    fn craft(
        &self,
        item: &mut ItemState,
        candidate_tiers: &[OpaqueIndex<Tier>],
        omens: &HashSet<Omen>,
    ) {
        match self {
            Self::Transmute => Transmute.craft(item, candidate_tiers, omens),
            Self::GreaterTransmute => GreaterTransmute.craft(item, candidate_tiers, omens),
            Self::PerfectTransmute => PerfectTransmute.craft(item, candidate_tiers, omens),
            Self::Augmentation => Augmentation.craft(item, candidate_tiers, omens),
            Self::GreaterAugmentation => GreaterAugmentation.craft(item, candidate_tiers, omens),
            Self::PerfectAugmentation => PerfectAugmentation.craft(item, candidate_tiers, omens),
            Self::Regal => Regal.craft(item, candidate_tiers, omens),
            Self::GreaterRegal => GreaterRegal.craft(item, candidate_tiers, omens),
            Self::PerfectRegal => PerfectRegal.craft(item, candidate_tiers, omens),
            Self::Exalt => Exalt.craft(item, candidate_tiers, omens),
            Self::GreaterExalt => GreaterExalt.craft(item, candidate_tiers, omens),
            Self::PerfectExalt => PerfectExalt.craft(item, candidate_tiers, omens),
            Self::Annulment => Annulment.craft(item, candidate_tiers, omens),
            Self::Alchemy => Alchemy.craft(item, candidate_tiers, omens),
            Self::Chaos => Chaos.craft(item, candidate_tiers, omens),
            Self::GreaterChaos => GreaterChaos.craft(item, candidate_tiers, omens),
            Self::PerfectChaos => PerfectChaos.craft(item, candidate_tiers, omens),
            Self::Essence(essence) => essence.craft(item, candidate_tiers, omens),
            Self::PerfectEssence(essence) => essence.craft(item, candidate_tiers, omens),
            Self::Desecrate => Desecrate.craft(item, candidate_tiers, omens),
        }
    }
}

impl CurrencyType {
    /// Get the list of omens that can be used with this currency type
    pub fn possible_omens(&self) -> HashSet<Omen> {
        use CurrencyType::*;
        use Omen::*;
        let omens = match self {
            Regal | GreaterRegal | PerfectRegal => vec![Homogenous],
            Annulment => vec![Sinistral, Dextral, Greater],
            Alchemy | PerfectEssence(_) | Desecrate => vec![Sinistral, Dextral],
            Chaos | GreaterChaos | PerfectChaos => vec![Sinistral, Dextral, Whittling],
            Exalt | GreaterExalt | PerfectExalt => {
                vec![Sinistral, Dextral, Homogenous, Greater]
            }
            _ => vec![],
        };

        HashSet::from_iter(omens)
    }

    pub const fn all() -> &'static [Self] {
        &[
            Self::Transmute,
            Self::GreaterTransmute,
            Self::PerfectTransmute,
            Self::Augmentation,
            Self::GreaterAugmentation,
            Self::PerfectAugmentation,
            Self::Regal,
            Self::GreaterRegal,
            Self::PerfectRegal,
            Self::Exalt,
            Self::GreaterExalt,
            Self::PerfectExalt,
            Self::Annulment,
            Self::Alchemy,
            Self::Chaos,
            Self::GreaterChaos,
            Self::PerfectChaos,
            Self::Desecrate,
        ]
    }
}
