use crate::{
    ecs::entities::{components::ComponentsData, Component, EntityId},
    prelude::{Mut, Ref, Tick},
};

use super::{
    filter::{Filters, QueryFilter},
    Query, QueryComponentType,
};

pub trait RunQuery {
    type Output;

    fn iter_mut(&mut self) -> Vec<Self::Output>;
    fn get(&mut self, entity_id: EntityId) -> Option<Self::Output>;
}

/// Retrieve information about the requested component type in the query
trait QueryGetType {
    /// Get component type wrapped in [`QueryComponentType`]
    fn get_type_id() -> QueryComponentType;

    /// Returns `None` for option types, otherwise panics
    fn get_none() -> Self
    where
        Self: Sized,
    {
        panic!("get_none() should not be called on non-Option types")
    }
}

impl QueryGetType for EntityId {
    #[inline]
    fn get_type_id() -> QueryComponentType {
        QueryComponentType::Normal(<EntityId as Component>::get_type_id())
    }
}

impl<C: Component> QueryGetType for &C {
    #[inline]
    fn get_type_id() -> QueryComponentType {
        QueryComponentType::Normal(C::get_type_id())
    }
}

impl<C: Component> QueryGetType for &mut C {
    #[inline]
    fn get_type_id() -> QueryComponentType {
        QueryComponentType::Normal(C::get_type_id())
    }
}

impl<C: Component> QueryGetType for Ref<'_, C> {
    #[inline]
    fn get_type_id() -> QueryComponentType {
        QueryComponentType::Normal(C::get_type_id())
    }
}

impl<C: Component> QueryGetType for Mut<'_, C> {
    #[inline]
    fn get_type_id() -> QueryComponentType {
        QueryComponentType::Normal(C::get_type_id())
    }
}

impl<C: Component> QueryGetType for Option<&C> {
    #[inline]
    fn get_type_id() -> QueryComponentType {
        QueryComponentType::Option(C::get_type_id())
    }

    #[inline]
    fn get_none() -> Self
    where
        Self: Sized,
    {
        None
    }
}

impl<C: Component> QueryGetType for Option<&mut C> {
    #[inline]
    fn get_type_id() -> QueryComponentType {
        QueryComponentType::Option(C::get_type_id())
    }

    #[inline]
    fn get_none() -> Self
    where
        Self: Sized,
    {
        None
    }
}

/// Downcast the requested component archetype data into the correct target type
trait QueryGetDowncasted<'a> {
    type Output;
    fn get_downcasted(comp: &mut ComponentsData, index: usize, tick: Tick) -> Self::Output;
}

impl<'a> QueryGetDowncasted<'a> for EntityId {
    type Output = EntityId;
    #[inline]
    fn get_downcasted(comp: &mut ComponentsData, index: usize, _: Tick) -> Self::Output {
        unsafe {
            *comp
                .get_untyped_lt(index)
                .as_ptr()
                .cast::<EntityId>()
                .as_ref()
        }
    }
}

impl<'a, C: Component> QueryGetDowncasted<'a> for &C {
    type Output = &'a C;
    #[inline]
    fn get_downcasted(comp: &mut ComponentsData, index: usize, _: Tick) -> Self::Output {
        unsafe { comp.get_untyped_lt(index).as_ptr().cast::<C>().as_ref() }
    }
}

impl<'a, C: Component> QueryGetDowncasted<'a> for &mut C {
    type Output = &'a mut C;
    #[inline]
    fn get_downcasted(comp: &mut ComponentsData, index: usize, tick: Tick) -> Self::Output {
        comp.set_changed_at(index, tick);
        unsafe { comp.get_untyped_lt(index).as_ptr().cast::<C>().as_mut() }
    }
}

impl<'a, C: Component> QueryGetDowncasted<'a> for Ref<'a, C> {
    type Output = Ref<'a, C>;
    #[inline]
    fn get_downcasted(comp: &mut ComponentsData, index: usize, tick: Tick) -> Self::Output {
        let data = comp.get(index, tick, tick);
        Ref::new(data)
    }
}

impl<'a, C: Component> QueryGetDowncasted<'a> for Mut<'a, C> {
    type Output = Mut<'a, C>;
    #[inline]
    fn get_downcasted(comp: &mut ComponentsData, index: usize, tick: Tick) -> Self::Output {
        let data = comp.get_mut(index, tick, tick);
        Mut::new(data)
    }
}

