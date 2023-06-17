use std::collections::BTreeMap;

use ghashy_collections::hash_tree::*;

// ───── Body ─────────────────────────────────────────────────────────────── //

fn main() {
    test_visually();

    println!("\nBenchmark: \n");
    benchmark();
}

fn benchmark() {
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

fn test_visually() {
    let mut tree = HashTree::new_with_seed(1);

    // These are random words
    tree.insert("mecha", "mechaV");
    tree.insert("Jaeger", "JaegerV");
    tree.insert("droid", "droidV");
    tree.insert("GingerBread", "GingerBreadV");
    tree.insert("Android", "AndroidV");
    tree.insert("robot", "robotV");
    tree.insert("robot", "robotV");
    tree.insert("ios", "iosV");
    tree.insert("windows", "windowsV");
    tree.insert("linux", "linuxV");
    tree.insert("blackberry", "blackberryV");
    tree.insert("bevy", "bevyV");
    tree.insert("rust", "rustV");
    tree.insert("meow", "meowV");
    tree.insert("black", "blackV");
    tree.insert("red", "redV");
    tree.insert("purple", "purpleV");
    tree.insert("green", "greenV");
    tree.insert("blue", "blueV");
    tree.insert("sun", "sunV");
    tree.insert("moon", "moonV");
    tree.insert("earth", "earthV");
    tree.insert("sword", "swordV");
    tree.insert("bow", "bowV");
    tree.insert("knife", "knifeV");
    tree.insert("t", "tV");

    tree.remove("droid");

    dbg!(tree);
}
