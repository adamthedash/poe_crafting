use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use eframe::egui;
use egui::{
    Align, CentralPanel, Checkbox, Color32, ComboBox, DragValue, Frame, Grid, Layout, ScrollArea,
    Ui,
};
use itertools::Itertools;
use poe_crafting::{
    ESSENCES, ITEM_TIERS, MODS_HV, TIERS_HV,
    currency::{CURRENCIES, Currency, CurrencyType},
    hashvec::OpaqueIndex,
    init,
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
    strategy::{Condition, ConditionGroup, ModifierCondition, Strategy},
    types::{Affix, Modifier, Omen, Tier},
    ui::{dropdown, multi_select_checkboxes, omen_selection, range_selector, rarity_dropdown},
};

#[derive(Debug)]
enum SimStatus {
    Done {
        results: HashMap<OpaqueIndex<Tier>, usize>,
    },
    Running {
        iterations_done: usize,
    },
}

#[derive(Debug)]
struct SimState {
    base_item: ItemState,
    status: Arc<Mutex<SimStatus>>,
    handle: JoinHandle<()>,
}

#[derive(Debug)]
enum Page {
    ItemBuilder,
    CraftProbability {
        selected_currency: CurrencyType,
        selected_omens: HashSet<Omen>,
        simulation_state: Option<SimState>,
        num_iters_exp: u32,
    },
    StrategyBuilder {
        strategy: Strategy,
    },
}

impl Page {
    fn all() -> Vec<Self> {
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
            },
        ]
    }

    fn name(&self) -> &str {
        match self {
            Page::ItemBuilder => "Item Builder",
            Page::CraftProbability { .. } => "Craft Probabilities",
            Page::StrategyBuilder { .. } => "Strategy Builder",
        }
    }
}

struct MyEguiApp {
    base_item: ItemState,
    page: Page,
}

impl Default for MyEguiApp {
    fn default() -> Self {
        Self {
            base_item: ItemState {
                base_type: "Amulet".to_string(),
                item_level: 100,
                rarity: Rarity::Normal,
                mods: vec![],
            },
            page: Page::ItemBuilder,
        }
    }
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // let data_root = Path::new("/home/adam/repos/data/poe"); // laptop
        let data_root = Path::new("/mnt/nvme_4tb/programming/data/poe2"); // desktop
        init(data_root);

        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                for page in Page::all() {
                    if ui.button(page.name()).clicked() {
                        self.page = page;
                    }
                }
            })
        });

        match self.page {
            Page::ItemBuilder => self.item_builder(ctx),
            Page::CraftProbability { .. } => self.craft_probability(ctx),
            Page::StrategyBuilder { .. } => self.strategy_builder(ctx),
        }
    }
}

