use std::{fs, path::Path};

use poe_crafting::parsers::{dat::Dats, poe2db, stat_desc};

fn main() {
    let data_root = Path::new("/home/adam/repos/data/poe"); // laptop
    let bake_path = Path::new("/home/adam/repos/rust/poe_crafting/data");

    // let data_root = Path::new("/mnt/nvme_4tb/programming/data/poe2"); // desktop
    // let bake_path = Path::new("/mnt/nvme_4tb/programming/rust/poe_crafting/data");

    let dats = Dats::load_tables(&data_root.join("tables"));
    dats.save_to_csv(&bake_path.join("tables"));

    let stat_desc = stat_desc::load(&data_root.join("stat_descriptions.json"));
    stat_desc::save(&bake_path.join("stat_descriptions.json"), &stat_desc);

    fs::create_dir_all(bake_path.join("coe")).unwrap();
    let poe2db = poe2db::load(&data_root.join("coe/poe2db_data_altered_weights.json"));
    poe2db::save(
        &bake_path.join("coe/poe2db_data_altered_weights.json"),
        &poe2db,
    );
}
