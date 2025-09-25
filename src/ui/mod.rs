pub mod components;
pub mod pages;

use std::{collections::HashSet, mem::replace, ops::RangeInclusive};

use egui::{ComboBox, DragValue, Ui};

use crate::{
    currency::{Currency, CurrencyType},
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
    strategy::Strategy,
    types::Omen,
    ui::pages::ui_debug,
};

/// Persisted state for the pages
#[derive(Debug)]
pub enum Page {
    ItemBuilder,
    CraftProbability {
        selected_currency: CurrencyType,
        selected_omens: HashSet<Omen>,
        simulation_state: Option<pages::currency_sim::SimState>,
        num_iters_exp: u32,
    },
    StrategyBuilder {
        strategy: Strategy,
        simulation_state: Option<pages::strategy_sim::SimState>,
    },
    UIDebug(ui_debug::PageState),
}

impl Page {
    pub fn all() -> Vec<Self> {
        use Page::*;

        vec![
            ItemBuilder,
            CraftProbability {
                selected_currency: CurrencyType::Transmute,
                selected_omens: HashSet::new(),
                simulation_state: None,
                num_iters_exp: 5,
            },
            StrategyBuilder {
                strategy: Strategy(vec![]),
                simulation_state: None,
            },
            UIDebug(ui_debug::PageState::default()),
        ]
    }

    pub fn name(&self) -> &str {
        match self {
            Page::ItemBuilder => "Item Builder",
            Page::CraftProbability { .. } => "Craft Probabilities",
            Page::StrategyBuilder { .. } => "Strategy Builder",
            Page::UIDebug(_) => "UI Debug",
        }
    }
}

/// Dropdown menu for Rarity. Returns the previous value if it has changed on this frame.
pub fn rarity_dropdown(ui: &mut Ui, value: &mut Rarity, key: &str) -> Option<Rarity> {
    dropdown(
        ui,
        value,
        &[Rarity::Normal, Rarity::Magic, Rarity::Rare]
            .iter()
            .collect::<Vec<_>>(),
        key,
        |r| format!("{:?}", r),
    )
}

/// Dropdown which returns the previously selected value on change
pub fn dropdown<T, F>(
    ui: &mut Ui,
    value: &mut T,
    values: &[&T],
    key: &str,
    formatter: F,
) -> Option<T>
where
    T: Clone + PartialEq,
    F: Fn(&T) -> String,
{
    let mut new_val = &*value;

    ComboBox::from_id_salt(key)
        .selected_text(formatter(value))
        .show_ui(ui, |ui| {
            for &val in values {
                ui.selectable_value(&mut new_val, val, formatter(val));
            }
        });

    if new_val != value {
        let old_val = replace(value, new_val.clone());
        Some(old_val)
    } else {
        None
    }
}

/// Selection for an inclusive range
pub fn range_selector(
    ui: &mut Ui,
    value: &mut RangeInclusive<usize>,
    possible_range: RangeInclusive<usize>,
) {
    ui.horizontal(|ui| {
        let mut min = *value.start();
        ui.label("Min");
        ui.add(DragValue::new(&mut min).range(possible_range.clone()));

        let mut max = *value.end();
        ui.label("Max");
        ui.add(DragValue::new(&mut max).range(possible_range));

        *value = min..=max;
    });
}

pub fn multi_select_checkboxes<T: Clone + PartialEq>(
    ui: &mut Ui,
    values: &mut Vec<T>,
    options: &[&T],
    formatter: fn(&T) -> String,
) {
    ui.horizontal(|ui| {
        for &option in options {
            let mut selected = values.contains(option);
            let old_selected = selected;
            ui.checkbox(&mut selected, formatter(option));
            match (old_selected, selected) {
                (true, false) => {
                    // Un-selected
                    values.retain(|val| val != option);
                }
                (false, true) => {
                    // Selected
                    values.push(option.clone());
                }
                _ => (),
            }
        }
    });
}

// Multi-checkbox for selecting omens for a currency
pub fn omen_selection(
    ui: &mut Ui,
    currency: &CurrencyType,
    omens: &mut HashSet<Omen>,
    item_filter: Option<&ItemState>,
) {
    let possible_omens = currency.possible_omens();
    let mut possible_omens = possible_omens.iter().collect::<Vec<_>>();
    if let Some(item) = item_filter {
        let candidate_tiers = get_valid_mods_for_item(item);
        possible_omens.retain(|&&o| {
            currency.can_be_used(
                item,
                &candidate_tiers,
                &HashSet::from_iter(std::iter::once(o)),
            )
        });
    }
    possible_omens.sort();

    let mut selected_omens = omens.iter().copied().collect::<Vec<_>>();
    multi_select_checkboxes(ui, &mut selected_omens, &possible_omens, |omen| {
        format!("{omen:?}")
    });

    *omens = HashSet::from_iter(selected_omens)
}
