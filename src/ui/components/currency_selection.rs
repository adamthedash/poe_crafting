use egui::Ui;
use itertools::Itertools;

use crate::currency::{Currency, CurrencyType};

/// Organised dropdown menu with sub-menus
pub fn currency_dropdown<'a>(
    ui: &mut Ui,
    value: &mut &'a CurrencyType,
    currencies: &[&'a CurrencyType],
) -> Option<&'a CurrencyType> {
    // Standard
    //  Exalt
    //      Base
    //      Greater
    //      Perfect
    // Essences
    //      Haste
    //          Lesser
    //          Base
    //          Greater
    //          Perfect
    //      Hysteria
    //      Horror
    //  Desecrate
    //      Jawbone
    //          Preserved
    //          Ancient

    // Bunch of pre-processing of the currencies to group them up nicely
    use CurrencyType::*;
    let categories = currencies
        .iter()
        .sorted_unstable_by_key(|c| match c {
            Essence(_) | PerfectEssence(_) => "Essence",
            _ => "Standard",
        })
        .chunk_by(|c| match c {
            Essence(_) | PerfectEssence(_) => "Essence",
            _ => "Standard",
        });

    let mut currencies = vec![];
    for (category, group) in &categories {
        // Group by base currency type: Eg. "Greater Exalt", "Exalt" -> "Exalt"
        let prefixes = match category {
            "Standard" => vec!["Greater ", "Perfect "],
            "Essence" => vec!["Lesser ", "Greater ", "Perfect "],
            _ => unreachable!(),
        };
        let base_types = group
            .map(|c| {
                // Try strip the prefix, otherwise just use the name as-is
                let base_name = prefixes
                    .iter()
                    .flat_map(|pre| c.name().strip_prefix(pre))
                    .next()
                    .unwrap_or(c.name());

                (base_name, c)
            })
            .sorted_unstable_by_key(|(base, _)| *base)
            .chunk_by(|(base, _)| *base);

        let subgroup = base_types
            .into_iter()
            .map(|(base_type, group)| {
                // Tier of currency: Eg. Greater/Perfect
                let group = group
                    .map(|(name, c)| (name, MenuEntry::One(c)))
                    .collect::<Vec<_>>();

                let group = if group.len() > 1 {
                    MenuEntry::Many(group)
                } else {
                    group[0].1.clone()
                };

                (base_type, group)
            })
            .collect::<Vec<_>>();

        currencies.push((category, MenuEntry::Many(subgroup)));
    }

    // Display currently selected currency at top level
    let name = value.name().to_string();
    currency_submenu(ui, value, &name, &currencies)
}

#[derive(Clone)]
enum MenuEntry<'a> {
    /// Displays the entry as-is
    One(&'a CurrencyType),
    /// Displays a submenu with the provided name and contents
    Many(Vec<(&'a str, MenuEntry<'a>)>),
}

/// Recursive submenus
fn currency_submenu<'a>(
    ui: &mut Ui,
    value: &mut &'a CurrencyType,
    name: &str,
    contents: &[(&str, MenuEntry<'a>)],
) -> Option<&'a CurrencyType> {
    ui.menu_button(name, |ui| {
        contents
            .iter()
            .flat_map(|(entry_name, entry_contents)| match entry_contents {
                MenuEntry::One(currency_type) => {
                    if ui.button(currency_type.name()).clicked() {
                        let old_value = *value;
                        *value = *currency_type;

                        (old_value != *value).then_some(old_value)
                    } else {
                        None
                    }
                }
                MenuEntry::Many(items) => currency_submenu(ui, value, entry_name, items),
            })
            .next()
    })
    .inner
    .flatten()
}
