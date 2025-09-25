use egui::CentralPanel;

use crate::{
    CURRENCIES, currency::CurrencyType, ui::components::currency_selection::currency_dropdown,
};

#[derive(Debug)]
pub struct PageState {
    currency: &'static CurrencyType,
}

impl Default for PageState {
    fn default() -> Self {
        Self {
            currency: CURRENCIES[0],
        }
    }
}

pub fn show_page(ctx: &egui::Context, state: &mut PageState) {
    CentralPanel::default().show(ctx, |ui| {
        currency_dropdown(ui, &mut state.currency, &CURRENCIES)
    });
}
