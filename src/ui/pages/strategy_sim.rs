use std::{
    collections::HashSet,
    path::Path,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use egui::{self, CentralPanel, Color32, Frame, Grid, ScrollArea, Ui};
use itertools::Itertools;

use crate::{
    CURRENCIES, MODS, TIERS,
    currency::{Currency, CurrencyType},
    hashvec::OpaqueIndex,
    io::SavedStrategy,
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
    strategy::{Condition, ConditionGroup, ModifierCondition, Strategy},
    types::{Modifier, Omen, Tier},
    ui::{
        Page, components::currency_selection::currency_dropdown, dropdown, multi_select_checkboxes,
        omen_selection, range_selector, rarity_dropdown,
    },
};

#[derive(Debug)]
pub enum SimStatus {
    InvalidCraft {
        item: ItemState,
        currency: CurrencyType,
        omens: HashSet<Omen>,
    },
    NoMatchingState {
        item: ItemState,
    },
    Running {
        current_iters: usize,
        total_iters: usize,
    },
    Done {
        state_transitions: Vec<Vec<usize>>,
    },
}

#[derive(Debug)]
pub struct SimState {
    base_item: ItemState,
    strategy: Strategy,
    handle: JoinHandle<()>,
    status: Arc<Mutex<SimStatus>>,
}

fn show_strategy_step(
    ui: &mut Ui,
    key: &str,
    condition: &mut Condition,
    candidate_mods: &[(OpaqueIndex<Modifier>, Vec<OpaqueIndex<Tier>>)],
) -> Option<OrderRequest> {
    ui.vertical(|ui| {
        Frame::default()
            .fill(Color32::DARK_RED)
            .show(ui, |ui| {
                let order_action = ui
                    .horizontal(|ui| {
                        // Button to remove this step
                        [
                            ui.button("X").clicked().then_some(OrderRequest::Remove),
                            ui.button("^").clicked().then_some(OrderRequest::MoveUp),
                            ui.button("v").clicked().then_some(OrderRequest::MoveDown),
                            ui.button("Copy").clicked().then_some(OrderRequest::Copy),
                        ]
                        .into_iter()
                        .flatten()
                        .next()
                    })
                    .inner;

                rarity_dropdown(ui, &mut condition.rarity, &format!("rarity_{key}"));

                // Condition groups
                let to_remove = condition
                    .groups
                    .iter_mut()
                    .enumerate()
                    .flat_map(|(i, group)| {
                        show_strategy_group(ui, &format!("{key}_{i}"), group, candidate_mods)
                            .then_some(i)
                    })
                    .next();

                if let Some(index) = to_remove {
                    condition.groups.remove(index);
                }

                // Button to add a new group
                if ui.button("Add new group").clicked() {
                    condition.groups.push(ConditionGroup::Count {
                        count: 0..=1,
                        mods: vec![],
                    });
                }

                order_action
            })
            .inner
    })
    .inner
}

fn show_strategy_group(
    ui: &mut Ui,
    key: &str,
    group: &mut ConditionGroup,
    candidate_mods: &[(OpaqueIndex<Modifier>, Vec<OpaqueIndex<Tier>>)],
) -> bool {
    Frame::default()
        .fill(Color32::DARK_GREEN)
        .show(ui, |ui| {
            // Button to remove this group
            let remove = ui.button("X").clicked();

            // Dropdown with ConditionGroup type
            let mut dropdown_type = match group {
                ConditionGroup::Count { .. } => "Count",
                ConditionGroup::Not(_) => "Not",
                ConditionGroup::AffixCount { .. } => "Affix Count",
            };
            let group_types = ["Count", "Not", "Affix Count"].iter().collect::<Vec<_>>();
            let old = dropdown(
                ui,
                &mut dropdown_type,
                &group_types,
                &format!("dropdown_cond_group_type_{key}"),
                |t| t.to_string(),
            );
            if old.is_some() {
                *group = match dropdown_type {
                    "Count" => ConditionGroup::Count {
                        count: 0..=1,
                        mods: vec![],
                    },
                    "Not" => ConditionGroup::Not(vec![]),
                    "Affix Count" => ConditionGroup::AffixCount {
                        suffixes: 0..=3,
                        prefixes: 0..=3,
                        affixes: 0..=6,
                    },
                    _ => unreachable!(),
                }
            }

            match group {
                ConditionGroup::Count {
                    count,
                    mods: mod_conds,
                } => {
                    range_selector(ui, count, 0..=100);

                    let to_remove = mod_conds
                        .iter_mut()
                        .enumerate()
                        .flat_map(|(i, mod_condition)| {
                            show_strategy_mod(
                                ui,
                                &format!("{key}_{i}"),
                                mod_condition,
                                candidate_mods,
                            )
                            .then_some(i)
                        })
                        .next();

                    if let Some(index) = to_remove {
                        mod_conds.remove(index);
                    }

                    if ui.button("Add mod").clicked() {
                        mod_conds.push(ModifierCondition {
                            mod_group: candidate_mods.first().unwrap().0,
                            levels: vec![],
                        });
                    }
                }
                ConditionGroup::Not(mod_ids) => {
                    // Show mods that can roll on this item
                    let mod_groups = candidate_mods
                        .iter()
                        .map(|(mod_id, _)| mod_id)
                        .collect::<Vec<_>>();

                    let to_remove = mod_ids
                        .iter_mut()
                        .enumerate()
                        .flat_map(|(i, mod_id)| {
                            ui.horizontal(|ui| {
                                // Button to remove this mod
                                let remove = ui.button("X").clicked();

                                dropdown(
                                    ui,
                                    mod_id,
                                    &mod_groups,
                                    &format!("dropdown_mod_group_{key}_{i}"),
                                    |mod_id| MODS[*mod_id].group.clone(),
                                );

                                remove
                            })
                            .inner
                            .then_some(i)
                        })
                        .next();

                    if let Some(index) = to_remove {
                        mod_ids.remove(index);
                    }

                    if ui.button("Add mod").clicked() {
                        mod_ids.push(candidate_mods.first().unwrap().0);
                    }
                }
                ConditionGroup::AffixCount {
                    prefixes,
                    suffixes,
                    affixes,
                } => {
                    ui.horizontal(|ui| {
                        ui.label("Prefixes");
                        range_selector(ui, prefixes, 0..=100);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Suffixes");
                        range_selector(ui, suffixes, 0..=100);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Affixes");
                        range_selector(ui, affixes, 0..=100);
                    });
                }
            }

            remove
        })
        .inner
}

fn show_strategy_mod(
    ui: &mut Ui,
    key: &str,
    mod_condition: &mut ModifierCondition,
    candidate_mods: &[(OpaqueIndex<Modifier>, Vec<OpaqueIndex<Tier>>)],
) -> bool {
    ui.horizontal(|ui| {
        // Button to remove this mod
        let remove = ui.button("X").clicked();

        // Show mods that can roll on this item
        let mod_groups = candidate_mods
            .iter()
            .map(|(mod_id, _)| mod_id)
            .collect::<Vec<_>>();

        let old = dropdown(
            ui,
            &mut mod_condition.mod_group,
            &mod_groups,
            &format!("dropdown_mod_group_{key}"),
            |mod_id| MODS[*mod_id].group.clone(),
        );

        if old.is_some() {
            // Mod changed, clear selected ilvls
            mod_condition.levels.clear();
        }

        // Checkboxes for ilvls
        let group_ilvls = candidate_mods
            .iter()
            .find(|(mod_id, _)| *mod_id == mod_condition.mod_group)
            .unwrap()
            .1
            .iter()
            .map(|&tier_id| &TIERS[tier_id].ilvl)
            .collect::<Vec<_>>();

        multi_select_checkboxes(ui, &mut mod_condition.levels, &group_ilvls, |ilvl| {
            format!("{}", ilvl)
        });

        remove
    })
    .inner
}

enum OrderRequest {
    Remove,
    MoveUp,
    MoveDown,
    Copy,
}

enum OrderAction {
    Remove(usize),
    MoveUp(usize),
    MoveDown(usize),
    Copy(usize),
}

fn get_tiers_mods(
    item: &ItemState,
) -> (
    Vec<OpaqueIndex<Tier>>,
    Vec<(OpaqueIndex<Modifier>, Vec<OpaqueIndex<Tier>>)>,
) {
    let candidate_tiers = get_valid_mods_for_item(item);
    let candidate_mods = candidate_tiers
        .iter()
        .sorted_unstable_by_key(|&&tier_id| {
            let tier = &TIERS[tier_id];

            (tier.mod_id, tier.ilvl)
        })
        .chunk_by(|&&tier_id| {
            let tier = &TIERS[tier_id];

            tier.mod_id
        });
    let candidate_mods = candidate_mods
        .into_iter()
        .map(|(mod_id, group)| (mod_id, group.copied().collect::<Vec<_>>()))
        .collect::<Vec<_>>();

    (candidate_tiers, candidate_mods)
}

pub fn show_page(page_state: &mut Page, ctx: &egui::Context, item: &mut ItemState) {
    let Page::StrategyBuilder {
        strategy,
        simulation_state,
    } = page_state
    else {
        unreachable!()
    };

    let (mut candidate_tiers, mut candidate_mods) = get_tiers_mods(item);

    CentralPanel::default().show(ctx, |ui| {
        ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    // Serialise strategy to JSON
                    let _ = SavedStrategy {
                        base_item: item.clone(),
                        strategy: strategy.clone(),
                    }
                    .save(Path::new("strat.json"));
                }
                if ui.button("Load").clicked() {
                    // Load strategy, TODO: verify that it's valid?
                    let saved_strategy = SavedStrategy::load(Path::new("strat.json"));
                    if let Ok(saved_strategy) = saved_strategy {
                        *strategy = saved_strategy.strategy;
                        *item = saved_strategy.base_item;
                        (candidate_tiers, candidate_mods) = get_tiers_mods(item);
                    }
                }
            });

            let order_action = strategy
                .0
                .iter_mut()
                .enumerate()
                .flat_map(|(i, (condition, action))| {
                    ui.horizontal(|ui| {
                        // Condition
                        let order_action =
                            show_strategy_step(ui, &format!("{i}"), condition, &candidate_mods)
                                .map(|request| match request {
                                    OrderRequest::Remove => OrderAction::Remove(i),
                                    OrderRequest::MoveUp => OrderAction::MoveUp(i),
                                    OrderRequest::MoveDown => OrderAction::MoveDown(i),
                                    OrderRequest::Copy => OrderAction::Copy(i),
                                });

                        // Action
                        ui.vertical(|ui| {
                            let mut do_action = action.is_some();
                            ui.checkbox(&mut do_action, "Craft");
                            if do_action {
                                if action.is_none() {
                                    // Default selection
                                    *action = Some((HashSet::new(), CurrencyType::Transmute));
                                }
                                // Show currency dropdown
                                let Some((selected_omens, currency)) = action else {
                                    unreachable!()
                                };

                                // Select Currency
                                let mut selected = &*currency;
                                let old_selected =
                                    currency_dropdown(ui, &mut selected, &CURRENCIES);
                                if old_selected.is_some() {
                                    // Currency changed, clear omens
                                    selected_omens.clear();
                                    *currency = selected.clone();
                                }

                                // Select Omens
                                omen_selection(ui, currency, selected_omens, None);
                            } else {
                                // No action - end state
                                *action = None;
                            }
                        });

                        order_action
                    })
                    .inner
                })
                .next();

            if let Some(action) = order_action {
                match action {
                    OrderAction::Remove(i) => {
                        strategy.0.remove(i);
                    }
                    OrderAction::MoveUp(i) => {
                        if i > 0 {
                            strategy.0.swap(i - 1, i);
                        }
                    }
                    OrderAction::MoveDown(i) => {
                        if i + 1 < strategy.0.len() {
                            strategy.0.swap(i, i + 1);
                        }
                    }
                    OrderAction::Copy(i) => strategy.0.insert(i + 1, strategy.0[i].clone()),
                };
            }

            // Button to add a new condition
            if ui.button("Add new condition").clicked() {
                strategy.0.push((
                    Condition {
                        rarity: Rarity::Normal,
                        groups: vec![],
                    },
                    None,
                ));
            }

            // Strategy simulation
            if ui.button("Go!").clicked() {
                *simulation_state = Some(run_sim(
                    item.clone(),
                    strategy.clone(),
                    100,
                    &candidate_tiers,
                ));
            }

            if let Some(sim_state) = simulation_state {
                match &*sim_state.status.lock().unwrap() {
                    SimStatus::InvalidCraft {
                        item,
                        currency,
                        omens,
                    } => {
                        ui.label("Invalid craft:");
                        ui.label(format!("{}", item));
                        ui.label(format!("{} {:?}", currency.name(), omens));
                    }
                    SimStatus::NoMatchingState { item } => {
                        ui.label("No matching condition for item:");
                        ui.label(format!("{}", item));
                    }
                    SimStatus::Running {
                        current_iters,
                        total_iters,
                    } => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(format!("{} / {}", current_iters, total_iters));
                        });
                    }
                    SimStatus::Done { state_transitions } => {
                        Grid::new("state_transitions_grid")
                            .num_columns(state_transitions.len())
                            .show(ui, |ui| {
                                for row in state_transitions {
                                    for cell in row {
                                        ui.label(format!("{cell}"));
                                    }
                                    ui.end_row();
                                }
                            });
                    }
                }
            }
        });
    });
}