impl MyEguiApp {
    /// Manually build an item by selecting modifiers
    fn item_builder(&mut self, ctx: &egui::Context) {
        let item_tiers = ITEM_TIERS.get().unwrap();
        let tiers = TIERS_HV.get().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            // ========== BASE ITEM ==============
            Grid::new("base_grid").num_columns(2).show(ui, |ui| {
                let mut base_items = item_tiers.keys().collect::<Vec<_>>();
                base_items.sort_unstable();

                ui.label("Base Item");
                let old_base = dropdown(
                    ui,
                    &mut self.base_item.base_type,
                    &base_items,
                    "combo_base",
                    |b| b.clone(),
                );
                ui.end_row();

                if old_base.is_some() {
                    // Base changed, clear mods from item
                    self.base_item.mods.clear();
                }

                // Ilvl
                let old_ilvl = self.base_item.item_level;
                ui.label("Item Level");
                ui.add(DragValue::new(&mut self.base_item.item_level).range(1..=100));
                ui.end_row();

                if self.base_item.item_level != old_ilvl {
                    // Ilvl changed, filter mods that are now too high
                    self.base_item.mods.retain(|&tier_id| {
                        let tier = &tiers[tier_id];
                        tier.ilvl >= self.base_item.item_level
                    });
                }

                // Rarity
                ui.label("Rarity");
                let old_rarity = rarity_dropdown(ui, &mut self.base_item.rarity, "rarity_dropdown");
                ui.end_row();

                if old_rarity.is_some() {
                    // Rarity changed, limit the mods
                    let max_affixes = match self.base_item.rarity {
                        Rarity::Normal => 0,
                        Rarity::Magic => 1,
                        Rarity::Rare => 3,
                    };
                    let prefixes = self.base_item.mods.iter().copied().filter(|tier_id| {
                        let tier = &tiers[*tier_id];
                        tier.affix == Affix::Prefix
                    });
                    let suffixes = self.base_item.mods.iter().copied().filter(|tier_id| {
                        let tier = &tiers[*tier_id];
                        tier.affix == Affix::Suffix
                    });
                    self.base_item.mods = prefixes
                        .take(max_affixes)
                        .chain(suffixes.take(max_affixes))
                        .collect::<Vec<_>>();
                }
            });

            // ============= Mods ====================
            display_mod_select_grid(ui, &mut self.base_item);
        });
    }

    /// Show the outcome distributions when crafting on a given item
    fn craft_probability(&mut self, ctx: &egui::Context) {
        // Unpack state variables
        let Page::CraftProbability {
            selected_currency,
            selected_omens,
            simulation_state,
            num_iters_exp,
        } = &mut self.page
        else {
            unreachable!()
        };

        let candidate_tiers = get_valid_mods_for_item(&self.base_item);

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
            .filter(|c| c.can_be_used(&self.base_item, &candidate_tiers, &HashSet::new()))
            .collect::<Vec<_>>();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Select Currency
            let old_selected =
                dropdown(ui, selected_currency, &currencies, "currency_select", |c| {
                    c.name().to_string()
                });
            if old_selected.is_some() {
                // Currency changed, clear omens
                selected_omens.clear();
            }

            // Select Omens
            omen_selection(ui, selected_currency, selected_omens, Some(&self.base_item));

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
                    self.base_item.clone(),
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

    fn strategy_builder(&mut self, ctx: &egui::Context) {
        let tiers = TIERS_HV.get().unwrap();

        let Page::StrategyBuilder { strategy } = &mut self.page else {
            unreachable!()
        };

        let candidate_tiers = get_valid_mods_for_item(&self.base_item);
        let candidate_mods = candidate_tiers
            .iter()
            .sorted_unstable_by_key(|&&tier_id| {
                let tier = &tiers[tier_id];

                (tier.mod_id, tier.ilvl)
            })
            .chunk_by(|&&tier_id| {
                let tier = &tiers[tier_id];

                tier.mod_id
            });
        let candidate_mods = candidate_mods
            .into_iter()
            .map(|(mod_id, group)| (mod_id, group.copied().collect::<Vec<_>>()))
            .collect::<Vec<_>>();

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
            .collect::<Vec<_>>();

        CentralPanel::default().show(ctx, |ui| {
            let to_remove = strategy
                .0
                .iter_mut()
                .enumerate()
                .flat_map(|(i, (condition, action))| {
                    // Condition
                    let remove =
                        show_strategy_step(ui, &format!("{i}"), condition, &candidate_mods)
                            .then_some(i);

                    // Action
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
                        let old_selected =
                            dropdown(ui, currency, &currencies, "currency_select", |c| {
                                c.name().to_string()
                            });
                        if old_selected.is_some() {
                            // Currency changed, clear omens
                            selected_omens.clear();
                        }

                        // Select Omens
                        omen_selection(ui, currency, selected_omens, None);
                    } else {
                        // No action - end state
                        *action = None;
                    }

                    remove
                })
                .next();

            if let Some(index) = to_remove {
                strategy.0.remove(index);
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

            // Action
        });
    }
}

