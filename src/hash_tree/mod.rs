#![deny(
    warnings,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unstable_features,
    unsafe_code,
    unused_import_braces,
    unused_qualifications,
    missing_docs
)]

// ───── TreePointer && TreeNode ──────────────────────────────────────────── //

struct TreeNode<K, V> {
    hash: u64,
    key: K,
    value: V,
    left: TreePointer<K, V>,
    right: TreePointer<K, V>,
}

enum TreePointer<K, V> {
    Empty,
    NonEmpty(Box<TreeNode<K, V>>),
}

impl<K, V> AsRef<Box<TreeNode<K, V>>> for TreePointer<K, V> {
    fn as_ref(&self) -> &Box<TreeNode<K, V>> {
        match *self {
            TreePointer::Empty => panic!(),
            TreePointer::NonEmpty(ref node) => node,
        }
    }
}

impl<K, V> AsMut<Box<TreeNode<K, V>>> for TreePointer<K, V> {
    fn as_mut(&mut self) -> &mut Box<TreeNode<K, V>> {
        match *self {
            TreePointer::NonEmpty(ref mut node) => node,
            TreePointer::Empty => panic!(),
        }
    }
}

impl<K, V> TreePointer<K, V> {
    fn new(left: Self, key: K, hash: u64, value: V, right: Self) -> Self {
        TreePointer::NonEmpty(Box::new(TreeNode {
            hash,
            key,
            value,
            left,
            right,
        }))
    }

    fn take(&mut self) -> Self {
        match self {
            TreePointer::Empty => Self::Empty,
            TreePointer::NonEmpty(_) => {
                std::mem::replace(self, TreePointer::Empty)
            }
        }
    }

    fn unwrap(self) -> Box<TreeNode<K, V>> {
        match self {
            TreePointer::NonEmpty(val) => val,
            TreePointer::Empty => {
                panic!("called `TreePointer::unwrap()` on a `Empty` value")
            }
        }
    }

    fn is_non_empty(&self) -> bool {
        match self {
            TreePointer::Empty => false,
            TreePointer::NonEmpty(_) => true,
        }
    }

    fn replace(&mut self, new: Self) -> Self {
        std::mem::replace(self, new)
    }

    fn extract_min(&mut self) -> Option<(K, u64, V)> {
        let mut node = None;

        if self.is_non_empty() {
            let mut current = self;

            while current.as_ref().left.is_non_empty() {
                current = &mut current.as_mut().left;
            }

            let temp = current.take().unwrap();
            node = Some((temp.key, temp.hash, temp.value));
            let _ = std::mem::replace(current, temp.right);
        }
        node
    }

    fn remove(&mut self, hash: u64) -> Option<Self> {
        use std::cmp::Ordering::*;
        use TreePointer::*;

        let mut current = self;

        let mut result = None;

        // A destructuring of `current` into its components. This doesn't
        // borrow `current` as a whole, it just borrows `node`.
        // So we can do this: current = &mut current.as_mut().*, inside the
        // cycle.
        while let NonEmpty(ref mut node) = current {
            match node.hash.cmp(&hash) {
                Less => current = &mut current.as_mut().right,
                Greater => current = &mut current.as_mut().left,
                Equal => match (&node.left, &node.right) {
                    (Empty, Empty) => {
                        // Removing edge node, easiest case
                        result = Some(current.replace(Empty));
                    }
                    (NonEmpty(_), Empty) => {
                        // Replace current with left node, if right is `Empty`
                        let take = node.left.take();
                        result = Some(current.replace(take));
                    }
                    (Empty, NonEmpty(_)) => {
                        // Same, but with right node
                        let take = node.right.take();
                        result = Some(current.replace(take));
                    }
                    (NonEmpty(_), NonEmpty(_)) => {
                        // Complicated case
                        //
                        // Take our minimal node from right, write it's
                        // data into `temp` variable, and then place on it's
                        // place it's right node.
                        let mut temp = node.right.extract_min().unwrap();
                        let cur = current.as_mut();
                        // Write our taken data into target node
                        std::mem::swap(&mut cur.key, &mut temp.0);
                        std::mem::swap(&mut cur.hash, &mut temp.1);
                        std::mem::swap(&mut cur.value, &mut temp.2);

                        // Return removed data
                        result = Some(NonEmpty(Box::new(TreeNode {
                            key: temp.0,
                            hash: temp.1,
                            value: temp.2,
                            left: Empty,
                            right: Empty,
                        })));
                    }
                },
            }
        }
        result
    }

    fn iter(&self) -> TreeIter<K, V> {
        let mut iter = TreeIter {
            unvisited: Vec::new(),
        };
        iter.push_left_edge(self);
        iter
    }

