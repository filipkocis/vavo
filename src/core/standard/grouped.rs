use crate::{assets::Handle, math::GlobalTransform, prelude::{Material, Mesh}, query::{Query, RunQuery}, render_assets::TransformStorage, system::SystemsContext};

/// One instance group represents a group of instances with the same material and mesh.
/// Instance count defines how many instances are in the group and instance offset is the offset in
/// the `TransformStorage` where these instances are stored.
pub struct InstanceGroup {
    pub material: Handle<Material>,
    pub mesh: Handle<Mesh>,
    pub instance_count: u32,
    pub instance_offset: u32,
}

impl InstanceGroup {
    pub fn new(material: Handle<Material>, mesh: Handle<Mesh>, instance_count: u32, instance_offset: u32) -> Self {
        Self {
            material,
            mesh,
            instance_count,
            instance_offset,
        }
    }
}

/// Grouped instances first by material and then by mesh.
#[derive(crate::macros::Resource)]
pub struct GroupedInstances {
    pub groups: Vec<InstanceGroup>,
}

impl GroupedInstances {
    /// Returns new grouped instances, requires transform storage to be a valid resource.
    /// Should be called before rendering and set as a resource.
    pub fn generate(
        ctx: &mut SystemsContext,
        mut query: Query<(&Handle<Material>, &Handle<Mesh>, &GlobalTransform)>,
    ) -> Self {
        // Prepare sorted storage
        let mut transforms = Vec::new();
        let mut sorted = Vec::<(&Handle<Material>, &Handle<Mesh>, &GlobalTransform)>::new();
        for (mat, mesh, global_transform) in query.iter_mut() {
            sorted.push((mat, mesh, global_transform));
        }

        // Sort by material and mesh
        sorted.sort_by(|a, b| {
            let material_cmp = a.0.id().cmp(&b.0.id());
            if material_cmp != std::cmp::Ordering::Equal {
                return material_cmp;
            }
            a.1.id().cmp(&b.1.id()) // mesh comparison
        });

        // Group by material and mesh
        let last_index = sorted.len().saturating_sub(1);
        let mut last_entry = None;
        let mut instance_count = 0;
        let mut instance_offset = 0;
        let mut groups = Vec::<InstanceGroup>::new();
        for (i, (material, mesh, global_transform)) in sorted.into_iter().enumerate() {
            if let Some((last_material, last_mesh, last_instance_count)) = last_entry {
                if last_material == *material && last_mesh == *mesh {
                    instance_count += 1;
                } else {
                    groups.push(InstanceGroup::new(
                        last_material, last_mesh, last_instance_count, instance_offset,
                    ));

                    instance_offset += last_instance_count;
                    instance_count = 1;
                }
            } else {
                instance_count = 1;
            }

            if i == last_index {
                groups.push(InstanceGroup::new(material.clone(), mesh.clone(), instance_count, instance_offset));
            }

            last_entry = Some((material.clone(), mesh.clone(), instance_count));
            transforms.push(global_transform.as_matrix().to_cols_array_2d());
        }

        // Set transforms storage
        let mut transforms_storage = ctx.resources.get_mut::<TransformStorage>().unwrap();
        transforms_storage.update(&transforms, transforms.len(), ctx);

        Self { groups }
    }
}
