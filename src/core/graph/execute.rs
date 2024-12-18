use wgpu::RenderPass;

use crate::{core::graph::NodeColorTarget, system::SystemsContext, world::entities::Entities};

use super::{data::{ColorTargetData, DepthTargetData}, GraphNode, NodeDepthTarget, RenderGraph};

pub struct RenderGraphContext<'a> {
    /// Current render pass, should be used to issue draw calls etc.
    pub pass: &'a mut RenderPass<'a>,
    /// Unsafe raw pointer to the node currently being executed, should not be used or modified unless you are sure what
    /// you are doing.
    pub node: *mut GraphNode,
    /// Unsafe raw pointer to render graph from which this context was derived, should not be used or modified unless you are sure what
    /// you are doing.
    pub graph: *mut RenderGraph,
}

impl<'a> RenderGraphContext<'a> {
    pub fn new(pass: &'a mut RenderPass<'a>, node: *mut GraphNode, graph: *mut RenderGraph) -> Self {
        Self {
            pass,
            node,
            graph,
        }
    }
}

impl RenderGraph {
    pub(crate) fn execute(&mut self, ctx: &mut SystemsContext, entities: &mut Entities) {
        // SAFETY: This is safe, we are bypassing borrow checker in order to mutably generate node data,
        // which is in the same loop as the immutable color / depth attachments
        let sorted = unsafe { &mut *(self as *mut RenderGraph) }.topological_sort_mut();

        for node in sorted {
            if node.data.needs_regen {
                node.generate_data(ctx);
            }

            // SAFETY: This is safe because it's only 'copied' to get the color and depth attachments
            let ctx_copy = unsafe { &mut *(ctx as *mut SystemsContext) };
            // SAFETY: Since encoder is derived from ctx, we just bypass the borrow checker
            let encoder = ctx.renderer.encoder().inner;

            let color_attachment = self.get_color_attachment(node, ctx);
            let depth_attachment = self.get_depth_attachment(node, ctx_copy);

            let mut render_pass = unsafe { &mut *encoder }.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("{} render pass", node.name)),
                color_attachments: &[color_attachment],
                depth_stencil_attachment: depth_attachment,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(node.data.pipeline.as_ref().expect("Pipeline should have been generated by now").render_pipeline());

            let graph_ctx = RenderGraphContext {
                pass: &mut render_pass,
                node,
                graph: self,
            };

            node.system.run(graph_ctx, ctx, entities);
        }
    }

    fn get_color_attachment<'a>(&self, node: &'a GraphNode, ctx: &'a mut SystemsContext) -> Option<wgpu::RenderPassColorAttachment<'a>> {
        let view = match node.data.color_target {
            Some(ref target_data) => {
                match target_data {
                    ColorTargetData::Texture(texture) => &texture.view,
                    ColorTargetData::RAE(rae) => &rae.view,
                }
            },
            None => {
                match &node.color_target {
                    NodeColorTarget::None => return None,
                    NodeColorTarget::Surface => ctx.renderer.view(),
                    NodeColorTarget::Node(ref name) => {
                        let graph = self as *const RenderGraph;
                        let graph = unsafe { &*graph };

                        let node = graph.get(name).expect(&format!("Node '{}' not found, but it is a color target for '{}'", name, node.name));
                        let color_attachment = self.get_color_attachment(node, ctx)
                            .expect(&format!("Node '{}' has no color attachment, but it is a color target for '{}'", name, node.name));

                        return Some(color_attachment)
                        
                    },
                    target => panic!("'{}' reached unexpected branch, it should have been generated and handled in NodeData", target)
                }
            }
        };

        Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            }
        })
    }

    fn get_depth_attachment<'a>(&self, node: &'a GraphNode, ctx: &'a mut SystemsContext) -> Option<wgpu::RenderPassDepthStencilAttachment<'a>> {
        let view = match node.data.depth_target {
            Some(ref target_data) => {
                match target_data {
                    DepthTargetData::Texture(texture) => &texture.view,
                    DepthTargetData::RAE(rae) => &rae.view,
                }
            },
            None => {
                match &node.depth_target {
                    NodeDepthTarget::None => return None,
                    NodeDepthTarget::Node(ref name) => {
                        let graph = self as *const RenderGraph;
                        let graph = unsafe { &*graph };

                        let node = graph.get(name).expect(&format!("Node '{}' not found, but it is a color target for '{}'", name, node.name));
                        let depth_attachment = self.get_depth_attachment(node, ctx)
                            .expect(&format!("Node '{}' has no color attachment, but it is a color target for '{}'", name, node.name));

                        return Some(depth_attachment)
                        
                    },
                    target => panic!("'{}' reached unexpected branch, it should have been generated and handled in NodeData", target)
                }
            }
        };

        Some(wgpu::RenderPassDepthStencilAttachment {
            view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        })
    }
}
