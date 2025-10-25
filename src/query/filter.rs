use std::{any::TypeId, marker::PhantomData};

use crate::prelude::Component;

/// A filter that checks if a component is marked as changed in the current frame. That is, if the
/// component was requested as a mutable reference in a query.
pub struct Changed<C: Component>(PhantomData<C>);

/// A filter that checks if a component is present.
pub struct With<C: Component>(PhantomData<C>);

/// A filter that checks if a component is **not** present.
pub struct Without<C: Component>(PhantomData<C>);

/// A filter that checks if a component was added since the last tick the system ran.
pub struct Added<C: Component>(PhantomData<C>);

/// A special filter that checks if any of the [filters](QueryFilter) evaluate to true.
/// Nested Ors are not supported.
#[allow(private_bounds)]
pub struct Or<F: QueryFilter>(PhantomData<F>);

/// This trait defines what can be used as a filter in a query
pub(crate) trait QueryFilter {
    /// Parses `Self` and applies it to `filters`
    fn into_filters(filters: &mut Filters);
}

impl<C: Component> QueryFilter for Changed<C> {
    #[inline]
    fn into_filters(filters: &mut Filters) {
        filters.changed.push(C::get_type_id())
    }
}

impl<C: Component> QueryFilter for With<C> {
    #[inline]
    fn into_filters(filters: &mut Filters) {
        filters.with.push(C::get_type_id())
    }
}

impl<C: Component> QueryFilter for Without<C> {
    #[inline]
    fn into_filters(filters: &mut Filters) {
        filters.without.push(C::get_type_id())
    }
}

impl<C: Component> QueryFilter for Added<C> {
    #[inline]
    fn into_filters(filters: &mut Filters) {
        filters.added.push(C::get_type_id())
    }
}

impl<F: QueryFilter> QueryFilter for Or<F> {
    #[inline]
    fn into_filters(filters: &mut Filters) {
        let or_filters = Filters::from::<F>();
        filters.or.push(or_filters);
    }
}

impl QueryFilter for () {
    #[inline]
    fn into_filters(_filters: &mut Filters) {
        // No filters to add
    }
}

macro_rules! impl_query_filter {
    ($($type:ident),+) => {
        impl<$($type: QueryFilter),+> QueryFilter for ($($type,)+) {
            fn into_filters(filters: &mut Filters) {
                $(
                    $type::into_filters(filters);
                )+
            }
        }
    };
}

impl_query_filter!(A);
impl_query_filter!(A, B);
impl_query_filter!(A, B, C);
impl_query_filter!(A, B, C, D);
impl_query_filter!(A, B, C, D, E);
impl_query_filter!(A, B, C, D, E, F);
impl_query_filter!(A, B, C, D, E, F, G);
impl_query_filter!(A, B, C, D, E, F, G, H);
impl_query_filter!(A, B, C, D, E, F, G, H, I);
impl_query_filter!(A, B, C, D, E, F, G, H, I, J);
impl_query_filter!(A, B, C, D, E, F, G, H, I, J, K);
impl_query_filter!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_query_filter!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_query_filter!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_query_filter!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_query_filter!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

/// Struct to store parsed T query filters
#[derive(Debug)]
pub(crate) struct Filters {
    pub changed: Vec<TypeId>,
    pub added: Vec<TypeId>,
    pub with: Vec<TypeId>,
    pub without: Vec<TypeId>,
    pub or: Vec<Filters>,
    pub empty: bool,

    /// Used inside of an `Or` filter, indicates if `with` or `without` filters evaluate to true,
    /// if so, it skips any further `changed` checks
    pub matches_existence: bool,
}

impl Filters {
    pub fn new() -> Self {
        Self {
            changed: Vec::new(),
            added: Vec::new(),
            with: Vec::new(),
            without: Vec::new(),
            or: Vec::new(),
            empty: true,
            matches_existence: false,
        }
    }

    /// Create a new `Filters` instance with populated filters from 'F'
    pub fn from<F: QueryFilter>() -> Filters {
        let mut filters = Filters::new();
        filters.add::<F>();
        filters
    }

    /// Appends filters from 'F'
    pub fn add<F: QueryFilter>(&mut self) {
        F::into_filters(self);
        self.empty = false;
    }
}
