use std::env;
use std::io::BufRead;

fn main() {
    println!("cargo:rerun-if-changed=data/connectome.csv");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = std::path::Path::new(&out_dir).join("connectome_data.rs");

    let csv_path = std::path::Path::new("data/connectome.csv");
    if !csv_path.exists() {
        let fallback = "pub const NEURON_NAMES: &[&str] = &[];\npub const EDGES: &[(u16, u16, u8, u16)] = &[];\npub const NUM_NEURONS: u16 = 0;\npub const NUM_EDGES: usize = 0;\n";
        std::fs::write(&dest_path, fallback).unwrap();
        return;
    }

    let file = std::fs::File::open(csv_path).unwrap();
    let reader = std::io::BufReader::new(file);
    let mut lines = reader.lines();

    let mut name_to_id = std::collections::HashMap::new();
    let mut names: Vec<String> = Vec::new();
    let mut edges: Vec<(u16, u16, u8, u16)> = Vec::new();

    lines.next();

    for line_result in lines {
        let line: String = match line_result {
            Ok(l) => l,
            Err(_) => continue,
        };
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 4 { continue; }
        
        let pre: String = parts[0].trim().to_string();
        let post: String = parts[1].trim().to_string();
        let conn_type_str: &str = parts[2].trim();
        let weight: u16 = parts[3].trim().parse().unwrap_or(1);

        if pre == "LegacyBodyWallMuscles" || post == "LegacyBodyWallMuscles" {
            continue;
        }

        for name in [&pre, &post] {
            if !name_to_id.contains_key(name) {
                let id = name_to_id.len() as u16;
                name_to_id.insert(name.clone(), id);
                names.push(name.clone());
            }
        }

        let pre_id = name_to_id[&pre];
        let post_id = name_to_id[&post];
        let conn_type = if conn_type_str == "chemical" { 0 } else { 1 };

        edges.push((pre_id, post_id, conn_type, weight));
    }

    let mut code = String::new();
    code.push_str(&format!("pub const NUM_NEURONS: u16 = {};\n", names.len()));
    code.push_str(&format!("pub const NUM_EDGES: usize = {};\n\n", edges.len()));
    code.push_str("pub const NEURON_NAMES: &[&str] = &[\n");
    for name in &names {
        code.push_str(&format!("    \"{}\",\n", name));
    }
    code.push_str("];\n\n");
    code.push_str("pub const EDGES: &[(u16, u16, u8, u16)] = &[\n");
    for (pre, post, conn_type, weight) in &edges {
        code.push_str(&format!("    ({}, {}, {}, {}),\n", pre, post, conn_type, weight));
    }
    code.push_str("];\n");

    std::fs::write(&dest_path, &code).unwrap();
    println!("cargo:warning=Generated connectome data: {} neurons, {} edges", names.len(), edges.len());
}