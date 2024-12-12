use std::collections::{HashMap, HashSet};

use super::GraphNode;

pub struct RenderGraph { 
    pub nodes: HashMap<String, GraphNode>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add(&mut self, node: GraphNode) {
        self.nodes.insert(node.name.clone(), node);
    }

    pub fn get(&self, name: &str) -> Option<&GraphNode> {
        self.nodes.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut GraphNode> {
        self.nodes.get_mut(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<GraphNode> {
        self.nodes.remove(name)
    }

    /// Returns a topological sort of mutable graph nodes
    pub fn topological_sort_mut(&mut self) -> Vec<&mut GraphNode> {
        let mut visited = HashSet::new();
        let mut sorted = Vec::new();

        let graph = self as *mut RenderGraph;
        for node in self.nodes.values_mut() {
            unsafe { &mut *graph }.visit(node, &mut visited, &mut sorted);
        }
        
        sorted.into_iter().map(|node| unsafe { &mut *node }).collect()
    }

    fn visit(&mut self, node: *mut GraphNode, visited: &mut HashSet<String>, sorted: &mut Vec<*mut GraphNode>) {
        let node = unsafe { &mut *node };
        if visited.contains(&node.name) {
            return;
        }

        visited.insert(node.name.clone());

        for dep in &node.dependencies {
            if let Some(dep_node) = self.get_mut(dep) {
                let dep_node = dep_node as *mut _;
                self.visit(dep_node, visited, sorted);
            }
        }

        sorted.push(node);
    }
}
