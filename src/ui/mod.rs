use std::{
    fmt::{Debug, Display},
    mem::replace,
};

use egui::{ComboBox, Ui};

use crate::item_state::Rarity;

/// Dropdown menu for Rarity. Returns the previous value if it has changed on this frame.
pub fn rarity_dropdown(ui: &mut Ui, value: &mut Rarity) -> Option<Rarity> {
    dropdown(
        ui,
        value,
        &[Rarity::Normal, Rarity::Magic, Rarity::Rare]
            .iter()
            .collect::<Vec<_>>(),
        "combo_rarity",
        |r| format!("{:?}", r),
    )
}

/// Dropdown which returns the previously selected value on change
pub fn dropdown<T: Clone + PartialEq>(
    ui: &mut Ui,
    value: &mut T,
    values: &[&T],
    key: &str,
    formatter: fn(&T) -> String,
) -> Option<T> {
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
