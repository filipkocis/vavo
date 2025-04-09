use crate::{app::Plugin, prelude::Time, query::Query, system::{SystemStage, SystemsContext}};

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
        println!("  ({i}) {} -> {:?}", node.0, node.1.dependencies);
    }

    println!("\nSorted Graph Nodes:");
    for (i, node) in graph.topological_sort_mut().into_iter().enumerate() {
        println!("  ({i}) {} -> {:?}", node.name, node.dependencies);
    }
}
