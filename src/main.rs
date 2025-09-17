use std::path::Path;

use eframe::egui;
use egui::{Align, Button, Checkbox, Color32, ComboBox, DragValue, Frame, Grid, Layout};
use itertools::Itertools;
use poe_crafting::{
    ITEM_TIERS, MODS, TIERS, init,
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
    types::Affix,
};

#[derive(Debug)]
enum Page {
    ItemBuilder,
    CraftProbability,
}

impl Page {
    const fn all() -> [Self; 2] {
        use Page::*;

        [ItemBuilder, CraftProbability]
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
        let data_root = Path::new("/home/adam/repos/data/poe"); // laptop
        // let data_root = Path::new("/mnt/nvme_4tb/programming/data/poe2"); // desktop
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
                    if ui.button(format!("{:?}", page)).clicked() {
                        self.page = page;
                    }
                }
            })
        });

        match self.page {
            Page::ItemBuilder => self.item_builder(ctx),
            Page::CraftProbability => self.craft_probability(ctx),
        }
    }
}

impl MyEguiApp {
    /// Manually build an item by selecting modifiers
    fn item_builder(&mut self, ctx: &egui::Context) {
        let item_tiers = ITEM_TIERS.get().unwrap();
        let tiers = TIERS.get().unwrap();
        let mods = MODS.get().unwrap();

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
                    self.base_item.mods.retain(|tier_id| {
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
                    let prefixes = self.base_item.mods.iter().filter(|tier_id| {
                        let tier = &tiers[*tier_id];
                        tier.affix == Affix::Prefix
                    });
                    let suffixes = self.base_item.mods.iter().filter(|tier_id| {
                        let tier = &tiers[*tier_id];
                        tier.affix == Affix::Suffix
                    });
                    self.base_item.mods = prefixes
                        .take(max_affixes)
                        .chain(suffixes.take(max_affixes))
                        .cloned()
                        .collect::<Vec<_>>();
                }
            });

            // ============= Mods ====================
            let candidate_tiers = get_valid_mods_for_item(&self.base_item);
            let affix_groups = candidate_tiers
                .iter()
                .map(|tier_id| &tiers[tier_id])
                .sorted_unstable_by_key(|tier| (&tier.affix, &tier.mod_id, &tier.ilvl))
                .chunk_by(|tier| &tier.affix);

            let item_tiers = self
                .base_item
                .mods
                .iter()
                .map(|tier_id| &tiers[tier_id])
                .collect::<Vec<_>>();

            for (affix, group) in &affix_groups {
                let mod_groups = group.chunk_by(|tier| &tier.mod_id);

                ui.heading(format!("{:?}", affix));

                Grid::new(format!("affix_grid_{:?}", affix))
                    .num_columns(3)
                    .show(ui, |ui| {
                        for (mod_id, group) in &mod_groups {
                            ui.label(mod_id);

                            // Tier ilvls
                            let item_tier = item_tiers.iter().find(|tier| &tier.mod_id == mod_id);
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                // https://github.com/emilk/egui/issues/2247
                                // Right alignment not supported in Grid yet, so use right-to-left
                                // & reversed element creation instead
                                let group = group.collect::<Vec<_>>();
                                for tier in group.into_iter().rev() {
                                    let label_text = format!("{:?}", tier.ilvl);

                                    let mut selected = self.base_item.mods.contains(&tier.id);
                                    let was_selected = selected;
                                    ui.add(Checkbox::new(&mut selected, label_text));

                                    if was_selected != selected {
                                        if !selected {
                                            // Mod unselected
                                            self.base_item
                                                .mods
                                                .retain(|tier_id| tier_id != &tier.id);
                                        } else if item_tier.is_some() {
                                            // Mod tier swapped
                                            self.base_item
                                                .mods
                                                .retain(|tier_id| &tiers[tier_id].mod_id != mod_id);
                                            self.base_item.mods.push(tier.id.clone());
                                        } else if !self.base_item.has_room(tier.affix) {
                                            // If we're already at max affixes, do nothing
                                        } else {
                                            // Add a new mod
                                            self.base_item.mods.push(tier.id.clone());
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
    fn craft_probability(&self, ctx: &egui::Context) {
        let item_tiers = ITEM_TIERS.get().unwrap();
        let tiers = TIERS.get().unwrap();
        let mods = MODS.get().unwrap();
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
