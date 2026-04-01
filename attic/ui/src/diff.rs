///
/// Represents a difference between two trees
///
#[derive(Clone, PartialEq, Serialize)]
pub struct Diff<TNode: Clone> {
    /// The address of this difference
    ///
    /// This is the empty list to indicate the root node, otherwise it's a list
    /// of node indices forming a path through the original tree.
    address: Vec<u32>,

    /// The new node that should replace the original node at this address
    replacement: TNode
}

impl<TNode: Clone> Diff<TNode> {
    ///
    /// Creates a new diff item
    ///
    pub fn new(address: &Vec<u32>, replacement: &TNode) -> Diff<TNode> {
        Diff { address: address.clone(), replacement: replacement.clone() }
    }

    pub fn address(&self) -> &Vec<u32> {
        &self.address
    }

    pub fn replacement(&self) -> &TNode {
        &self.replacement
    }
}

///
/// Trait implemented by tree structures that can describe how they
/// differ from another structure.
///
pub trait DiffableTree where Self: Clone {
    ///
    /// Retrieves the child nodes of this item
    ///
    fn child_nodes<'a>(&'a self) -> Vec<&'a Self>;

    ///
    /// Returns true if this node is different from the specified node
    /// (excluding child nodes)
    ///
    fn is_different(&self, compare_to: &Self) -> bool;
}

///
/// Computes the difference between two trees
///
pub fn diff_tree<TNode: DiffableTree>(source: &TNode, target: &TNode) -> Vec<Diff<TNode>> {
    diff_tree_run(&vec![], source, target)
}

///
/// Computes the difference between two trees (where we know the address)
///
fn diff_tree_run<TNode: DiffableTree>(address: &Vec<u32>, source: &TNode, target: &TNode) -> Vec<Diff<TNode>> {
    let is_different = source.is_different(target);

    if is_different {
        // Different nodes replace the source with the target
        vec![Diff::new(address, target)]
    } else {
        // If the nodes are not different, then check the child nodes
        let source_children = source.child_nodes();
        let target_children = target.child_nodes();

        if source_children.len() != target_children.len() {
            // If the child node counts are different, the nodes are different
            vec![Diff::new(address, target)]
        } else if source_children.len() == 0 {
            // No point generating all the iteration structures if there's nothing to iterate
            vec![]
        } else {
            // Check for differences in the child nodes
            let mut differences     = vec![];
            let mut node_address    = address.clone();

            for node_index in 0..source_children.len() {
                // Generate the address for the next node
                node_address.push(node_index as u32);

                // Push any differences it might have
                let child_differences = diff_tree_run(&node_address, source_children[node_index], target_children[node_index]);
                differences.extend(child_differences);

                // Reset the node address vector so we can re-use it in the next pass
                node_address.pop();
            }

            differences
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, PartialEq, Serialize, Deserialize)]
    struct TestTree {
        id: u32,
        child_nodes: Vec<TestTree>
    }

    impl DiffableTree for TestTree {
        fn child_nodes<'a>(&'a self) -> Vec<&'a TestTree> {
            self.child_nodes.iter().collect()
        }

        fn is_different(&self, compare_to: &Self) -> bool {
            self.id != compare_to.id
        }
    }

    fn simple_tree() -> TestTree {
        TestTree {
            id: 0,
            child_nodes: vec![
                TestTree {
                    id: 1,
                    child_nodes: vec![]
                },

                TestTree {
                    id: 2,
                    child_nodes: vec![
                        TestTree {
                            id: 3,
                            child_nodes: vec![]
                        },
                        TestTree {
                            id: 4,
                            child_nodes: vec![]
                        }
                    ]
                }
            ]
        }
    }

    fn tree_with_single_id_diff() -> TestTree {
        TestTree {
            id: 0,
            child_nodes: vec![
                TestTree {
                    id: 1,
                    child_nodes: vec![]
                },

                TestTree {
                    id: 3,
                    child_nodes: vec![
                        TestTree {
                            id: 3,
                            child_nodes: vec![]
                        },
                        TestTree {
                            id: 4,
                            child_nodes: vec![]
                        }
                    ]
                }
            ]
        }
    }

    fn tree_with_multiple_differences() -> TestTree {
        TestTree {
            id: 0,
            child_nodes: vec![
                TestTree {
                    id: 1,
                    child_nodes: vec![]
                },

                TestTree {
                    id: 2,
                    child_nodes: vec![
                        TestTree {
                            id: 4,
                            child_nodes: vec![]
                        },
                        TestTree {
                            id: 5,
                            child_nodes: vec![]
                        }
                    ]
                }
            ]
        }
    }

    fn tree_with_extra_child_nodes() -> TestTree {
        TestTree {
            id: 0,
            child_nodes: vec![
                TestTree {
                    id: 1,
                    child_nodes: vec![]
                },

                TestTree {
                    id: 2,
                    child_nodes: vec![
                        TestTree {
                            id: 3,
                            child_nodes: vec![]
                        },
                        TestTree {
                            id: 4,
                            child_nodes: vec![]
                        },
                        TestTree {
                            id: 5,
                            child_nodes: vec![]
                        },
                        TestTree {
                            id: 6,
                            child_nodes: vec![]
                        }
                    ]
                }
            ]
        }
    }

    fn tree_with_multiple_differences_in_sublevels() -> TestTree {
        TestTree {
            id: 0,
            child_nodes: vec![
                TestTree {
                    id: 1,
                    child_nodes: vec![]
                },

                TestTree {
                    id: 3,
                    child_nodes: vec![
                        TestTree {
                            id: 4,
                            child_nodes: vec![]
                        },
                        TestTree {
                            id: 5,
                            child_nodes: vec![]
                        }
                    ]
                }
            ]
        }
    }

    #[test]
    fn compare_tree_against_self_is_empty() {
        let tree        = simple_tree();
        let differences = diff_tree(&tree, &tree);

        assert!(differences.len() == 0);
    }

    #[test]
    fn compare_tree_against_different_ids_has_single_difference() {
        let tree_a      = simple_tree();
        let tree_b      = tree_with_single_id_diff();
        let differences = diff_tree(&tree_a, &tree_b);

        assert!(differences.len() == 1);
        assert!(differences[0].address() == &vec![1]);
    }

    #[test]
    fn compare_tree_against_different_ids_has_multiple_difference() {
        let tree_a      = simple_tree();
        let tree_b      = tree_with_multiple_differences();
        let differences = diff_tree(&tree_a, &tree_b);

        assert!(differences.len() == 2);
        assert!(differences[0].address() == &vec![1, 0]);
        assert!(differences[1].address() == &vec![1, 1]);
    }

    #[test]
    fn tree_with_extra_child_nodes_is_different() {
        let tree_a      = simple_tree();
        let tree_b      = tree_with_extra_child_nodes();
        let differences = diff_tree(&tree_a, &tree_b);

        assert!(differences.len() == 1);
        assert!(differences[0].address() == &vec![1]);
    }

    #[test]
    fn tree_with_fewer_child_nodes_is_different() {
        let tree_a      = tree_with_extra_child_nodes();
        let tree_b      = simple_tree();
        let differences = diff_tree(&tree_a, &tree_b);

        assert!(differences.len() == 1);
        assert!(differences[0].address() == &vec![1]);
    }

    #[test]
    fn tree_diffs_in_sublevels_not_reported() {
        let tree_a      = simple_tree();
        let tree_b      = tree_with_multiple_differences_in_sublevels();
        let differences = diff_tree(&tree_a, &tree_b);

        assert!(differences.len() == 1);
        assert!(differences[0].address() == &vec![1]);
    }
}
