use egui::Ui;
use itertools::Itertools;

use crate::currency::{Currency, CurrencyType};

/// Organised dropdown menu with sub-menus
pub fn currency_dropdown<'a>(
    ui: &mut Ui,
    value: &mut &'a CurrencyType,
    currencies: &[&'a CurrencyType],
) {
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
    currency_submenu(ui, value, &name, &currencies);
}

#[derive(Clone)]
enum MenuEntry<'a> {
    One(&'a CurrencyType),
    Many(Vec<(&'a str, MenuEntry<'a>)>),
}

fn currency_submenu<'a>(
    ui: &mut Ui,
    value: &mut &'a CurrencyType,
    name: &str,
    contents: &[(&str, MenuEntry<'a>)],
) {
    ui.menu_button(name, |ui| {
        for (entry_name, entry_contents) in contents {
            match entry_contents {
                MenuEntry::One(currency_type) => {
                    if ui.button(currency_type.name()).clicked() {
                        *value = *currency_type
                    }
                }
                MenuEntry::Many(items) => {
                    currency_submenu(ui, value, entry_name, items);
                }
            };
        }
    });
}
