/**
*   All the data in a standardised format I want
*/
use std::collections::{HashMap, HashSet};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::hashvec::OpaqueIndex;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
pub enum Affix {
    Prefix,
    Suffix,
    Corrupted,
}

// Not sure if this is the best way to represent this
#[derive(Debug, Clone)]
pub enum ModType {
    Normal,
    Desecrated,
    Essence,
}

/// Eg. IncreasedLife
pub type ModGroup = String;
pub type ModFamily = String;

pub type ModTag = String;

/// Represents a family of modifiers
#[derive(Debug, Clone)]
pub struct Modifier {
    /// Eg. BaseLocalDefencesAndLife
    pub group: ModGroup,
    /// Eg. Fire, Attack
    pub tags: HashSet<ModTag>,
    pub mod_type: ModType,
    /// The individual stats provided by this mod Eg. 2 for hybrid attack/accuracy
    pub stats: Vec<StatID>,

    pub family: ModFamily,
}

/// Eg. Strength7
pub type TierId = String;

/// A specific tier of a modifier
#[derive(Debug, Clone)]
pub struct Tier {
    pub id: TierId,
    /// Eg. of the Hare
    pub name: String,
    /// Link to parent modifier
    pub mod_id: OpaqueIndex<Modifier>,
    /// Where this mod rolls
    pub affix: Affix,
    /// Minimum required ilvl
    pub ilvl: u32,
    /// One range for each stat provided by this mod
    pub value_ranges: Vec<[i32; 2]>,
    /// Sample weight when rolling this mod
    pub weight: u32,
    /// The "rolling group" for this mod - maybe better to have in the Modifier struct?
    pub mod_domain: usize,
}

/// Eg. %_attack_damage_per_glory_consumed_for_6_seconds_up_to_100
pub type StatID = String;

/// A single stat provided by a mod
#[derive(Debug)]
pub struct Stat {
    /// Eg. additional_strength
    pub id: StatID,
}

/// Eg. Sceptre, Boots (dex)
pub type BaseItemId = String;

/// The base type of an item
#[derive(Debug)]
struct BaseItem {
    name: BaseItemId,
}

/// Modifiers which cannot occur together
type ConflictingMods = Vec<HashSet<ModGroup>>;

/// Mods which can occur on a given base item type
pub type ItemMods = HashMap<BaseItemId, Vec<TierId>>;

/// How a stat is formatted in a specific situation
/// TODO: formatters are identified by tuples of StatIDs.
///     Eg. min/max added damage mods
///
/// Eg. gloves with hybrid es/accuracy and flat damage will be:
///     es/acc mod -> (es stat, acc stat) -> (es formatter, acc formatter)
///     flat mod -> (min stat, max stat) -> flat formatter
///
///     Not sure if theres any weirder examples
///     1) Attempt to look up by all [StatID]s for a mod
///     2) Fall back to looking up each StatID individually
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct StatFormatter {
    /// When this formatter is applied
    #[serde(rename = "condition")]
    pub conditions: Vec<Condition>,
    /// Format descriptors for a single value
    #[serde(rename = "format")]
    pub formats: Vec<String>,
    /// Value pre-processing functions
    pub index_handlers: Vec<Vec<String>>,
    /// Format string with {i} and [either/or] placeholders
    pub string: String,
}

impl StatFormatter {
    /// Get the mod as it appears on the trade site
    /// Eg. "+# increased Attack Speed"
    pub fn trade_string(&self) -> String {
        let string = self.string.clone();

        // "{i}" -> "#"
        let string = self
            .formats
            .iter()
            .enumerate()
            .fold(string, |string, (i, replace)| {
                string.replacen(&format!("{{{i}}}"), replace, 1)
            });

        // "[A|B]" -> B
        let regex_either = Regex::new(r"\[([^\]]+)\]").unwrap();
        let replaces = regex_either
            .captures_iter(&string)
            .map(|hit| {
                // First capture group is the entire thing
                let find = hit.iter().next().unwrap().unwrap().as_str();
                // Replace with last item in the list
                let replace = hit
                    .iter()
                    .last()
                    .unwrap()
                    .unwrap()
                    .as_str()
                    .split("|")
                    .last()
                    .unwrap();

                (find, replace)
            })
            .collect::<Vec<_>>();

        replaces
            .iter()
            .fold(string.clone(), |acc, (find, replace)| {
                acc.replacen(find, replace, 1)
            })
    }

    /// A string ready to format!()
    /// Eg. "+{} increased Attack Speed"
    pub fn rust_fmt_string(&self) -> String {
        self.trade_string().replace("#", "{}")
    }

    /// Format with placeholder values for a tier
    /// Eg. "+(5-12) increased Attack Speed"
    pub fn format_value_range(&self, value_ranges: &[[i32; 2]]) -> String {
        value_ranges
            .iter()
            .fold(self.rust_fmt_string(), |string, value_range| {
                string.replacen("{}", &format!("({}-{})", value_range[0], value_range[1]), 1)
            })
    }
}

pub fn get_matching_formatter<'a>(
    formatters: &'a [StatFormatter],
    values: &[i32],
) -> &'a StatFormatter {
    formatters
        .iter()
        .find(|formatter| {
            formatter
                .conditions
                .iter()
                .zip(values)
                .all(|(cond, val)| cond.matches(*val))
        })
        .unwrap_or_else(|| {
            panic!(
                "No matching formatter! values {:?} formatters {:?}",
                values, formatters
            )
        })
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Condition {
    pub min: Option<i32>,
    pub max: Option<i32>,
    pub negate: Option<bool>,
}

impl Condition {
    pub fn matches(&self, value: i32) -> bool {
        self.min.map(|min| value >= min).unwrap_or(true)
            && self.max.map(|max| value <= max).unwrap_or(true)
        // TODO: Not sure how negated works
    }
}

/// Formatters for stats, the first matching formatter is used.
/// LUT key is "|" delimited StatID's
pub type StatFormatters = HashMap<String, Vec<StatFormatter>>;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Omen {
    Sinistral,
    Dextral,
    Homogenous,
    Greater,
    Whittling,
}