impl<'a, C: QueryGetDowncasted<'a>> QueryGetDowncasted<'a> for Option<C> {
    type Output = Option<C::Output>;
    #[inline]
    fn get_downcasted(comp: &mut ComponentsData, index: usize, tick: Tick) -> Self::Output {
        Some(C::get_downcasted(comp, index, tick))
    }
}

macro_rules! impl_run_query {
    ($($lt:lifetime $types:ident),+; $($filter:ident),*) => {
        #[allow(unused_parens)]
        impl<$($lt),+, $($types),+, $($filter),*> RunQuery for Query<($($types),+), ($($filter),*)>
        where
            $(
                $types: QueryGetType + QueryGetDowncasted<$lt, Output = $types>
            ,)+
            $(
                $filter: QueryFilter
            ),*
        {
            type Output = ($($types),+);

            fn iter_mut(&mut self) -> Vec<($($types),+)> {
                let mut filters = Filters::new();
                $(filters.add::<$filter>();)*

                let requested_types = [$($types::get_type_id()),+];
                let mut result = Vec::new();
                let entities = unsafe { &mut *self.entities };
                let current_tick = entities.tick();

                // Iterate over archetypes that match the query
                for (archetype, changed_filter_indices) in entities.archetypes_filtered(&requested_types, &mut filters) {
                    let mut type_index = 0;
                    // Extract specific component vecs and their indices into a $type variable
                    $(
                        #[allow(non_snake_case)]
                        #[allow(unused_assignments)]
                        let $types = {
                            // TODO: use [${index()}] once meta vars are stabilized
                            let query_type = &requested_types[type_index];
                            let type_id = query_type.get_inner_type();
                            type_index += 1;

                            let maybe_index = if query_type.is_option() {
                                // Don't panic since Option doesn't have to be present
                                archetype.try_component_index(type_id)
                            } else {
                                Some(archetype.component_index(type_id))
                            };

                            if let Some(index) = maybe_index {
                                Some(archetype.get_components_data_mut(index))
                            } else {
                                None
                            }
                        };
                    )+

                    for entity_index in 0..archetype.len() {
                        if !archetype.check_changed_fields(entity_index, &changed_filter_indices, self.system_last_run) {
                            continue;
                        }

                        // SAFETY: We know that the components are of the correct type $type
                        result.push(($(unsafe {
                            if let Some(components) = $types {
                                $types::get_downcasted(&mut *components, entity_index, current_tick)
                            } else {
                                // If requested type is Option<T> and isn't present
                                $types::get_none()
                            }
                        }),+));
                    }
                }

                result
            }

            fn get(&mut self, entity_id: EntityId) -> Option<($($types),+)> {
                let mut filters = Filters::new();
                $(filters.add::<$filter>();)*

                let requested_types = [$($types::get_type_id()),+];
                let entities = unsafe { &mut *self.entities };
                let current_tick = entities.tick();

                // Get the entity location
                let location = entities.tracking.get_location(entity_id)?;

                let entity_index = location.index();
                let id = location.archetype_id();
                let archetype = entities.archetypes.get_mut(&id).expect("archetype should exist");

                // Check if the archetype matches the query
                if let Some(changed_filter_indices) = archetype.filtered(&requested_types, &mut filters) {
                    let mut type_index = 0;
                    // Extract specific component vecs and their indices into a $type variable
                    $(
                        #[allow(non_snake_case)]
                        #[allow(unused_assignments)]
                        let $types = {
                            let query_type = &requested_types[type_index];
                            let type_id = query_type.get_inner_type();
                            type_index += 1;

                            let maybe_index = if query_type.is_option() {
                                // Don't panic since Option doesn't have to be present
                                archetype.try_component_index(type_id)
                            } else {
                                Some(archetype.component_index(type_id))
                            };

                            if let Some(index) = maybe_index {
                                Some(archetype.get_components_data_mut(index))
                            } else {
                                None
                            }
                        };
                    )+

                    if !archetype.check_changed_fields(entity_index, &changed_filter_indices, self.system_last_run) {
                        return None;
                    }

                    // SAFETY: We know that the components are of the correct type $type
                    return Some(($(unsafe {
                        if let Some(components) = $types {
                            $types::get_downcasted(&mut *components, entity_index, current_tick)
                        } else {
                            // If requested type is Option<T> and isn't present
                            $types::get_none()
                        }
                    }),+));
                }

                None
            }

            // #[allow(unused_parens)]
            // fn iter(&mut self) -> Vec<($(&'b $types),+)> {
            //     let requested_types = vec![$(TypeId::of::<$types>()),+];
            //     let mut result = Vec::new();
            //
            //     for archetype in self.entities.archetypes_filtered(&requested_types) {
            //         // Extract specific component vecs into a $type variable
            //         $(
            //             #[allow(non_snake_case)]
            //             let $types = {
            //                 let type_id = TypeId::of::<$types>();
            //                 let index = *archetype.types().get(&type_id).expect("type should exist in archetype");
            //                 archetype.components_at(index)
            //             };
            //         )+
            //
            //         for i in 0..archetype.len() {
            //             result.push(($(unsafe {
            //                 (&*$types)[i]
            //                     .downcast_ref::<$types>()
            //                     .expect("variable $type[i] should downcast into $type")
            //             }),+));
            //         }
            //     }
            //
            //     result
            // }
        }
    };
}

