use crate::{math::{GlobalTransform, Transform}, prelude::{Changed, With, Without}, query::{Query, RunQuery}, system::SystemsContext, world::{Children, EntityId, Parent}};

/// Internal system that updates global transforms of entities with changed local transforms.
pub fn update_global_transforms(_: &mut SystemsContext, mut q: Query<()>) {
    // update root entities
    let mut query = q.cast::<(&mut GlobalTransform, &Transform), (Changed<Transform>, Without<Parent>)>();
    for (global, local) in query.iter_mut() {
        global.update(local);
    }

    // recursively update children of updated entities
    let mut query = q.cast::<(&EntityId, &mut GlobalTransform), (With<Children>, Changed<Transform>)>();
    for (id, global) in query.iter_mut() {
        update_children(*id, global, q.cast());
    }
}

fn update_children(parent_id: EntityId, parent_global: &GlobalTransform, mut parent_query: Query<&Children>) {
    // get children of parent
    let children = match parent_query.get(parent_id) {
        Some(children) => children,
        None => return,
    };

    // update every child recursively
    let mut child_query = parent_query.cast::<(&mut GlobalTransform, &Transform), With<Parent>>();
    for child in &children.ids {
        if let Some((global, local)) = child_query.get(*child) {
            // update child of parent
            *global = parent_global.combine_child(local);

            // recursively update children of child
            update_children(*child, global, child_query.cast());
        } 
    }
}
