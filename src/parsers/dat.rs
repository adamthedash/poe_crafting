#![allow(non_snake_case)]
use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};

use crate::{
    currency::{CurrencyType, Essence, PerfectEssence},
    hashvec::HashVec,
    internal::TIERS,
    types::{Affix, ModGroup, ModType, Modifier, StatID, Tier, TierId},
};

/// Deserialise any json-encoded value
fn deserialize_json_encoded<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

fn serialize_json_encoded<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    let s = serde_json::to_string(value).unwrap();
    serializer.serialize_str(&s)
}

/// Helper trait to make CSV record loading easier
pub trait RecordLoader: DeserializeOwned {
    /// Load the provided CSV into a structured iterator
    fn load_from_path(path: &Path) -> impl Iterator<Item = Self> {
        csv::Reader::from_path(path)
            .unwrap()
            .into_deserialize::<Self>()
            .map(|row| row.expect("Failed to deserialise row"))
    }

    fn load_from_bytes(bytes: &[u8]) -> impl Iterator<Item = Self> {
        csv::Reader::from_reader(bytes)
            .into_deserialize::<Self>()
            .map(|row| row.expect("Failed to deserialise row"))
    }
}

impl<T: DeserializeOwned> RecordLoader for T {}

pub trait RecordSaver {
    fn save_to_csv(&self, path: &Path);
}

impl<T: Serialize> RecordSaver for Vec<T> {
    fn save_to_csv(&self, path: &Path) {
        let mut writer =
            csv::Writer::from_path(path).unwrap_or_else(|_| panic!("Failed to write: {:?}", path));
        for record in self {
            writer.serialize(record).unwrap();
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct ModsRecord {
    /// Eg. Strength6
    pub Id: TierId,
    /// Index into ModType table
    pub ModType: usize,
    pub Domain: usize,

    #[serde(
        deserialize_with = "deserialize_json_encoded",
        serialize_with = "serialize_json_encoded"
    )]
    pub Families: Vec<usize>,
    // Mod Tags
    #[serde(
        deserialize_with = "deserialize_json_encoded",
        serialize_with = "serialize_json_encoded"
    )]
    pub ImplicitTags: Vec<usize>,
    /// Affix
    pub GenerationType: u32,
    /// Min ilvl
    pub Level: u32,
    /// Eg. of the Mongoose
    pub Name: String,
    /// Index into Stats table
    pub Stat1: Option<usize>,
    pub Stat2: Option<usize>,
    pub Stat3: Option<usize>,
    pub Stat4: Option<usize>,
    // /// Stat value ranges
    #[serde(
        deserialize_with = "deserialize_json_encoded",
        serialize_with = "serialize_json_encoded"
    )]
    pub Stat1Value: [i32; 2],
    #[serde(
        deserialize_with = "deserialize_json_encoded",
        serialize_with = "serialize_json_encoded"
    )]
    pub Stat2Value: [i32; 2],
    #[serde(
        deserialize_with = "deserialize_json_encoded",
        serialize_with = "serialize_json_encoded"
    )]
    pub Stat3Value: [i32; 2],
    #[serde(
        deserialize_with = "deserialize_json_encoded",
        serialize_with = "serialize_json_encoded"
    )]
    pub Stat4Value: [i32; 2],
}

#[derive(Deserialize, Serialize)]
pub struct ModTypeRecord {
    pub Name: ModGroup,
}

#[derive(Deserialize, Serialize)]
pub struct ModFamilyRecord {
    pub Id: ModGroup,
}

#[derive(Deserialize, Serialize)]
pub struct StatRecord {
    pub Id: StatID,
}

#[derive(Deserialize, Serialize)]
pub struct BaseItemTypesRecord {
    pub Name: String,
    pub ItemClass: usize,
    pub ModDomain: usize,
}

#[derive(Deserialize, Serialize)]
pub struct ItemClassesRecord {
    pub Id: String,
}

#[derive(Deserialize, Serialize)]
pub struct TagsRecord {
    pub Id: String,
    pub DisplayString: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct EssencesRecord {
    BaseItemType: usize,
}

#[derive(Deserialize, Serialize)]
pub struct EssenceTargetItemCategoriesRecord {
    #[serde(
        deserialize_with = "deserialize_json_encoded",
        serialize_with = "serialize_json_encoded"
    )]
    ItemClasses: Vec<usize>,
}