    fn debug_iter(&self) -> TreeDebugIter<K, V> {
        let mut iter = TreeDebugIter {
            unvisited: Vec::new(),
        };
        iter.push_left_edge(self);
        iter
    }
}

impl<'a, K: 'a, V: 'a> IntoIterator for &'a TreePointer<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = TreeIter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// ───── TreeIter ─────────────────────────────────────────────────────────── //

/// State of symmetrical iteration of `BinaryTree`
pub struct TreeIter<'a, K: 'a, V: 'a> {
    /// Stack references to `TreeNode`'s.
    ///
    /// Since we are using methods `push` and
    /// `pop` of type Vec, the top of the stack is the end of the Vec.
    ///
    /// Node, which will be next in iteration, is placing on top of the stack,
    /// but his ancestors, which were not visited by iteration - on the bottom.
    /// If the stack is empty, iteration is finished.
    unvisited: Vec<&'a TreeNode<K, V>>,
}

impl<'a, K: 'a, V: 'a> TreeIter<'a, K, V> {
    fn push_left_edge(&mut self, mut tree_ptr: &'a TreePointer<K, V>) {
        while let TreePointer::NonEmpty(ref node) = *tree_ptr {
            self.unvisited.push(node.as_ref());
            tree_ptr = &node.left;
        }
    }
}

impl<'a, K, V> Iterator for TreeIter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        // Find node, which will be returned by this iteration, or stop
        // iteration.
        let node = match self.unvisited.pop() {
            None => return None,
            Some(n) => n,
        };

        // Next node will be the leftmost descedant of right son of this node,
        // so place the path to him in the stack.
        self.push_left_edge(&node.right);

        // Create the reference to the value of this node
        Some((&node.key, &node.value))
    }
}

// ───── TreeDebugIter ────────────────────────────────────────────────────── //

/// State of symmetrical iteration of `BinaryTree`
struct TreeDebugIter<'a, K: 'a, V: 'a> {
    /// Stack references to `TreeNode`'s.
    ///
    /// Since we are using methods `push` and
    /// `pop` of type Vec, the top of the stack is the end of the Vec.
    ///
    /// Node, which will be next in iteration, is placing on top of the stack,
    /// but his ancestors, which were not visited by iteration - on the bottom.
    /// If the stack is empty, iteration is finished.
    unvisited: Vec<&'a TreeNode<K, V>>,
}

impl<'a, K: 'a, V: 'a> TreeDebugIter<'a, K, V> {
    fn push_left_edge(&mut self, mut tree_ptr: &'a TreePointer<K, V>) {
        while let TreePointer::NonEmpty(ref node) = *tree_ptr {
            self.unvisited.push(node.as_ref());
            tree_ptr = &node.left;
        }
    }
}

impl<'a, K, V> Iterator for TreeDebugIter<'a, K, V> {
    type Item = &'a TreeNode<K, V>;
    fn next(&mut self) -> Option<Self::Item> {
        // Find node, which will be returned by this iteration, or stop
        // iteration.
        let node = match self.unvisited.pop() {
            None => return None,
            Some(n) => n,
        };

        // Next node will be the leftmost descedant of right son of this node,
        // so place the path to him in the stack.
        self.push_left_edge(&node.right);

        // Create the reference to the value of this node
        Some(node)
    }
}

// ───── HashTree ─────────────────────────────────────────────────────────── //

/// `HashTree` is a collection of pairs which are sorted by hash,
/// generated for every key.
pub struct HashTree<K, V> {
    root: TreePointer<K, V>,
    state: ahash::RandomState,
}

