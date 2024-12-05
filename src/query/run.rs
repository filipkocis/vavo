use std::any::{Any, TypeId};

use super::Query;

pub trait RunQuery<T, U> {
    fn iter_mut(&mut self) -> Vec<T>;
    fn iter(&mut self) -> Vec<U>;
}

macro_rules! impl_run_query {
    // Base case for a single pair of types
    ($($types:ident),+) => {
        #[allow(unused_parens)]
        impl<'a, 'b, $($types: 'static),+> 
            RunQuery<
                ($(&'b mut $types),+),
                ($(&'b $types),+)
            >
        for Query<'a, ($($types),+)>
        where 'a: 'b
        {
            #[allow(unused_parens)]
            fn iter_mut(&mut self) -> Vec<($(&'b mut $types),+)> {
                let requested_types = vec![$(TypeId::of::<$types>()),+];
                let mut result = Vec::new();

                for archetype in self.entities.archetypes_filtered(&requested_types) {
                    // Extract specific component vecs into a $type variable
                    $(
                        #[allow(non_snake_case)]
                        let $types = {
                            let type_id = TypeId::of::<$types>();
                            let index = *archetype.types().get(&type_id).expect("type should exist in archetype");
                            &mut archetype.components[index] as *mut Vec<Box<dyn Any>>
                        };
                    )+

                    for i in 0..archetype.len() {
                        result.push(($(unsafe { 
                            (&mut *$types)[i]
                                .downcast_mut::<$types>()
                                .expect("variable $type[i] should downcast into $type")
                        }),+));
                    }
                }

                result
            }

            #[allow(unused_parens)]
            fn iter(&mut self) -> Vec<($(&'b $types),+)> {
                let requested_types = vec![$(TypeId::of::<$types>()),+];
                let mut result = Vec::new();

                for archetype in self.entities.archetypes_filtered(&requested_types) {
                    // Extract specific component vecs into a $type variable
                    $(
                        #[allow(non_snake_case)]
                        let $types = {
                            let type_id = TypeId::of::<$types>();
                            let index = *archetype.types().get(&type_id).expect("type should exist in archetype");
                            &archetype.components[index] as *const Vec<Box<dyn Any>>
                        };
                    )+

                    for i in 0..archetype.len() {
                        result.push(($(unsafe { 
                            (&*$types)[i]
                                .downcast_ref::<$types>()
                                .expect("variable $type[i] should downcast into $type")
                        }),+));
                    }
                }

                result
            }
        }
    };
}

impl_run_query!(T);
impl_run_query!(T, U);
impl_run_query!(T, U, V);
impl_run_query!(T, U, V, W);
impl_run_query!(T, U, V, W, X);
impl_run_query!(T, U, V, W, X, Y);
impl_run_query!(T, U, V, W, X, Y, Z);

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
