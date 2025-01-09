use std::fmt::Debug;

use crate::prelude::*;
use crate::render_assets::{RenderAssetEntry, RenderAssets};
use crate::ui::node::{ComputedNode, Node};
use crate::ui::text::{Text, TextBuffer};

use super::update::has_resized;

pub struct TempNode<'a> {
    pub id: EntityId,
    pub node: &'a Node,
    pub computed: &'a mut ComputedNode,
    pub transform: &'a mut Transform,
    pub children: Vec<TempNode<'a>>,

    pub text: Option<&'a mut Text>,
    pub text_rae: Option<RenderAssetEntry<TextBuffer>>,
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
/// Runs on `Changed<Node>` filter, or `WindowEvent::Resized` event
pub fn nodes_to_temp_graph<'a>(ctx: &mut SystemsContext, q: &mut Query<()>) -> Vec<TempNode<'a>> {
    let mut check_updated = q.cast::<
        &EntityId,
        (With<Node>, With<ComputedNode>, With<Transform>, Changed<Node>)
    >();

    // if zero nodes where updated and window has not been resized,
    // do not run and return empty
    if check_updated.iter_mut().is_empty() && !has_resized(ctx) {
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

            text: None,
            text_rae: None,
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

            text: None,
            text_rae: None,
        };
        root_nodes.push(root)
    }

    // populate with children and ui node components
    for root in root_nodes.iter_mut() {
        // children
        if let Some(children) = q.cast::<&Children, ()>().get(root.id) {
            for child in &children.ids {
                root.children.push(build_temp_node_for(ctx, *child, q))
            }
        };

        // TODO: replace these once query filter supports Option<T>
        // text
        root.text = q.cast::<&mut Text, ()>().get(root.id);

        // TODO: add other node types as options, like Image, Button, etc.
    }

    root_nodes
}

/// Returns a TempNode<'a> for a given EntityId, fully populated with children recursively
fn build_temp_node_for<'a>(ctx: &mut SystemsContext, id: EntityId, query: &mut Query<()>) -> TempNode<'a> {
    // root
    let mut node_query = query.cast::<(&Node, &mut ComputedNode, &mut Transform), ()>();
    let (node, computed, transform) = node_query.get(id).expect("Node not found");
    // reset old computed
    *computed = ComputedNode::default();

    // children
    let mut children_query = query.cast::<&Children, ()>();
    let mut built_children = Vec::new();
    if let Some(children) = children_query.get(id) {
        for child in &children.ids {
            built_children.push(build_temp_node_for(ctx, *child, query))
        }
    }

    // text
    let mut text_buffers = ctx.resources.get_mut::<RenderAssets<TextBuffer>>().expect("TextBuffer render assets not found");
    let text = query.cast::<&mut Text, ()>().get(id);
    // HINT: currently this will be replaced in compute_z_index every time, we will keep it anyways
    let text_rae = text.as_ref().map(|text| {
        text_buffers.get_by_entity(&id, &**text, ctx)
    });

    // TODO: add other node types as options, like Image, Button, etc.

    TempNode {
        id,
        node,
        computed,
        transform,
        children: built_children,

        text,
        text_rae,
    }
}