impl<K, V> HashTree<K, V>
where
    K: core::hash::Hash + Eq,
{
    /// Create new empty `HashTree`.
    pub fn new() -> Self {
        let state = ahash::RandomState::new();
        HashTree {
            root: TreePointer::Empty,
            state,
        }
    }

    /// Create new empty `HashTree` with custom seed. It will always hash same
    /// keys with the same hashes, so the order of elements in binary tree will
    /// be preserved. May be useful for serialization.
    pub fn new_with_seed(seed: u64) -> Self {
        let state = ahash::RandomState::with_seeds(seed, seed, seed, seed);
        HashTree {
            root: TreePointer::Empty,
            state,
        }
    }

    /// Insert an element to a `HashTree`. If a value is already present in the
    /// `HashTree`, the old value is returned, otherwise None is returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // Generate hash for key
        let hash = self.state.hash_one(std::borrow::Borrow::borrow(&key));

        // If BinaryTree is empty, create root node
        let mut parent = &mut self.root;
        loop {
            match parent {
                TreePointer::Empty => {
                    *parent = TreePointer::new(
                        TreePointer::Empty,
                        key,
                        hash,
                        value,
                        TreePointer::Empty,
                    );
                    return None;
                }
                TreePointer::NonEmpty(node) => match hash.cmp(&node.hash) {
                    std::cmp::Ordering::Less => {
                        parent = &mut node.left;
                    }
                    std::cmp::Ordering::Equal => {
                        let temp = std::mem::replace(&mut node.value, value);
                        return Some(temp);
                    }
                    std::cmp::Ordering::Greater => {
                        parent = &mut node.right;
                    }
                },
            }
        }
    }

    /// Get value by key. Returns an Optional value. If there is no value by
    /// this key - None is returned.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        match self.find_pointer(key) {
            TreePointer::Empty => None,
            TreePointer::NonEmpty(node) => Some(&node.value),
        }
    }

    /// Remove pair from `HashTree`, returns value, or None if not present.
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        match self.root.remove(self.state.hash_one(key)) {
            Some(inner) => match inner {
                TreePointer::Empty => None,
                TreePointer::NonEmpty(node) => Some(node.value),
            },
            None => None,
        }
    }

    /// Get iterator for `HashTree`
    pub fn iter<'a>(&'a mut self) -> TreeIter<'a, K, V> {
        self.root.iter()
    }

    fn find_pointer<Q: ?Sized>(&self, key: &Q) -> &TreePointer<K, V>
    where
        K: std::borrow::Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        let hash = self.state.hash_one(key);
        let mut parent = &self.root;

        loop {
            match parent {
                TreePointer::NonEmpty(node) => match hash.cmp(&node.hash) {
                    std::cmp::Ordering::Less => {
                        parent = &node.left;
                        continue;
                    }
                    std::cmp::Ordering::Equal => return parent,
                    std::cmp::Ordering::Greater => {
                        parent = &node.right;
                        continue;
                    }
                },
                TreePointer::Empty => return parent,
            }
        }
    }
}

impl<'a, K: 'a, V: 'a> IntoIterator for &'a HashTree<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = TreeIter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.root.iter()
    }
}

impl<K, V> std::fmt::Debug for HashTree<K, V>
where
    K: std::fmt::Display,
    V: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = Err(std::fmt::Error);
        for item in self.root.debug_iter() {
            let left = match &item.left {
                TreePointer::Empty => format!("Empty"),
                TreePointer::NonEmpty(node) => format!("key is {}", node.key),
            };
            let right = match &item.right {
                TreePointer::Empty => format!("Empty"),
                TreePointer::NonEmpty(node) => format!("key is {}", node.key),
            };
            result = f.write_str(&format!(
                "\n\nHash: {}\nKey: {}\nValue: {}\nLeft: {}\nRight: {}",
                item.hash, item.key, item.value, left, right
            ));
        }
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

impl<K, V, Q> std::ops::Index<&Q> for HashTree<K, V>
where
    K: std::hash::Hash + Eq + std::borrow::Borrow<Q>,
    Q: Eq + std::hash::Hash + ?Sized,
{
    type Output = V;

    fn index(&self, index: &Q) -> &Self::Output {
        match self.find_pointer(index) {
            TreePointer::Empty => panic!("No entry found for key"),
            TreePointer::NonEmpty(node) => &node.value,
        }
    }
}

// ───── Tests ────────────────────────────────────────────────────────────── //

#[cfg(test)]
mod tests {
    use ahash::RandomState;

    use super::TreePointer::*;
    use super::*;

    #[test]
    fn test_nodes_creation() {
        // Create a small tree
        let pointer = generate_unhashed_tree();

        // Iterate it
        let v: Vec<_> = pointer.iter().map(|(&s1, &s2)| (s1, s2)).collect();

        assert_eq!(
            v,
            [
                ("mecha", "mechaV"),
                ("Jaeger", "JaegerV"),
                ("droid", "droidV"),
                ("GingerBread", "GingerBreadV"),
                ("Android", "AndroidV"),
                ("robot", "robotV")
            ]
        );
    }

    #[test]
    fn test_removing_node() {
        let mut tree: HashTree<u32, f32> = HashTree::new_with_seed(1);
        tree.insert(1, 10.); // 2
        tree.insert(2, 20.); // 1
        tree.insert(3, 30.); // 3
        tree.insert(4, 40.); // 5
        tree.insert(5, 50.); // 4

        let element = tree.remove(&3);
        let not_element = tree.remove(&155);

        assert_eq!(element, Some(30.));
        assert_eq!(not_element, None);

        let element = tree.remove(&4);
        assert_eq!(element, Some(40.));
        assert_eq!(not_element, None);

        let element = tree.remove(&5);
        assert_eq!(element, Some(50.));
        assert_eq!(not_element, None);
    }

