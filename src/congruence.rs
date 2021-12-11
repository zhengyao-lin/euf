use std::fmt;

pub type NodeIndex = usize;
pub type SymbolIndex = usize;

struct NodeData {
    symbol: SymbolIndex,
    cong_class: Option<NodeIndex>,
    cong_height: usize, // upper bound on the height of the tree formed by cong_class
    parents: Vec<NodeIndex>,
    children: Vec<NodeIndex>,
}

pub struct CongruenceGraph {
    nodes: Vec<NodeData>,
}

impl CongruenceGraph {
    pub fn new() -> CongruenceGraph {
        CongruenceGraph { nodes: vec![] }
    }

    // /// Get the symbol of a node
    // pub fn get_symbol(&self, node: NodeIndex) -> SymbolIndex {
    //     self.nodes[node].symbol
    // }

    // /// Check if a node has a parent with the given symbol
    // pub fn has_parent_with_symbol(&self, node: NodeIndex, symbol: SymbolIndex) -> bool {
    //     for parent in &self.nodes[node].parents {
    //         if self.get_symbol(*parent) == symbol {
    //             return true
    //         }
    //     }
    //     return false
    // }

    /// Add a parent to the given children
    pub fn add_node(&mut self, symbol: SymbolIndex, children: &Vec<NodeIndex>) -> NodeIndex {
        // if there exists a node with the same symbol and children, return that node
        for (i, node) in self.nodes.iter().enumerate() {
            if node.symbol == symbol && &node.children == children {
                return i;
            }
        }

        // check that all children exists
        let new_index = self.nodes.len();
        for child in children {
            debug_assert!(child < &self.nodes.len(), "node {} does not exist", child);
            self.nodes[*child].parents.push(new_index);
        }
        self.nodes.push(NodeData {
            symbol,
            cong_class: None,
            cong_height: 0,
            parents: vec![],
            children: children.clone(), 
        });
        return new_index;
    }

    /// Find the representative of the congruence class that node belongs to
    pub fn get_congruent_class(&self, node: NodeIndex) -> NodeIndex {
        let mut rep = node;
        while let Some(next_rep) = self.nodes[rep].cong_class {
            rep = next_rep;
        }
        rep
    }

    /// Check if node1 and node2 have exactly the same arguments
    pub fn have_congruent_children(&self, node1: NodeIndex, node2: NodeIndex) -> bool {
        let node1_children = &self.nodes[node1].children;
        let node2_children = &self.nodes[node2].children;

        if node1_children.len() != node2_children.len() {
            return false
        }

        for (node1_child, node2_child) in node1_children.iter().zip(node2_children.iter()) {
            if self.get_congruent_class(*node1_child) != self.get_congruent_class(*node2_child) {
                return false
            }
        }

        true
    }
    
    /// Merge the congruence classes of two nodes
    pub fn merge_congruence_classes(&mut self, node1: NodeIndex, node2: NodeIndex) {
        let mut to_be_merged = vec![(node1, node2)];

        while let Some((node1, node2)) = to_be_merged.pop() {
            let node1_class = self.get_congruent_class(node1);
            let node2_class = self.get_congruent_class(node2);

            // nothing to do
            if node1_class == node2_class {
                continue
            }

            // make the cong tree more balanced
            if self.nodes[node1_class].cong_height < self.nodes[node2_class].cong_height {
                self.nodes[node1].cong_class = Some(node2_class);
            } else {
                self.nodes[node2].cong_class = Some(node1_class);
            }

            // TODO: this is a bit slow
            // need a better way to find
            // congruent ancestors
            for i in 0..self.nodes.len() {
                for j in 0..i {
                    if self.nodes[i].symbol == self.nodes[j].symbol &&
                       self.have_congruent_children(i, j) {
                        to_be_merged.push((i, j));
                    }
                }
            }
        }
    }
}

impl fmt::Display for CongruenceGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, node) in self.nodes.iter().enumerate() {
            write!(f, "{}: symbol={}, cong={:?}/{}", i, node.symbol, node.cong_class, self.get_congruent_class(i))?;

            if !node.parents.is_empty() {
                write!(f, ", parent:")?;
                for parent in &node.parents {
                    write!(f, " {}", parent)?;
                }
            }

            if !node.children.is_empty() {
                write!(f, ", children:")?;
                for child in &node.children {
                    write!(f, " {}", child)?;
                }
            }

            if i + 1 < self.nodes.len() {
                writeln!(f, "")?;
            }
        }
        Ok(())
    }
}
