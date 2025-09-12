#![allow(non_snake_case)]
use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Deserializer, de::DeserializeOwned};
use serde_with::serde_as;

use crate::types::{Affix, ModFamily, ModGroup, ModTag, ModType, Modifier, StatID, Tier, TierId};

fn deserialize_json_array_u32<'de, D>(deserializer: D) -> Result<Vec<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

fn deserialize_json_array_i32<'de, D>(deserializer: D) -> Result<[i32; 2], D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

/// Helper trait to make CSV record loading easier
pub trait RecordLoader: DeserializeOwned {
    /// Load the provided CSV into a structured iterator
    fn load(path: &Path) -> impl Iterator<Item = Self> {
        csv::Reader::from_path(path)
            .unwrap()
            .into_deserialize::<Self>()
            .map(|row| row.expect("Failed to deserialise row"))
    }
}

impl<T: DeserializeOwned> RecordLoader for T {}

#[serde_as]
#[derive(Deserialize)]
pub struct ModRecord {
    /// Eg. Strength6
    pub Id: TierId,
    /// Index into ModType table
    pub ModType: u32,
    pub Domain: u32,

    #[serde(deserialize_with = "deserialize_json_array_u32")]
    pub Families: Vec<u32>,
    // Mod Tags
    #[serde(deserialize_with = "deserialize_json_array_u32")]
    pub ImplicitTags: Vec<u32>,
    /// Affix
    pub GenerationType: u32,
    /// Min ilvl
    pub Level: u32,
    /// Eg. of the Mongoose
    pub Name: String,
    /// Index into Stats table
    pub Stat1: Option<u32>,
    pub Stat2: Option<u32>,
    pub Stat3: Option<u32>,
    pub Stat4: Option<u32>,
    // /// Stat value ranges
    #[serde(deserialize_with = "deserialize_json_array_i32")]
    pub Stat1Value: [i32; 2],
    #[serde(deserialize_with = "deserialize_json_array_i32")]
    pub Stat2Value: [i32; 2],
    #[serde(deserialize_with = "deserialize_json_array_i32")]
    pub Stat3Value: [i32; 2],
    #[serde(deserialize_with = "deserialize_json_array_i32")]
    pub Stat4Value: [i32; 2],
}

#[derive(Deserialize)]
pub struct ModTypeRecord {
    pub Name: ModGroup,
}

#[derive(Deserialize)]
pub struct ModFamilyRecord {
    pub Id: ModGroup,
}

#[derive(Deserialize)]
pub struct StatRecord {
    pub Id: StatID,
}

#[derive(Deserialize)]
pub struct BaseItemTypesRecord {
    pub Name: String,
    pub ItemClass: u32,
    /// This should actually be ModDomain, but there's a bug in poe_data_tools
    pub SoundEffect: u32,
}

#[derive(Deserialize)]
pub struct ItemClassRecord {
    pub Id: String,
}

#[derive(Deserialize)]
pub struct TagsRecord {
    pub Id: String,
    pub DisplayString: Option<String>,
}

#[derive(Deserialize)]
pub struct EssencesRecord {
    BaseItemType: u32,
}

#[derive(Deserialize)]
pub struct EssenceTargetItemCategoriesRecord {
    #[serde(deserialize_with = "deserialize_json_array_u32")]
    ItemClasses: Vec<u32>,
}

#[derive(Deserialize)]
pub struct EssenceModRecord {
    pub Essence: u32,
    pub TargetItemCategory: u32,
    pub Mod1: Option<u32>,
    /// The mod that is displayed on the essence item, not what is actually rolled on use
    /// Used for essences that can give many outcomes.
    /// Eg. Mark of the Abyssal Lord (Prefix or Suffix)
    /// Eg. +# to Strength, Dexterity or Intelligence
    pub DisplayMod: Option<u32>,
    /// The possible outcome mods for multi-outcome essences
    #[serde(deserialize_with = "deserialize_json_array_u32")]
    pub OutcomeMods: Vec<u32>,
}

pub struct Dats {
    pub mod_records: Vec<ModRecord>,
    pub mod_type_records: Vec<ModTypeRecord>,
    pub mod_family_records: Vec<ModFamilyRecord>,
    pub stat_records: Vec<StatRecord>,
    pub base_item_types_records: Vec<BaseItemTypesRecord>,
    pub item_class_records: Vec<ItemClassRecord>,
    pub tags_records: Vec<TagsRecord>,
    pub essences_records: Vec<EssencesRecord>,
    pub essence_target_item_categories_records: Vec<EssenceTargetItemCategoriesRecord>,
    pub essence_mod_records: Vec<EssenceModRecord>,
}

impl Dats {
    pub fn load_tables(data_root: &Path) -> Self {
        Self {
            mod_records: ModRecord::load(&data_root.join("TODO_PATH")).collect(),
            mod_type_records: ModTypeRecord::load(&data_root.join("TODO_PATH")).collect(),
            mod_family_records: ModFamilyRecord::load(&data_root.join("TODO_PATH")).collect(),
            stat_records: StatRecord::load(&data_root.join("TODO_PATH")).collect(),
            base_item_types_records: BaseItemTypesRecord::load(&data_root.join("TODO_PATH")).collect(),
            item_class_records: ItemClassRecord::load(&data_root.join("TODO_PATH")).collect(),
            tags_records: TagsRecord::load(&data_root.join("TODO_PATH")).collect(),
            essences_records: EssencesRecord::load(&data_root.join("TODO_PATH")).collect(),
            essence_target_item_categories_records: EssenceTargetItemCategoriesRecord::load(&data_root.join("TODO_PATH")).collect(),
            essence_mod_records: EssenceModRecord::load(&data_root.join("TODO_PATH")).collect(),
        }
    }
}

