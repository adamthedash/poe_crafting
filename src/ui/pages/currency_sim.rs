use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use egui::{self, DragValue, Grid, ScrollArea, Ui};
use itertools::Itertools;

use crate::{
    ESSENCES, MODS, TIERS,
    currency::{CURRENCIES, Currency, CurrencyType},
    hashvec::OpaqueIndex,
    item_state::{ItemState, get_valid_mods_for_item},
    types::{Omen, Tier},
    ui::{Page, dropdown, omen_selection},
};

#[derive(Debug)]
pub enum SimStatus {
    Done {
        results: HashMap<OpaqueIndex<Tier>, usize>,
    },
    Running {
        iterations_done: usize,
    },
}

#[derive(Debug)]
pub struct SimState {
    _base_item: ItemState,
    status: Arc<Mutex<SimStatus>>,
    _handle: JoinHandle<()>,
}

/// Start a crafting simulation in a new thread
fn run_sim(
    base_item: ItemState,
    currency: CurrencyType,
    omens: HashSet<Omen>,
    num_iters: u64,
) -> SimState {
    let candidate_tiers = get_valid_mods_for_item(&base_item);

    let status = Arc::new(Mutex::new(SimStatus::Running { iterations_done: 0 }));
    SimState {
        _base_item: base_item.clone(),
        status: status.clone(),
        _handle: thread::spawn({
            move || {
                let mut results = HashMap::<_, usize>::new();
                let before_mods = base_item.mods.iter().copied().collect::<HashSet<_>>();
                for _ in 0..num_iters {
                    // Apply the currency
                    let mut item = base_item.clone();
                    currency.craft(&mut item, &candidate_tiers, &omens);

                    // Figure out which mod was added
                    let after_mods = item.mods.iter().copied().collect::<HashSet<_>>();
                    let added = after_mods.difference(&before_mods);
                    for tier_id in added {
                        *results.entry(*tier_id).or_default() += 1;
                    }

                    // Update status
                    let mut status = status.lock().unwrap();
                    let SimStatus::Running { iterations_done } = &mut *status else {
                        unreachable!();
                    };
                    *iterations_done += 1;
                }

                // Give the results back
                let mut status = status.lock().unwrap();
                *status = SimStatus::Done { results };
            }
        }),
    }
}

/// A grid showing the % chance for each mod to roll
fn display_sim_results(ui: &mut Ui, results: &HashMap<OpaqueIndex<Tier>, usize>) {
    let total_iters = results.values().sum::<usize>();

    let affix_groups = results
        .iter()
        .map(|(&tier_id, &count)| (&TIERS[tier_id], count))
        .sorted_unstable_by_key(|(tier, _)| (tier.affix, &tier.mod_id, tier.ilvl))
        .chunk_by(|(tier, _)| tier.affix);

    ScrollArea::new([false, true]).show(ui, |ui| {
        for (affix, mod_group) in &affix_groups {
            ui.heading(format!("{:?}", affix));
            Grid::new(format!("results_grid_{:?}", affix))
                .num_columns(2)
                .show(ui, |ui| {
                    for (&mod_id, group) in &mod_group.chunk_by(|(tier, _)| &tier.mod_id) {
                        let modifier = &MODS[mod_id];
                        ui.label(&modifier.group);

                        let tier_counts = group.collect::<Vec<_>>();
                        Grid::new(format!("results_grid_{}", modifier.group))
                            .num_columns(tier_counts.len())
                            .show(ui, |ui| {
                                // Ilvls on top row
                                tier_counts.iter().for_each(|(tier, _)| {
                                    ui.label(format!("{}", tier.ilvl));
                                });
                                ui.end_row();

                                // Roll % on bottom row
                                tier_counts.into_iter().for_each(|(_, count)| {
                                    ui.label(format!(
                                        "{:.1}%",
                                        (count as f32 / total_iters as f32) * 100.
                                    ));
                                });
                            });

                        ui.end_row();
                    }
                });
        }
    });
}

pub fn show_page(page_state: &mut Page, ctx: &egui::Context, item: &ItemState) {
    // Unpack state variables
    let Page::CraftProbability {
        selected_currency,
        selected_omens,
        simulation_state,
        num_iters_exp,
    } = page_state
    else {
        unreachable!()
    };

    let candidate_tiers = get_valid_mods_for_item(item);

    let currencies = CURRENCIES
        .iter()
        .chain(ESSENCES.get().unwrap().iter().sorted_unstable_by_key(|e| {
            let name = e.name();
            let sort = match name.split(" ").next().unwrap() {
                "Lesser" => 0,
                "Essence" => 1,
                "Greater" => 2,
                "Perfect" => 3,
                _ => 4,
            };
            (sort, name)
        }))
        .filter(|c| c.can_be_used(item, &candidate_tiers, &HashSet::new()))
        .collect::<Vec<_>>();

    egui::CentralPanel::default().show(ctx, |ui| {
        // Select Currency
        let old_selected = dropdown(ui, selected_currency, &currencies, "currency_select", |c| {
            c.name().to_string()
        });
        if old_selected.is_some() {
            // Currency changed, clear omens
            selected_omens.clear();
        }

        // Select Omens
        omen_selection(ui, selected_currency, selected_omens, Some(item));

        // 10^N iterations
        ui.add(
            DragValue::new(num_iters_exp)
                .range(0..=10)
                .custom_formatter(|n, _| format!("{:?}", 10_u64.pow(n as u32))),
        );
        let n = 10_u64.pow(*num_iters_exp);

        // Simulation
        if ui.button("Go!").clicked() {
            let state = run_sim(
                item.clone(),
                selected_currency.clone(),
                selected_omens.clone(),
                n,
            );
            *simulation_state = Some(state);
        }

        if let Some(sim_state) = simulation_state {
            match &*sim_state.status.lock().unwrap() {
                SimStatus::Done { results } => {
                    display_sim_results(ui, results);
                }
                SimStatus::Running { iterations_done } => {
                    ui.spinner();
                    ui.label(format!("{} / {}", iterations_done, n));
                }
            }
        }
    });
}
