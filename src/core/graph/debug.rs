use crate::{
    app::Plugin,
    prelude::Time,
    query::Query,
    system::{SystemStage, SystemsContext},
};

/// QOL plugin to print the render graph topology
pub struct DebugRenderGraphPlugin;

impl Plugin for DebugRenderGraphPlugin {
    fn build(&self, app: &mut crate::prelude::App) {
        app.register_system(
            |ctx: &mut SystemsContext, _: Query<()>| {
                let time = ctx.resources.get::<Time>().unwrap();
                if time.tick() == 1 {
                    print_render_graph_topology(ctx);
                }
            },
            SystemStage::Render,
        );
    }
}

pub fn print_render_graph_topology(ctx: &mut SystemsContext) {
    println!("Render Graph Topology:");

    let graph = unsafe { &mut *ctx.graph };
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
        if let Some(deps) = normalized.get(&node.name) {
            if !deps.is_empty() {
                println!("  AfterN -> {:?}", deps);
            }
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
