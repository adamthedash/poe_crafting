use std::{collections::HashSet, path::Path};

use poe_crafting::{
    ESSENCES, ITEM_TIERS, TIERS_HV,
    currency::{CURRENCIES, Currency},
    init,
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
};
use random_choice::random_choice;

fn main() {
    // let data_root = Path::new("/home/adam/repos/data/poe"); // laptop
    let data_root = Path::new("/mnt/nvme_4tb/programming/data/poe2"); // desktop
    init(data_root);

    let item_tiers = ITEM_TIERS.get().unwrap();

    let bases = item_tiers.keys().collect::<Vec<_>>();
    let weights = vec![1.; bases.len()];

    for _ in 0..100 {
        let base_type = random_choice().random_choice_f32(&bases, &weights, 1)[0];
        let mut item = ItemState {
            base_type: (*base_type).clone(),
            item_level: 75,
            rarity: Rarity::Normal,
            mods: vec![],
        };
        let candidate_tiers = get_valid_mods_for_item(&item);

        for _ in 0..100 {
            // Select a random currency
            let currencies = CURRENCIES
                .iter()
                .chain(ESSENCES.get().unwrap())
                .filter(|c| c.can_be_used(&item, &candidate_tiers, &HashSet::new()))
                .collect::<Vec<_>>();
            let weights = vec![1.; currencies.len()];
            let currency = random_choice().random_choice_f32(&currencies, &weights, 1)[0];

            // Select random omens
            let omens = currency
                .possible_omens()
                .into_iter()
                // Only omens that can be used
                .filter(|omen| {
                    currency.can_be_used(
                        &item,
                        &candidate_tiers,
                        &HashSet::from_iter(std::iter::once(omen.clone())),
                    )
                })
                .collect::<Vec<_>>();
            let weights = vec![1.; omens.len()];
            let omens = HashSet::from_iter(
                random_choice()
                    .random_choice_f32(&omens, &weights, 1)
                    .into_iter()
                    .map(|o| (*o).clone()),
            );

            // println!("{:?} {:?}", omens, currency);
            let before = item.clone();
            currency.craft(&mut item, &candidate_tiers, &omens);
            if !item.is_valid() {
                println!("invalid item");
                before.print_item();
                println!("- {:?} {:?} ->", omens, currency);
                item.print_item();
                panic!()
            }
        }

        // item.print_item();
        // println!();
    }
}
