use std::any::{Any, TypeId};

use crate::ecs::entities::EntityId;

use super::{Query, filter::{Filters, QueryFilter}};

pub trait RunQuery {
    type Output;

    fn iter_mut(&mut self) -> Vec<Self::Output>;
    fn get(&mut self, entity_id: EntityId) -> Option<Self::Output>;
}

trait QueryGetType {
    fn get_type_id() -> TypeId;
}

impl<T: 'static> QueryGetType for &T {
    fn get_type_id() -> TypeId {
        TypeId::of::<T>()
    }
}

impl<T: 'static> QueryGetType for &mut T {
    fn get_type_id() -> TypeId {
        TypeId::of::<T>()
    }
}

trait QueryGetDowncasted<'a> {
    type Output;
    fn get_downcasted(comp: &'a mut Box<dyn Any>) -> Self::Output;
    fn is_mut() -> bool {
        false
    }
}

impl<'a, Type: 'static> QueryGetDowncasted<'a> for &Type {
    type Output = &'a Type;
    fn get_downcasted(comp: &'a mut Box<dyn Any>) -> Self::Output {
        comp.downcast_ref::<Type>().expect("downcast failed")
    }
}

impl<'a, T: 'static> QueryGetDowncasted<'a> for &mut T {
    type Output = &'a mut T;
    fn get_downcasted(comp: &'a mut Box<dyn Any>) -> Self::Output {
        comp.downcast_mut::<T>().expect("downcast failed")
    }

    fn is_mut() -> bool {
        true
    }
}

macro_rules! impl_run_query {
    ($($types:ident),+; $($filter:ident),*) => {
        #[allow(unused_parens)]
        impl<'t, $($types),+, $($filter),*> RunQuery for Query<($($types),+), ($($filter),*)>
        where
            $(
                $types: QueryGetType + QueryGetDowncasted<'t, Output = $types>
            ,)+
            $(
                $filter: QueryFilter
            ),*
        {
            type Output = ($($types),+);

            #[allow(unused_parens)]
            fn iter_mut(&mut self) -> Vec<($($types),+)> {
                #[allow(unused_mut)]
                let mut filters = Filters::new();
                $(filters.add::<$filter>();)*
                let has_changed_filters = filters.has_changed_filters();

                let requested_types = vec![$($types::get_type_id()),+];
                let mut result = Vec::new();

                for archetype in unsafe { &mut *self.entities }.archetypes_filtered(&requested_types, &mut filters) {
                    // Extract specific component vecs and their indices into a $type variable
                    $(
                        #[allow(non_snake_case)]
                        let $types = {
                            let type_id = $types::get_type_id();
                            let index = archetype.get_component_index(&type_id).expect("type should exist in archetype");

                            // marks the whole column if query doesn't contain `changed` filters
                            if !has_changed_filters && $types::is_mut() {
                                // Mark all components as mutated if $type is &mut T
                                archetype.mark_mutated(index);
                            }

                            (archetype.components_at_mut(index), index)
                        };
                    )+

                    let component_indices_filter = archetype.get_changed_filter_indices(&filters);
                    for entity_index in 0..archetype.len() {
                        if !archetype.check_changed_fields(entity_index, &component_indices_filter) {
                            continue;
                        }

                        // SAFETY: We know that the components are of the correct type $type
                        result.push(($(unsafe {
                            // marks only specific components if query contains `changed` filters
                            if has_changed_filters && $types::is_mut() {
                                // Mark entity's component as mutated if $type is &mut T
                                archetype.mark_mutated_single(entity_index, $types.1);
                            }

                            $types::get_downcasted(&mut (&mut *$types.0)[entity_index])
                        }),+));
                    }
                }

                result
            }

            fn get(&mut self, entity_id: EntityId) -> Option<($($types),+)> {
                #[allow(unused_mut)]
                let mut filters = Filters::new();
                $(filters.add::<$filter>();)*

                let requested_types = vec![$($types::get_type_id()),+];

                for archetype in unsafe { &mut *self.entities }.archetypes_filtered(&requested_types, &mut filters) {
                    let entity_index = match archetype.get_entity_index(entity_id) {
                        Some(index) => index,
                        None => continue,
                    };

                    // Extract specific component vecs and their indices into a $type variable
                    $(
                        #[allow(non_snake_case)]
                        let $types = {
                            let type_id = $types::get_type_id();
                            let index = archetype.get_component_index(&type_id).expect("type should exist in archetype");

                            (archetype.components_at_mut(index), index)
                        };
                    )+

                    let component_indices_filter = archetype.get_changed_filter_indices(&filters);

                    if !archetype.check_changed_fields(entity_index, &component_indices_filter) {
                        return None;
                    }

                    // SAFETY: We know that the components are of the correct type $type
                    return Some(($(unsafe {
                        if $types::is_mut() {
                            // Mark entity's component as mutated if $type is &mut T
                            archetype.mark_mutated_single(entity_index, $types.1);
                        }

                        $types::get_downcasted(&mut (&mut *$types.0)[entity_index])
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

impl_run_query!(T; );
impl_run_query!(T; F);

impl_run_query!(T, U; );
impl_run_query!(T, U; F);

impl_run_query!(T, U, V; );
impl_run_query!(T, U, V; F);

impl_run_query!(T, U, V, W; );
impl_run_query!(T, U, V, W; F);

impl_run_query!(T, U, V, W, X; );
impl_run_query!(T, U, V, W, X; F);

impl_run_query!(T, U, V, W, X, Y; );
impl_run_query!(T, U, V, W, X, Y; F);

impl_run_query!(T, U, V, W, X, Y, Z; );
impl_run_query!(T, U, V, W, X, Y, Z; F);

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
