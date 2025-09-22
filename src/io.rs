use std::{
    fs::{self, File},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{
    ESSENCES, MODS_HV, TIERS_HV,
    currency::{CURRENCIES, Currency, CurrencyType},
    hashvec::OpaqueIndex,
    item_state::ItemState,
    strategy::Strategy,
    types::{Modifier, Tier},
};

impl Serialize for OpaqueIndex<Modifier> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mods = MODS_HV.get().unwrap();
        serializer.serialize_str(&mods[*self].group)
    }
}

impl<'de> Deserialize<'de> for OpaqueIndex<Modifier> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mods = MODS_HV.get().unwrap();
        let mod_id = String::deserialize(deserializer)?;
        let index = mods.get_opaque(&mod_id);
        Ok(index)
    }
}

impl Serialize for OpaqueIndex<Tier> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let tiers = TIERS_HV.get().unwrap();
        serializer.serialize_str(&tiers[*self].id)
    }
}

impl<'de> Deserialize<'de> for OpaqueIndex<Tier> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let tiers = TIERS_HV.get().unwrap();
        let tier_id = String::deserialize(deserializer)?;
        let index = tiers.get_opaque(&tier_id);
        Ok(index)
    }
}

impl Serialize for CurrencyType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.name())
    }
}

impl<'de> Deserialize<'de> for CurrencyType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let name = String::deserialize(deserializer)?;
        CURRENCIES
            .iter()
            .chain(ESSENCES.get().unwrap())
            .find(|c| c.name() == name)
            .cloned()
            .ok_or_else(|| serde::de::Error::custom(format!("Unknown currency: {name}")))
    }
}

#[derive(Serialize, Deserialize)]
pub struct SavedStrategy {
    pub base_item: ItemState,
    pub strategy: Strategy,
}

impl SavedStrategy {
    /// Serialise the strategy to a JSON file
    pub fn save(&self, path: &Path) {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .unwrap();
        serde_json::to_writer(&mut file, self).unwrap();
    }

    /// Load the strategy from a JSON file
    pub fn load(path: &Path) -> Self {
        let file = File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    }
}
