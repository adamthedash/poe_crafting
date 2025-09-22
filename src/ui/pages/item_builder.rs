use crate::{
    ITEM_TIERS, MODS_HV, TIERS_HV,
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
    types::Affix,
    ui::{dropdown, rarity_dropdown},
};
use egui::{self, Align, Checkbox, DragValue, Grid, Layout, Ui};
use itertools::Itertools;

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

pub fn show_page(ctx: &egui::Context, item: &mut ItemState) {
    let item_tiers = ITEM_TIERS.get().unwrap();
    let tiers = TIERS_HV.get().unwrap();

    egui::CentralPanel::default().show(ctx, |ui| {
        // ========== BASE ITEM ==============
        Grid::new("base_grid").num_columns(2).show(ui, |ui| {
            let mut base_items = item_tiers.keys().collect::<Vec<_>>();
            base_items.sort_unstable();

            ui.label("Base Item");
            let old_base = dropdown(ui, &mut item.base_type, &base_items, "combo_base", |b| {
                b.clone()
            });
            ui.end_row();

            if old_base.is_some() {
                // Base changed, clear mods from item
                item.mods.clear();
            }

            // Ilvl
            let old_ilvl = item.item_level;
            ui.label("Item Level");
            ui.add(DragValue::new(&mut item.item_level).range(1..=100));
            ui.end_row();

            if item.item_level != old_ilvl {
                // Ilvl changed, filter mods that are now too high
                item.mods.retain(|&tier_id| {
                    let tier = &tiers[tier_id];
                    tier.ilvl >= item.item_level
                });
            }

            // Rarity
            ui.label("Rarity");
            let old_rarity = rarity_dropdown(ui, &mut item.rarity, "rarity_dropdown");
            ui.end_row();

            if old_rarity.is_some() {
                // Rarity changed, limit the mods
                let max_affixes = match item.rarity {
                    Rarity::Normal => 0,
                    Rarity::Magic => 1,
                    Rarity::Rare => 3,
                };
                let prefixes = item.mods.iter().copied().filter(|tier_id| {
                    let tier = &tiers[*tier_id];
                    tier.affix == Affix::Prefix
                });
                let suffixes = item.mods.iter().copied().filter(|tier_id| {
                    let tier = &tiers[*tier_id];
                    tier.affix == Affix::Suffix
                });
                item.mods = prefixes
                    .take(max_affixes)
                    .chain(suffixes.take(max_affixes))
                    .collect::<Vec<_>>();
            }
        });

        // ============= Mods ====================
        display_mod_select_grid(ui, item);
    });
}