impl_run_query!('a T; );
impl_run_query!('a T; F);

impl_run_query!('a T, 'b U; );
impl_run_query!('a T, 'b U; F);

impl_run_query!('a T, 'b U, 'c V; );
impl_run_query!('a T, 'b U, 'c V; F);

impl_run_query!('a T, 'b U, 'c V, 'd W; );
impl_run_query!('a T, 'b U, 'c V, 'd W; F);

impl_run_query!('a T, 'b U, 'c V, 'd W, 'e X; );
impl_run_query!('a T, 'b U, 'c V, 'd W, 'e X; F);

impl_run_query!('a T, 'b U, 'c V, 'd W, 'e X, 'f Y; );
impl_run_query!('a T, 'b U, 'c V, 'd W, 'e X, 'f Y; F);

impl_run_query!('a T, 'b U, 'c V, 'd W, 'e X, 'f Y, 'g Z; );
impl_run_query!('a T, 'b U, 'c V, 'd W, 'e X, 'f Y, 'g Z; F);

// impl<'a, 'b, T: 'static, U: 'static> RunQuery<(&'b mut T, &'b mut U)>
// for Query<'a, (&'b mut T, &'b mut U)>
// where 'a: 'b
// {
//     fn iter_mut(&mut self) -> Vec<(&'b mut T, &'b mut U)> {
//         let type_id_t = TypeId::of::<T>();
//         let type_id_u = TypeId::of::<U>();
//
//         let type_id = TypeId::of::<(T, U)>();
//         let archetypes = self.world.archetypes.get_mut(&type_id).unwrap();
//
//         let index_t = archetypes.types.get(&type_id_t).unwrap();
//         let index_u = archetypes.types.get(&type_id_u).unwrap();
//
//         let split = archetypes.components.split_at_mut(*index_t);
//
//         let components_t = &mut split.0[*index_t] as *mut Vec<Box<dyn Any>>;
//         let components_u = &mut split.1[*index_u] as *mut Vec<Box<dyn Any>>;
//         // let components_u = &mut split.1[*index_u];
//
//         let mut result = Vec::new();
//         for i in 0..archetypes.entities.len() {
//             let t = unsafe { &mut *components_t }[i].downcast_mut::<T>().unwrap();
//             let u = unsafe { &mut *components_u }[i].downcast_mut::<U>().unwrap();
//             result.push((t, u));
//         }
//
//         result
//     }
// }

// struct QueryStruct<'a, 'b, T> {
//     world: &'a mut World,
//     components: Vec<&'b mut Vec<Box<dyn Any>>>,
//     _marker: std::marker::PhantomData<T>,
// }
// trait PrepareQuery<'a> {
//     fn prepare(world: &'a mut World) -> Self;
// }
// impl<'a, 'b, T: 'static> PrepareQuery<'a> for QueryStruct<'a, 'b, T> where 'a: 'b {
//     fn prepare(world: &'a mut World) -> Self {
//         let type_id = TypeId::of::<T>();
//
//         let archetypes = world.archetypes.get_mut(&type_id).unwrap();
//
//         let index_t = archetypes.types.get(&type_id).unwrap();
//
//         let split = archetypes.components.split_at_mut(*index_t);
//
//         let components_t = &mut split.0[*index_t];
//
//
//         QueryStruct {
//             world,
//             components: vec![components_t],
//             _marker: std::marker::PhantomData,
//         }
//     }
// }
