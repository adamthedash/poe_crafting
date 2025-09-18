use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use eframe::egui;
use egui::{Align, Checkbox, ComboBox, DragValue, Grid, Layout, ScrollArea};
use itertools::Itertools;
use poe_crafting::{
    ESSENCES, ITEM_TIERS, MODS_HV, TIERS_HV,
    currency::{CURRENCIES, Currency},
    hashvec::OpaqueIndex,
    init,
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
    types::{Affix, OmenId, Tier},
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
        selected_currency: usize,
        selected_omens: HashSet<OmenId>,
        simulation_state: Option<SimState>,
        num_iters_exp: u32,
    },
}

impl Page {
    fn all() -> Vec<Self> {
        use Page::*;

        vec![
            ItemBuilder,
            CraftProbability {
                selected_currency: 0,
                selected_omens: HashSet::new(),
                simulation_state: None,
                num_iters_exp: 2,
            },
        ]
    }

    fn name(&self) -> &str {
        match self {
            Page::ItemBuilder => "Item Builder",
            Page::CraftProbability { .. } => "Craft Probabilities",
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
        }
    }
}

impl MyEguiApp {
    /// Manually build an item by selecting modifiers
    fn item_builder(&mut self, ctx: &egui::Context) {
        let item_tiers = ITEM_TIERS.get().unwrap();
        let tiers = TIERS_HV.get().unwrap();
        let mods = MODS_HV.get().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            // ========== BASE ITEM ==============
            Grid::new("base_grid").num_columns(2).show(ui, |ui| {
                let mut base_items = item_tiers.keys().collect::<Vec<_>>();
                base_items.sort_unstable();

                let mut selected = base_items
                    .iter()
                    .position(|b| **b == self.base_item.base_type)
                    .unwrap();
                let was_selected = selected;

                ui.label("Base Item");
                ComboBox::from_id_salt("combo_base").show_index(
                    ui,
                    &mut selected,
                    base_items.len(),
                    |i| base_items[i],
                );
                self.base_item.base_type = base_items[selected].clone();
                ui.end_row();

                if selected != was_selected {
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
                let old_rarity = self.base_item.rarity;
                ComboBox::from_id_salt("combo_rarity")
                    .selected_text(format!("{:?}", self.base_item.rarity))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.base_item.rarity, Rarity::Normal, "Normal");
                        ui.selectable_value(&mut self.base_item.rarity, Rarity::Magic, "Magic");
                        ui.selectable_value(&mut self.base_item.rarity, Rarity::Rare, "Rare");
                    });
                ui.end_row();
                if self.base_item.rarity != old_rarity {
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
            let candidate_tiers = get_valid_mods_for_item(&self.base_item);
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
                            let item_tier = self
                                .base_item
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

                                    let mut selected = self.base_item.mods.contains(&group_tier_id);
                                    let was_selected = selected;
                                    ui.add(Checkbox::new(&mut selected, label_text));

                                    if was_selected != selected {
                                        if !selected {
                                            // Mod unselected
                                            self.base_item
                                                .mods
                                                .retain(|&tier_id| tier_id != group_tier_id);
                                        } else if item_tier.is_some() {
                                            // Mod tier swapped
                                            self.base_item
                                                .mods
                                                .retain(|&tier_id| tiers[tier_id].mod_id != mod_id);
                                            self.base_item.mods.push(group_tier_id);
                                        } else if !self.base_item.has_room(tier.affix) {
                                            // If we're already at max affixes, do nothing
                                        } else {
                                            // Add a new mod
                                            self.base_item.mods.push(group_tier_id);
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

        let item_tiers = ITEM_TIERS.get().unwrap();
        let tiers = TIERS_HV.get().unwrap();
        let mods = MODS_HV.get().unwrap();

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
            let old_selected = *selected_currency;
            ComboBox::from_id_salt("currency_select").show_index(
                ui,
                selected_currency,
                currencies.len(),
                |i| currencies[i].name(),
            );
            let currency = currencies[*selected_currency];
            if old_selected != *selected_currency {
                // Currency changed, clear omens
                selected_omens.clear();
            }

            // Select Omens
            let mut omens = currency
                .possible_omens()
                .into_iter()
                // Only omens that can be used
                .filter(|omen| {
                    currency.can_be_used(
                        &self.base_item,
                        &candidate_tiers,
                        &HashSet::from_iter(std::iter::once(omen.clone())),
                    )
                })
                .collect::<Vec<_>>();
            omens.sort();

            ui.horizontal(|ui| {
                for omen_id in omens {
                    // Individual omen buttons
                    let mut selected = selected_omens.contains(&omen_id);
                    let old_selected = selected;
                    ui.checkbox(&mut selected, &omen_id);
                    match (old_selected, selected) {
                        (true, false) => {
                            // Un-selected
                            selected_omens.remove(&omen_id);
                        }
                        (false, true) => {
                            // Selected
                            selected_omens.insert(omen_id.clone());
                        }
                        _ => (),
                    }
                }
            });

            // 10^N iterations
            ui.add(
                DragValue::new(num_iters_exp)
                    .range(0..=10)
                    .custom_formatter(|n, _| format!("{:?}", 10_u64.pow(n as u32))),
            );
            let n = 10_u64.pow(*num_iters_exp);

            // Simulation
            if ui.button("Go!").clicked() {
                let status = Arc::new(Mutex::new(SimStatus::Running { iterations_done: 0 }));
                let state = SimState {
                    base_item: self.base_item.clone(),
                    status: status.clone(),
                    handle: thread::spawn({
                        let base_item = self.base_item.clone();
                        let currency = currency.clone();
                        let candidate_tiers = candidate_tiers.clone();
                        let selected_omens = selected_omens.clone();

                        move || {
                            let mut results = HashMap::<_, usize>::new();
                            let before_mods =
                                base_item.mods.iter().copied().collect::<HashSet<_>>();
                            for _ in 0..n {
                                // Apply the currency
                                let mut item = base_item.clone();
                                currency.craft(&mut item, &candidate_tiers, &selected_omens);

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
                };
                *simulation_state = Some(state);
            }

            if let Some(sim_state) = simulation_state {
                match &*sim_state.status.lock().unwrap() {
                    SimStatus::Done { results } => {
                        let affix_groups = results
                            .iter()
                            .map(|(&tier_id, &count)| (&tiers[tier_id], count))
                            .sorted_unstable_by_key(|(tier, _)| {
                                (tier.affix, &tier.mod_id, tier.ilvl)
                            })
                            .chunk_by(|(tier, _)| tier.affix);

                        ScrollArea::new([false, true]).show(ui, |ui| {
                            for (affix, mod_group) in &affix_groups {
                                ui.heading(format!("{:?}", affix));
                                Grid::new(format!("results_grid_{:?}", affix))
                                    .num_columns(2)
                                    .show(ui, |ui| {
                                        for (&mod_id, group) in
                                            &mod_group.chunk_by(|(tier, _)| &tier.mod_id)
                                        {
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
                                                    tier_counts.into_iter().for_each(
                                                        |(_, count)| {
                                                            ui.label(format!(
                                                                "{:.1}%",
                                                                (count as f32 / n as f32) * 100.
                                                            ));
                                                        },
                                                    );
                                                });

                                            ui.end_row();
                                        }
                                    });
                            }
                        });
                    }
                    SimStatus::Running { iterations_done } => {
                        ui.spinner();
                        ui.label(format!("{} / {}", iterations_done, n));
                    }
                }
            }
            ui.label(format!("{:?}", simulation_state))
        });
    }
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