#[derive(Deserialize, Serialize)]
pub struct EssenceModsRecord {
    pub Essence: usize,
    pub TargetItemCategory: usize,
    pub Mod1: Option<usize>,
    /// The mod that is displayed on the essence item, not what is actually rolled on use
    /// Used for essences that can give many outcomes.
    /// Eg. Mark of the Abyssal Lord (Prefix or Suffix)
    /// Eg. +# to Strength, Dexterity or Intelligence
    pub DisplayMod: Option<usize>,
    /// The possible outcome mods for multi-outcome essences
    #[serde(
        deserialize_with = "deserialize_json_encoded",
        serialize_with = "serialize_json_encoded"
    )]
    pub OutcomeMods: Vec<usize>,
}

pub struct Dats {
    pub mods: Vec<ModsRecord>,
    pub mod_type: Vec<ModTypeRecord>,
    pub mod_family: Vec<ModFamilyRecord>,
    pub stats: Vec<StatRecord>,
    pub base_item_types: Vec<BaseItemTypesRecord>,
    pub item_classes: Vec<ItemClassesRecord>,
    pub tags: Vec<TagsRecord>,
    pub essences: Vec<EssencesRecord>,
    pub essence_target_item_categories: Vec<EssenceTargetItemCategoriesRecord>,
    pub essence_mods: Vec<EssenceModsRecord>,
}

impl Dats {
    pub fn load_tables(data_root: &Path) -> Self {
        Self {
            mods: ModsRecord::load_from_path(&data_root.join("data/mods.csv")).collect(),
            mod_type: ModTypeRecord::load_from_path(&data_root.join("data/modtype.csv")).collect(),
            mod_family: ModFamilyRecord::load_from_path(&data_root.join("data/modfamily.csv"))
                .collect(),
            stats: StatRecord::load_from_path(&data_root.join("data/stats.csv")).collect(),
            base_item_types: BaseItemTypesRecord::load_from_path(
                &data_root.join("data/baseitemtypes.csv"),
            )
            .collect(),
            item_classes: ItemClassesRecord::load_from_path(
                &data_root.join("data/itemclasses.csv"),
            )
            .collect(),
            tags: TagsRecord::load_from_path(&data_root.join("data/tags.csv")).collect(),
            essences: EssencesRecord::load_from_path(&data_root.join("data/essences.csv"))
                .collect(),
            essence_target_item_categories: EssenceTargetItemCategoriesRecord::load_from_path(
                &data_root.join("data/essencetargetitemcategories.csv"),
            )
            .collect(),
            essence_mods: EssenceModsRecord::load_from_path(
                &data_root.join("data/essencemods.csv"),
            )
            .collect(),
        }
    }

    pub fn save_to_csv(&self, root: &Path) {
        self.mods.save_to_csv(&root.join("data/mods.csv"));
        self.mod_type.save_to_csv(&root.join("data/modtype.csv"));
        self.mod_family.save_to_csv(&root.join("data/modfamily.csv"));
        self.stats.save_to_csv(&root.join("data/stats.csv"));
        self.base_item_types
            .save_to_csv(&root.join("data/baseitemtypes.csv"));
        self.item_classes.save_to_csv(&root.join("data/itemclasses.csv"));
        self.tags.save_to_csv(&root.join("data/tags.csv"));
        self.essences.save_to_csv(&root.join("data/essences.csv"));
        self.essence_target_item_categories
            .save_to_csv(&root.join("data/essencetargetitemcategories.csv"));
        self.essence_mods.save_to_csv(&root.join("data/essencemods.csv"));
    }

    pub fn load_tables_embedded() -> Self {
        Self {
            mods: ModsRecord::load_from_bytes(include_bytes!("../../data/tables/data/mods.csv"))
                .collect(),
            mod_type: ModTypeRecord::load_from_bytes(include_bytes!(
                "../../data/tables/data/modtype.csv"
            ))
            .collect(),
            mod_family: ModFamilyRecord::load_from_bytes(include_bytes!(
                "../../data/tables/data/modfamily.csv"
            ))
            .collect(),
            stats: StatRecord::load_from_bytes(include_bytes!("../../data/tables/data/stats.csv"))
                .collect(),
            base_item_types: BaseItemTypesRecord::load_from_bytes(include_bytes!(
                "../../data/tables/data/baseitemtypes.csv"
            ))
            .collect(),
            item_classes: ItemClassesRecord::load_from_bytes(include_bytes!(
                "../../data/tables/data/itemclasses.csv"
            ))
            .collect(),
            tags: TagsRecord::load_from_bytes(include_bytes!("../../data/tables/data/tags.csv"))
                .collect(),
            essences: EssencesRecord::load_from_bytes(include_bytes!(
                "../../data/tables/data/essences.csv"
            ))
            .collect(),
            essence_target_item_categories: EssenceTargetItemCategoriesRecord::load_from_bytes(
                include_bytes!("../../data/tables/data/essencetargetitemcategories.csv"),
            )
            .collect(),
            essence_mods: EssenceModsRecord::load_from_bytes(include_bytes!(
                "../../data/tables/data/essencemods.csv"
            ))
            .collect(),
        }
    }
}

