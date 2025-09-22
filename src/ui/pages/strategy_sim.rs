use crate::ui::Page;
use std::collections::HashSet;

use crate::{
    ESSENCES, MODS_HV, TIERS_HV,
    currency::{CURRENCIES, Currency, CurrencyType},
    hashvec::OpaqueIndex,
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
    strategy::{Condition, ConditionGroup, ModifierCondition},
    types::{Modifier, Tier},
    ui::{dropdown, multi_select_checkboxes, omen_selection, range_selector, rarity_dropdown},
};
use egui::{self, CentralPanel, Color32, Frame, Ui};
use itertools::Itertools;

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

pub fn show_page(page_state: &mut Page, ctx: &egui::Context, item: &ItemState) {
    let tiers = TIERS_HV.get().unwrap();

    let Page::StrategyBuilder { strategy } = page_state else {
        unreachable!()
    };

    let candidate_tiers = get_valid_mods_for_item(item);
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
                let remove = show_strategy_step(ui, &format!("{i}"), condition, &candidate_mods)
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
                    let old_selected = dropdown(
                        ui,
                        currency,
                        &currencies,
                        &format!("currency_select_{i}"),
                        |c| c.name().to_string(),
                    );
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

        // Strategy simulation
        if ui.button("Go!").clicked() {
            let mut item = item.clone();

            let mut state_transitions = vec![vec![0_usize; strategy.0.len()]; strategy.0.len()];
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

                assert!(
                    currency.can_be_used(&item, &candidate_tiers, omens),
                    "Currency cannot be used in current state!"
                );

                currency.craft(&mut item, &candidate_tiers, omens);
            }

            if !finished_state {
                // Process exited because there was no matching condition
                ui.label("No matching condition!");
                println!("{}", item);
            }

            println!("{:#?}", state_transitions);
        }
    });
}
