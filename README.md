# This is a crate with my attempt of creating collections in Rust.

## HashTree
Mix of `BTreeMap` and
`HashMap` from standard library: it is a collection of pairs of keys & values, sorted by hash
and internally implemented like a binary tree.

### Using:
```rust
use ghashy_collections::hash_tree::*;

let mut tree = HashTree::new();
tree.insert("Key", "Value");
assert_eq!(tree["Key"], "Value");

```