pub fn load_essences(dats: &Dats) -> Vec<CurrencyType> {
    // Essence -> CoarseBaseId -> [ModId]
    let mut essence_base_mods = HashMap::<_, HashMap<_, _>>::new();
    dats.essence_mods.iter().for_each(|row| {
        // essencemods.TargetItemCategory -> essencetargetitemcategories.ItemClasses ->
        //      itemclasses.Id -(kinda)-> BaseItemId
        let target_base_items = dats.essence_target_item_categories[row.TargetItemCategory]
            .ItemClasses
            .iter()
            .map(|target_item_class| dats.item_classes[*target_item_class].Id.clone())
            .collect::<Vec<_>>();

        // essencemods.Mod1 -> mods
        // essencemods.OutcomeMods -> mods
        let mods = if let Some(mod_index) = row.Mod1 {
            // Single outcome
            vec![TIERS.opaque(&dats.mods[mod_index].Id)]
        } else {
            // Multiple outcomes
            row.OutcomeMods
                .iter()
                .map(|&mod_index| TIERS.opaque(&dats.mods[mod_index].Id))
                .collect()
        };

        for base_item in target_base_items {
            essence_base_mods
                .entry(row.Essence)
                .or_default()
                .insert(base_item, mods.clone());
        }
    });

    essence_base_mods
        .into_iter()
        .map(|(essence_index, base_mods)| {
            // essencemods.Essence -> essences.BaseItemType -> baseitemtypes.Name
            let name = &dats.base_item_types[dats.essences[essence_index].BaseItemType].Name;

            let special_essences = ["Insanity", "Hysteria", "Horror", "Delirium", "Abyss"];

            if name.starts_with("Perfect")
                | special_essences.iter().any(|suffix| name.ends_with(suffix))
            {
                CurrencyType::PerfectEssence(PerfectEssence {
                    name: name.clone(),
                    tiers: base_mods,
                })
            } else {
                CurrencyType::Essence(Essence {
                    name: name.clone(),
                    tiers: base_mods,
                })
            }
        })
        .collect()
}

pub fn load_mod_tiers(dats: &Dats) -> (HashVec<TierId, Tier>, HashVec<ModGroup, Modifier>) {
    dats.mods.iter().fold(
        (HashVec::default(), HashVec::default()),
        |(mut tiers, mut mod_stats), row| {
            // Parse out value ranges
            let stats_ranges = [
                (row.Stat1, row.Stat1Value),
                (row.Stat2, row.Stat2Value),
                (row.Stat3, row.Stat3Value),
                (row.Stat4, row.Stat4Value),
            ];
            let mut stats = vec![];
            let mut value_ranges = vec![];
            for (stat_id, value_range) in stats_ranges {
                if let Some(stat_id) = stat_id {
                    let stat_id = &dats.stats[stat_id].Id;
                    stats.push(stat_id.clone());
                    value_ranges.push(value_range);
                }
            }

            let mod_group = &dats.mod_type[row.ModType].Name;

            // TODO: Skip empty families?
            let mod_family = &dats.mod_family[*row.Families.first().unwrap_or(&0)].Id;

            let affix = match row.GenerationType {
                1 => Affix::Prefix,
                2 => Affix::Suffix,
                // TODO: rest of affixes
                _ => Affix::Corrupted,
            };

            let tags = row
                .ImplicitTags
                .iter()
                .flat_map(|index| dats.tags[*index].DisplayString.clone())
                .collect();

            let modifier = Modifier {
                group: mod_group.clone(),
                tags,
                // TODO: mod type
                mod_type: ModType::Normal,
                stats,
                family: mod_family.clone(),
            };
            let mod_id = if !mod_stats.contains_key(mod_group) {
                mod_stats.insert(mod_group.clone(), modifier)
            } else {
                mod_stats.opaque(mod_group)
            };

            tiers.insert(
                row.Id.clone(),
                Tier {
                    id: row.Id.clone(),
                    name: row.Name.clone(),
                    mod_id,
                    ilvl: row.Level,
                    value_ranges,
                    mod_domain: row.Domain,
                    // This will be filled in afterwards from poe2db source
                    weight: 0,
                    affix,
                },
            );

            (tiers, mod_stats)
        },
    )
}
