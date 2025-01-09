use crate::prelude::*;

use super::{grouped::GroupedInstances, light_data::PreparedLightData};

pub fn graph_prerender_preparation_system(
    ctx: &mut SystemsContext,
    mut query: Query<()>
) {
    let prepared_light_data = PreparedLightData::prepare(ctx, query.cast());
    let grouped_instances = GroupedInstances::generate(ctx, query.cast());

    ctx.resources.insert(prepared_light_data);
    ctx.resources.insert(grouped_instances);
}
