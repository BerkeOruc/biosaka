#[path = "build/herm.rs"]
mod herm;
#[path = "build/male.rs"]
mod male;
#[path = "build/gen.rs"]
mod gen;

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=data/connectome.csv");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("connectome_data.rs");

    let csv_path = std::path::Path::new("data/connectome.csv");
    let (herm_names, herm_edges) = if csv_path.exists() {
        herm::load_csv(csv_path)
    } else {
        (Vec::new(), Vec::new())
    };

    let (male_names, male_edges) = male::build_male_connectome(&herm_names, &herm_edges);

    let mut file = std::fs::File::create(&dest_path).unwrap();
    gen::write_connectome_data(&mut file, &herm_names, &herm_edges, &male_names, &male_edges)
        .unwrap();

    println!(
        "cargo:warning=Herm: {} neurons, {} edges | Male: {} neurons, {} edges",
        herm_names.len(),
        herm_edges.len(),
        male_names.len(),
        male_edges.len()
    );
}
