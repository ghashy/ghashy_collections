//! This is a crate with my attempt of creating collections in Rust.
//!
//! Now it contains only `HashTree` - a mix of `BTreeMap` and
//! `HashMap`: it is a collection of pairs of keys & values, sorted by hash
//! and internally implemented like a binary tree.
//!
//! Using:
//! ```
//! use ghashy_collections::hash_tree::*;
//!
//! let mut tree = HashTree::new();
//! tree.insert("Key", "Value");
//! assert_eq!(tree["Key"], "Value");
//!
//! ```

// ───── Submodules ───────────────────────────────────────────────────────── //

/// This is a module with `HashTree` related code.
pub mod hash_tree;