fn show_strategy_step(
    ui: &mut Ui,
    key: &str,
    condition: &mut Condition,
    candidate_mods: &[(OpaqueIndex<Modifier>, Vec<OpaqueIndex<Tier>>)],
) -> bool {
    Frame::default()
        .fill(Color32::DARK_RED)
        .show(ui, |ui| {
            // Button to remove this step
            let remove = ui.button("X").clicked();

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

            remove
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
                    "Not" => ConditionGroup::Not(HashSet::new()),
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
                ConditionGroup::Not(hash_set) => todo!(),
                ConditionGroup::AffixCount {
                    suffixes,
                    prefixes,
                    affixes,
                } => todo!(),
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
            |mod_id| {
                let mods = MODS_HV.get().unwrap();
                mods[*mod_id].group.clone()
            },
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
            .map(|&tier_id| {
                let tiers = TIERS_HV.get().unwrap();
                &tiers[tier_id].ilvl
            })
            .collect::<Vec<_>>();

        multi_select_checkboxes(ui, &mut mod_condition.levels, &group_ilvls, |ilvl| {
            format!("{}", ilvl)
        });

        remove
    })
    .inner
}

/// A grid of all the mods that can roll on the item with some checkboxes to let the user modify
/// the item
fn display_mod_select_grid(ui: &mut Ui, item: &mut ItemState) {
    let tiers = TIERS_HV.get().unwrap();
    let mods = MODS_HV.get().unwrap();

    let candidate_tiers = get_valid_mods_for_item(item);

    let affix_groups = candidate_tiers
        .iter()
        .map(|&tier_id| &tiers[tier_id])
        .sorted_unstable_by_key(|tier| (&tier.affix, &tier.mod_id, &tier.ilvl))
        .chunk_by(|tier| &tier.affix);

    for (affix, group) in &affix_groups {
        let mod_groups = group.chunk_by(|tier| &tier.mod_id);

        ui.heading(format!("{:?}", affix));

        Grid::new(format!("affix_grid_{:?}", affix))
            .num_columns(3)
            .show(ui, |ui| {
                for (&mod_id, group) in &mod_groups {
                    ui.label(&mods[mod_id].group);

                    // Tier ilvls
                    let item_tier = item
                        .mods
                        .iter()
                        .copied()
                        .find(|&tier_id| tiers[tier_id].mod_id == mod_id);

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        // https://github.com/emilk/egui/issues/2247
                        // Right alignment not supported in Grid yet, so use right-to-left
                        // & reversed element creation instead
                        let group = group.collect::<Vec<_>>();
                        for tier in group.into_iter().rev() {
                            let group_tier_id = tiers.get_opaque(&tier.id);

                            let label_text = format!("{:?}", tier.ilvl);

                            let mut selected = item.mods.contains(&group_tier_id);
                            let was_selected = selected;
                            ui.add(Checkbox::new(&mut selected, label_text));

                            if was_selected != selected {
                                if !selected {
                                    // Mod unselected
                                    item.mods.retain(|&tier_id| tier_id != group_tier_id);
                                } else if item_tier.is_some() {
                                    // Mod tier swapped
                                    item.mods.retain(|&tier_id| tiers[tier_id].mod_id != mod_id);
                                    item.mods.push(group_tier_id);
                                } else if !item.has_room(tier.affix) {
                                    // If we're already at max affixes, do nothing
                                } else {
                                    // Add a new mod
                                    item.mods.push(group_tier_id);
                                }
                            }
                        }
                    });

                    // Tags
                    let modifier = &mods[mod_id];
                    ui.horizontal(|ui| {
                        for tag in &modifier.tags {
                            ui.label(tag);
                        }
                    });

                    ui.end_row();
                }
            });
    }
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
        base_item: base_item.clone(),
        status: status.clone(),
        handle: thread::spawn({
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
    let tiers = TIERS_HV.get().unwrap();
    let mods = MODS_HV.get().unwrap();

    let total_iters = results.values().sum::<usize>();

    let affix_groups = results
        .iter()
        .map(|(&tier_id, &count)| (&tiers[tier_id], count))
        .sorted_unstable_by_key(|(tier, _)| (tier.affix, &tier.mod_id, tier.ilvl))
        .chunk_by(|(tier, _)| tier.affix);

    ScrollArea::new([false, true]).show(ui, |ui| {
        for (affix, mod_group) in &affix_groups {
            ui.heading(format!("{:?}", affix));
            Grid::new(format!("results_grid_{:?}", affix))
                .num_columns(2)
                .show(ui, |ui| {
                    for (&mod_id, group) in &mod_group.chunk_by(|(tier, _)| &tier.mod_id) {
                        let modifier = &mods[mod_id];
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

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))),
    )
    .unwrap();
}
