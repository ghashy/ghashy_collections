use std::collections::BTreeMap;

use ghashy_collections::hash_tree::*;

// ───── Body ─────────────────────────────────────────────────────────────── //

fn main() {
    benchmark();
}

fn benchmark() {
    // let size = 100;
    let size = 1000000;

    // HashTree
    let mut now = std::time::Instant::now();
    let mut tree = HashTree::new();
    for index in 0..size {
        tree.insert(index, index + 1);
    }
    let _ = tree.get(&99999);

    println!("HashTree: {} microseconds", now.elapsed().as_micros());

    // BTreeMap
    now = std::time::Instant::now();
    let mut tree = BTreeMap::new();
    for index in 0..size {
        tree.insert(index, index + 1);
    }
    let _ = tree.get(&99999);
    println!("BTreeMap: {} microseconds", now.elapsed().as_micros());

    // HashMap
    now = std::time::Instant::now();
    let mut tree = std::collections::HashMap::new();
    for index in 0..size {
        tree.insert(index, index + 1);
    }
    let _ = tree.get(&99999);
    println!("HashMap: {} microseconds", now.elapsed().as_micros());

    // HashMap with custom hasher
    now = std::time::Instant::now();
    let mut tree =
        std::collections::HashMap::with_hasher(ahash::RandomState::new());
    for index in 0..size {
        tree.insert(index, index + 1);
    }
    let _ = tree.get(&99999);
    println!(
        "HashMap with ahash: {} microseconds",
        now.elapsed().as_micros()
    );
}
