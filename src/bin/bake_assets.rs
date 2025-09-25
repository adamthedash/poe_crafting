use std::path::Path;

use poe_crafting::parsers::dat::Dats;

fn main() {
    let data_root = Path::new("/home/adam/repos/data/poe"); // laptop
    let bake_path = Path::new("/home/adam/repos/rust/poe_crafting/data");

    let dats = Dats::load_tables(&data_root.join("tables"));
    dats.save_to_csv(&bake_path.join("tables"));
}
