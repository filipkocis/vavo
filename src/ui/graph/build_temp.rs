use std::fmt::Debug;

use crate::prelude::*;
use crate::ui::node::{ComputedNode, Node};

pub struct TempNode<'a> {
    pub id: EntityId,
    pub node: &'a Node,
    pub computed: &'a mut ComputedNode,
    pub transform: &'a mut Transform,
    pub children: Vec<TempNode<'a>>,
}

impl Debug for TempNode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TempNode")
            .field("id", &self.id)
            .field("children", &self.children)
            .finish() 
    }
}

/// Returns temp nodes with populated children, or empty if zero nodes were updated.
/// Runs on `Changed<Node>` filter.
pub fn nodes_to_temp_graph<'a>(q: &mut Query<'a, ()>) -> Vec<TempNode<'a>> {
    let mut check_updated = q.cast::<
        &EntityId,
        (With<Node>, With<ComputedNode>, With<Transform>, Changed<Node>)
    >();
    // if no nodes where updated, do not run and return empty
    if check_updated.iter_mut().is_empty() {
        return Vec::new();
    }

    let mut root_query = q.cast::<
        (&EntityId, &Node, &mut ComputedNode, &mut Transform), 
        Without<Parent>
    >();
    
    // populate with root nodes
    let mut root_nodes = Vec::new();
    for (id, node, computed, transform) in root_query.iter_mut() {
        let root = TempNode {
            id: *id,
            node,
            computed,
            transform,
            children: Vec::new(),
        };
        root_nodes.push(root)
    }

    let mut nonui_root_query = q.cast::<
        (&EntityId, &Node, &mut ComputedNode, &mut Transform), 
        With<Parent>
    >();

    // populate with root nodes that have nonui parents
    for (id, node, computed, transform) in nonui_root_query.iter_mut() {
        let parent = q.cast::<&Parent, ()>().get(*id).expect("Parent not found");
        if q.cast::<&Node, ()>().get(parent.id).is_some() {
            continue;
        }

        let root = TempNode {
            id: *id,
            node,
            computed,
            transform,
            children: Vec::new(),
        };
        root_nodes.push(root)
    }

    // populate with children
    for root in root_nodes.iter_mut() {
        let children = match q.cast::<&Children, ()>().get(root.id) {
            Some(children) => children,
            None => continue,
        };

        for child in &children.ids {
            root.children.push(build_temp_node_for(*child, q))
        }
    }

    root_nodes
}

/// Returns a TempNode<'a> for a given EntityId, fully populated with children recursively
fn build_temp_node_for<'a>(id: EntityId, query: &mut Query<'a, ()>) -> TempNode<'a> {
    let mut node_query = query.cast::<(&Node, &mut ComputedNode, &mut Transform), ()>();
    let (node, computed, transform) = node_query.get(id).expect("Node not found");

    let mut children_query = query.cast::<&Children, ()>();
    let mut built_children = Vec::new();
    if let Some(children) = children_query.get(id) {
        for child in &children.ids {
            built_children.push(build_temp_node_for(*child, query))
        }
    }

    TempNode {
        id,
        node,
        computed,
        transform,
        children: built_children,
    }
}
