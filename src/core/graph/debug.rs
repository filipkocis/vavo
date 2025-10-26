use crate::{
    app::{App, Plugin},
    core::graph::RenderGraph,
    prelude::{Res, Time},
    system::SystemStage,
};

/// QOL plugin to print the render graph topology
pub struct DebugRenderGraphPlugin;

impl Plugin for DebugRenderGraphPlugin {
    fn build(&self, app: &mut crate::prelude::App) {
        app.register_system(
            |time: Res<Time>, app: &mut App| {
                if time.tick() == 1 {
                    // Safe because we don't mutate the graph
                    let graph = unsafe { app.render_graph() };
                    print_render_graph_topology(graph);
                }
            },
            SystemStage::Render,
        );
    }
}

pub fn print_render_graph_topology(graph: &RenderGraph) {
    println!("Render Graph Topology:");
    println!("Graph Nodes: {}", graph.nodes.len());

    println!("\nUnsorted Graph Nodes:");
    for (i, node) in graph.nodes.iter().enumerate() {
        println!("({i}) {}", node.0);
        if !node.1.after.is_empty() {
            println!("  After -> {:?}", node.1.after);
        }
        if !node.1.before.is_empty() {
            println!("  Before -> {:?}", node.1.before);
        }
    }

    let normalized = graph.normalize_dependencies();
    let mut names = Vec::new();

    println!("\nSorted Graph Nodes:");
    for (i, node) in graph.sorted.iter().enumerate() {
        let node = unsafe { &mut **node };
        names.push(node.name.clone());

        println!("({i}) {}", node.name);
        if let Some(deps) = normalized.get(&node.name)
            && !deps.is_empty()
        {
            println!("  AfterN -> {:?}", deps);
        }
        if !node.after.is_empty() {
            println!("  After -> {:?}", node.after);
        }
        if !node.before.is_empty() {
            println!("  Before -> {:?}", node.before);
        }
    }

    println!("\nGraph Nodes in Sequence:");
    println!("  {}", names.join(" -> "));
}
