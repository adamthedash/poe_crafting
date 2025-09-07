use std::fs::File;

use poe_crafting::{
    crafting::roll_mod,
    currency::{Augmentation, Currency, Regal, Transmute},
    item_state::{ItemState, Rarity, get_valid_mods_for_item},
    parser::Root,
};

fn main() {
    let root: Root = serde_json::from_reader(
        File::open("/mnt/nvme_4tb/programming/data/poe2/poec_data_altered_json.json")
            .expect("failed to open file"),
    )
    .expect("failed to parse file");

    let mut item = ItemState {
        base_type: &root.bases.seq[root.bases.ind[&4]], // Quiver
        item_level: 75,
        rarity: Rarity::Normal,
        mods: vec![],
    };

    println!("{item:?}");
    let valid_mods = get_valid_mods_for_item(&item, &root);

    let transmute_mods = Transmute::possible_mods(&item, &valid_mods);

    println!("simulating...");
    for _ in 0..100000 {
        let rolled = roll_mod(&transmute_mods);
        // println!("{:?}", rolled);
    }
    println!("done");
}
