use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use serde::{Deserialize, Deserializer};
use serde_with::serde_as;

use crate::types::{Affix, ModFamily, ModGroup, ModType, Modifier, StatID, Tier, TierId};

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
pub struct BaseItemTypeRecord {
    pub Name: String,
    pub ItemClass: u32,
    /// This should actually be ModDomain, but there's a bug in poe_data_tools
    pub SoundEffect: u32,
}

#[derive(Deserialize)]
pub struct ItemClassRecord {
    pub Id: String,
}

pub fn load_mod_groups(path: &Path) -> Vec<ModGroup> {
    csv::Reader::from_path(path)
        .unwrap()
        .deserialize::<ModTypeRecord>()
        .map(|row| row.unwrap().Name)
        .collect()
}
pub fn load_mod_families(path: &Path) -> Vec<ModFamily> {
    csv::Reader::from_path(path)
        .unwrap()
        .deserialize::<ModFamilyRecord>()
        .map(|row| row.unwrap().Id)
        .collect()
}

pub fn load_stat_ids(path: &Path) -> Vec<StatID> {
    csv::Reader::from_path(path)
        .unwrap()
        .deserialize::<StatRecord>()
        .map(|row| row.unwrap().Id)
        .collect()
}

pub fn load_mod_tiers(
    path: &Path,
    stat_ids: &[StatID],
    mod_groups: &[ModGroup],
    mod_familites: &[ModFamily],
) -> (HashMap<TierId, Tier>, HashMap<ModGroup, Modifier>) {
    csv::Reader::from_path(path)
        .unwrap()
        .deserialize::<ModRecord>()
        .map(Result::unwrap)
        .fold(
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

                println!("{:?} {:?}", row.Id, row.GenerationType);
                let affix = match row.GenerationType {
                    1 => Affix::Prefix,
                    2 => Affix::Suffix,
                    // TODO: rest of affixes
                    _ => Affix::Corrupted,
                };

                let modifier = Modifier {
                    group: mod_group.clone(),
                    // TODO: tags
                    tags: HashSet::new(),
                    affix,
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
                        // TODO: this will be filled in afterwards
                        weight: 0,
                    },
                );

                (tiers, mod_stats)
            },
        )
}

pub fn load_item_classes(path: &Path) -> Vec<String> {
    csv::Reader::from_path(path)
        .unwrap()
        .deserialize::<ItemClassRecord>()
        .map(|row| row.unwrap().Id)
        .collect()
}

pub fn load_mod_domains(
    path: &Path,
    item_classes: &[String],
) -> (HashMap<String, u32>, HashMap<String, u32>) {
    let mut coarse_domains = HashMap::new();

    let specific_domains = csv::Reader::from_path(path)
        .unwrap()
        .deserialize::<BaseItemTypeRecord>()
        .map(|row| {
            let row = row.unwrap();

            coarse_domains.insert(
                item_classes[row.ItemClass as usize].clone(),
                row.SoundEffect,
            );

            (row.Name, row.SoundEffect)
        })
        .collect();

    (specific_domains, coarse_domains)
}