pub fn load_mod_groups(path: &Path) -> Vec<ModGroup> {
    ModTypeRecord::load(path).map(|row| row.Name).collect()
}
pub fn load_mod_families(path: &Path) -> Vec<ModFamily> {
    ModFamilyRecord::load(path).map(|row| row.Id).collect()
}

pub fn load_stat_ids(path: &Path) -> Vec<StatID> {
    StatRecord::load(path).map(|row| row.Id).collect()
}

pub fn load_mod_tags(path: &Path) -> Vec<Option<ModTag>> {
    TagsRecord::load(path)
        .map(|row| row.DisplayString.map(|_| row.Id))
        .collect()
}

pub fn load_base_item_types(path: &Path) -> Vec<String> {
    BaseItemTypesRecord::load(path)
        .map(|row| row.Name)
        .collect()
}

pub fn load_essences(path: &Path, base_item_types: &[String]) -> Vec<String> {
    EssencesRecord::load(path)
        .map(|row| base_item_types[row.BaseItemType as usize].clone())
        .collect()
}

pub fn load_essence_target_item_categories(
    path: &Path,
    item_classes: &[String],
) -> Vec<Vec<String>> {
    EssenceTargetItemCategoriesRecord::load(path)
        .map(|row| {
            row.ItemClasses
                .into_iter()
                .map(|item_class| item_classes[item_class as usize].clone())
                .collect()
        })
        .collect()
}

pub fn load_essence_mods(
    path: &Path,
    essence_names: &[String],
) -> HashMap<u32, HashMap<u32, Vec<u32>>> {
    let mut essences = HashMap::new();
    EssenceModRecord::load(path).for_each(|row| {
        // essencemods.Essence -> essences.BaseItemType -> baseitemtypes.Name
        let name = &essence_names[row.Essence as usize];
        // essencemods.TargetItemCategory -> essencetargetitemcategories.ItemClasses ->
        //      itemclasses.Id -(kinda)-> BaseItemId
        // essencemods.Mod1 -> mods
        // essencemods.OutcomeMods -> mods
    });

    essences
}

pub fn load_mod_tiers(
    path: &Path,
    stat_ids: &[StatID],
    mod_groups: &[ModGroup],
    mod_familites: &[ModFamily],
    mod_tags: &[Option<ModTag>],
) -> (HashMap<TierId, Tier>, HashMap<ModGroup, Modifier>) {
    ModRecord::load(path).fold(
        (HashMap::new(), HashMap::new()),
        |(mut tiers, mut mod_stats), row| {
            // Parse out value ranges
            let mut stats = vec![];
            let mut value_ranges = vec![];
            if let Some(stat_id) = row.Stat1 {
                let stat_id = &stat_ids[stat_id as usize];
                stats.push(stat_id.clone());
                value_ranges.push(row.Stat1Value);
            }
            if let Some(stat_id) = row.Stat2 {
                let stat_id = &stat_ids[stat_id as usize];
                stats.push(stat_id.clone());
                value_ranges.push(row.Stat2Value);
            }
            if let Some(stat_id) = row.Stat3 {
                let stat_id = &stat_ids[stat_id as usize];
                stats.push(stat_id.clone());
                value_ranges.push(row.Stat3Value);
            }
            if let Some(stat_id) = row.Stat4 {
                let stat_id = &stat_ids[stat_id as usize];
                stats.push(stat_id.clone());
                value_ranges.push(row.Stat4Value);
            }

            let mod_group = mod_groups[row.ModType as usize].clone();

            // TODO: Skip empty families?
            let mod_family = &mod_familites[*row.Families.first().unwrap_or(&0) as usize];

            let affix = match row.GenerationType {
                1 => Affix::Prefix,
                2 => Affix::Suffix,
                // TODO: rest of affixes
                _ => Affix::Corrupted,
            };

            let tags = row
                .ImplicitTags
                .iter()
                .flat_map(|index| mod_tags[*index as usize].clone())
                .collect();

            let modifier = Modifier {
                group: mod_group.clone(),
                tags,
                // TODO: mod type
                mod_type: ModType::Normal,
                stats,
                family: mod_family.clone(),
            };
            if !mod_stats.contains_key(&mod_group) {
                mod_stats.insert(mod_group.clone(), modifier);
            }

            tiers.insert(
                row.Id.clone(),
                Tier {
                    id: row.Id,
                    name: row.Name,
                    mod_id: mod_group,
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

pub fn load_item_classes(path: &Path) -> Vec<String> {
    ItemClassRecord::load(path).map(|row| row.Id).collect()
}

pub fn load_mod_domains(
    path: &Path,
    item_classes: &[String],
) -> (HashMap<String, u32>, HashMap<String, u32>) {
    BaseItemTypesRecord::load(path).fold(
        (HashMap::new(), HashMap::new()),
        |(mut specific, mut coarse), row| {
            coarse.insert(
                item_classes[row.ItemClass as usize].clone(),
                row.SoundEffect,
            );
            specific.insert(row.Name, row.SoundEffect);

            (specific, coarse)
        },
    )
}