    #[test]
    fn test_hash_tree_creation() {
        let mut tree: HashTree<u32, f32> = HashTree::new_with_seed(1);
        tree.insert(1, 10.); // 2
        tree.insert(2, 20.); // 1
        tree.insert(3, 30.); // 3
        tree.insert(4, 40.); // 5
        tree.insert(5, 50.); // 4

        match &tree.root {
            Empty => {}
            NonEmpty(node) => {
                assert_eq!(node.key, 1);
                match &node.left {
                    Empty => {}
                    NonEmpty(node) => {
                        assert_eq!(node.key, 2);
                    }
                }
                match &node.right {
                    Empty => {}
                    NonEmpty(node) => {
                        assert_eq!(node.key, 3);
                        match &node.right {
                            Empty => {}
                            NonEmpty(node) => {
                                assert_eq!(node.key, 4);
                                match &node.left {
                                    Empty => {}
                                    NonEmpty(node) => {
                                        assert_eq!(node.key, 5);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_find_pointer() {
        let tree = create_tree();

        match tree.find_pointer("mecha") {
            Empty => panic!("mecha not found"),
            NonEmpty(node) => {
                assert_eq!(node.key, "mecha");
                assert_eq!(node.value, "mechaV");
            }
        }

        match tree.find_pointer("Nothing") {
            Empty => {}
            NonEmpty(_) => {
                panic!("Failed")
            }
        }

        match tree.find_pointer("GingerBread") {
            Empty => panic!("GingerBread not found"),
            NonEmpty(node) => {
                assert_eq!(node.key, "GingerBread");
                assert_eq!(node.value, "GingerBreadV");
            }
        }
    }

    #[test]
    fn test_get() {
        let tree = create_tree();

        assert_eq!(tree.get("mecha"), Some(&"mechaV"));
    }

    #[test]
    fn test_index_operator() {
        let tree = create_tree();

        assert_eq!(tree["mecha"], "mechaV");
    }

    #[test]
    fn test_removing_node_order() {
        let mut tree = HashTree::new_with_seed(1);

        // These are random words
        // Scheme of binary tree in diagrams folder
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

        assert_eq!(
            tree.iter().map(|(&k, _)| k).collect::<Vec<_>>(),
            vec![
                "red",
                "moon",
                "bevy",
                "robot",
                "meow",
                "green",
                "sword",
                "purple",
                "knife",
                "droid",
                "blue",
                "t",
                "windows",
                "rust",
                "linux",
                "mecha",
                "blackberry",
                "sun",
                "GingerBread",
                "Android",
                "ios",
                "earth",
                "bow",
                "Jaeger",
                "black"
            ]
        );

        let removed = tree.remove("droid").unwrap();
        assert_eq!(removed, "droidV");

        let removed = tree.remove("GingerBread").unwrap();
        assert_eq!(removed, "GingerBreadV");

        assert_eq!(
            tree.iter().map(|(&k, _)| k).collect::<Vec<_>>(),
            vec![
                "red",
                "moon",
                "bevy",
                "robot",
                "meow",
                "green",
                "sword",
                "purple",
                "knife",
                "blue",
                "t",
                "windows",
                "rust",
                "linux",
                "mecha",
                "blackberry",
                "sun",
                "Android",
                "ios",
                "earth",
                "bow",
                "Jaeger",
                "black"
            ]
        );
    }

    fn create_tree<'a>() -> HashTree<&'a str, &'a str> {
        let mut tree = HashTree::new();

        tree.insert("mecha", "mechaV");
        tree.insert("Jaeger", "JaegerV");
        tree.insert("droid", "droidV");
        tree.insert("GingerBread", "GingerBreadV");
        tree.insert("Android", "AndroidV");
        tree.insert("robot", "robotV");
        tree
    }

    fn generate_unhashed_tree<'a>() -> TreePointer<&'a str, &'a str> {
        let state = RandomState::new();
        let subtree_l = TreePointer::new(
            Empty,
            "mecha",
            state.hash_one("mecha"),
            "mechaV",
            Empty,
        );
        let subtree_rlrl = TreePointer::new(
            Empty,
            "GingerBread",
            state.hash_one("GingerBread"),
            "GingerBreadV",
            Empty,
        );
        let subtree_rlr = TreePointer::new(
            subtree_rlrl,
            "Android",
            state.hash_one("Android"),
            "AndroidV",
            Empty,
        );
        let subtree_rl = TreePointer::new(
            Empty,
            "droid",
            state.hash_one("droid"),
            "droidV",
            subtree_rlr,
        );
        let subtree_r = TreePointer::new(
            subtree_rl,
            "robot",
            state.hash_one("robot"),
            "robotV",
            Empty,
        );
        let tree = TreePointer::new(
            subtree_l,
            "Jaeger",
            state.hash_one("Jaeger"),
            "JaegerV",
            subtree_r,
        );
        tree
    }
}
