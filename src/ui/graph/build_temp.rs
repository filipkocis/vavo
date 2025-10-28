use std::fmt::Debug;

use crate::event::event_handler::EventReader;
use crate::prelude::*;
use crate::render_assets::RenderAssetEntry;
use crate::ui::image::UiImage;
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
    /// Uninitialized when building the temp graph, will be populated in `resolve_z_index` when
    /// recreating the [`text buffer`](crate::ui::text::TextBuffer) with the correct z-index
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
/// Runs on `Changed<Node | Text | UiImage | Transform>` filters, or `WindowEvent::Resized` event
pub fn nodes_to_temp_graph<'a>(
    event_reader: EventReader,
    q: &mut Query<()>
) -> Vec<TempNode<'a>> {
    let mut check_updated = q.cast::<
        EntityId,
        (
            With<Node>, With<ComputedNode>, 
            Or<(Changed<Node>, Changed<Text>, Changed<UiImage>, Changed<Transform>)>
        )
    >();

    // if zero nodes where updated and window has not been resized,
    // do not run and return empty
    if check_updated.iter_mut().is_empty() && !has_resized(&event_reader) {
        return Vec::new();
    }

    // TODO: add other node types as options, like Image, Button, etc.
    let mut root_query = q.cast::<
        (EntityId, &Node, &mut ComputedNode, &mut Transform, Option<&Children>, Option<&Parent>, Option<&mut Text>), 
        ()
    >();
    
    // populate with root nodes
    let mut root_nodes = Vec::new();
    for (id, node, computed, transform, children, parent, text) in root_query.iter_mut() {
        if let Some(parent) = parent {
            // populate only with root nodes that have nonui parents
            if q.cast::<&Node, ()>().get(parent.id).is_some() {
                continue;
            }
        }

        let mut root = TempNode {
            id,
            node,
            computed,
            transform,
            children: Vec::new(),

            text,
            text_rae: None,
        };

        // populate with children
        if let Some(children) = children {
            for child in &children.ids {
                root.children.push(build_temp_node_for(*child, q))
            }
        };

        root_nodes.push(root);
    }
    
    root_nodes
}

/// Returns a TempNode<'a> for a given EntityId, fully populated with children recursively
fn build_temp_node_for<'a>(id: EntityId, query: &mut Query<()>) -> TempNode<'a> {
    // root
    let mut node_query = query.cast::<(&Node, &mut ComputedNode, &mut Transform, Option<&Children>, Option<&mut Text>), ()>();
    let (node, computed, transform, children, text) = node_query.get(id).expect("Node not found");
    // reset old computed
    *computed = ComputedNode::default();

    // children
    let mut built_children = Vec::new();
    if let Some(children) = children {
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

        text,
        text_rae: None,
    }
}
