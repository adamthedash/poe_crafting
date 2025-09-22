use serde::{Deserialize, Serialize, de::value::StringDeserializer};

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
            .find(|c| c.name() == &name)
            .cloned()
            .ok_or_else(|| serde::de::Error::custom(format!("Unknown currency: {name}")))
    }
}

#[derive(Serialize, Deserialize)]
pub struct SavedStrategy {
    base_item: ItemState,
    strategy: Strategy,
}