fn run_sim(
    base_item: ItemState,
    strategy: Strategy,
    num_iters: usize,
    candidate_tiers: &[OpaqueIndex<Tier>],
) -> SimState {
    let candidate_tiers = candidate_tiers.to_vec();
    let status = Arc::new(Mutex::new(SimStatus::Running {
        current_iters: 0,
        total_iters: num_iters,
    }));

    SimState {
        base_item: base_item.clone(),
        strategy: strategy.clone(),
        status: status.clone(),
        handle: thread::spawn(move || {
            let mut state_transitions = vec![vec![0_usize; strategy.0.len()]; strategy.0.len()];
            for _ in 0..num_iters {
                let mut item = base_item.clone();

                let mut finished_state = false;
                let mut prev_state = None;
                while let Some(index) = strategy.get(&item) {
                    // Keep track of state transitions
                    if let Some(prev) = prev_state {
                        let row: &mut Vec<_> = &mut state_transitions[prev];
                        row[index] += 1;
                    }
                    prev_state = Some(index);

                    let Some((omens, currency)) = &strategy.0[index].1 else {
                        // End step, break out
                        finished_state = true;
                        break;
                    };

                    if !currency.can_be_used(&item, &candidate_tiers, omens) {
                        // Condition matched but the currency can't be used on it
                        *status.lock().unwrap() = SimStatus::InvalidCraft {
                            item,
                            currency: currency.clone(),
                            omens: omens.clone(),
                        };
                        return;
                    }

                    currency.craft(&mut item, &candidate_tiers, omens);
                }

                if !finished_state {
                    // Process exited because there was no matching condition
                    *status.lock().unwrap() = SimStatus::NoMatchingState { item };
                    return;
                }
            }

            *status.lock().unwrap() = SimStatus::Done { state_transitions };
        }),
    }
}
