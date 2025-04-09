use std::collections::{HashMap, HashSet};

use winit::dpi::PhysicalSize;

use super::GraphNode;

pub struct RenderGraph {
    pub(crate) nodes: HashMap<String, GraphNode>,
    /// Topological sort of `self.nodes`, updated on each node add/remove
    pub(crate) sorted: Vec<*mut GraphNode>,
}

impl RenderGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            sorted: Vec::new(),
        }
    }

    pub fn add(&mut self, node: GraphNode) {
        self.nodes.insert(node.name.clone(), node);
        self.update_topological_sort();
    }

    pub fn get(&self, name: &str) -> Option<&GraphNode> {
        self.nodes.get(name)
    }

    /// Returns a mutable reference to a node with name `name`.
    ///
    /// # Info
    /// If you change the dependencies, make sure to call [`Self::update_topological_sort`] for it
    /// to take an effect
    pub fn get_mut(&mut self, name: &str) -> Option<&mut GraphNode> {
        self.nodes.get_mut(name)
    }

    pub fn remove(&mut self, name: &str) -> Option<GraphNode> {
        let node = self.nodes.remove(name);
        if node.is_some() {
            self.update_topological_sort();
        }
        node
    }

    /// Sorts the graph nodes topologically. Only call on node change
    pub fn update_topological_sort(&mut self) {
        let mut visited = HashSet::new();
        let mut sorted = Vec::new();

        let graph = self as *mut RenderGraph;
        let normalized = self.normalize_dependencies();

        for node in self.nodes.values_mut() {
            let mut path = Vec::new();
            unsafe { &mut *graph }.visit(&normalized, &mut path, node, &mut visited, &mut sorted);
        }
        
        self.sorted = sorted;
    }

    /// Populates the `before` dependencies with the respective `after` dependencies from nodes
    pub(crate) fn normalize_dependencies(&mut self) -> HashMap<String, Vec<String>> {
        let mut nodes = self.nodes.iter()
            .map(|(k, v)| (k.clone(), v.after.clone()))
            .collect::<HashMap<_, _>>();

        for node in self.nodes.values() {
            for before in &node.before {
                if let Some(dep_node) = nodes.get_mut(before) {
                    dep_node.push(node.name.clone());
                }
            }
        }

        nodes
    }

    /// Traverse the graph
    fn visit(&mut self, normalized: &HashMap<String, Vec<String>>, path: &mut Vec<String>, node: *mut GraphNode, visited: &mut HashSet<String>, sorted: &mut Vec<*mut GraphNode>) {
        let node = unsafe { &mut *node };
        let name = &node.name;

        if path.contains(name) {
            path.push(name.clone());
            panic!("Cyclic render graph dependencies: {:?}", path);
        }

        if visited.contains(name) {
            return;
        }

        path.push(name.clone());
        visited.insert(name.clone());

        let dependencies = normalized.get(name).expect("Normalized nodes should contain graph node");
        for dep in dependencies {
            if let Some(dep_node) = self.get_mut(dep) {
                let dep_node = dep_node as *mut _;
                self.visit(normalized, path, dep_node, visited, sorted);
            }
        }

        path.pop();
        sorted.push(node);
    }

    pub(crate) fn resize(&mut self, size: PhysicalSize<u32>) {
        for node in self.nodes.values_mut() {
            node.resize(&size);
        }
    }
}
