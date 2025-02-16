use std::{any::TypeId, marker::PhantomData};

/// A filter that checks if a component is marked as changed in the current frame. That is, if the
/// component was requested as a mutable reference in a query.
pub struct Changed<T>(PhantomData<T>);

/// A filter that checks if a component is present.
pub struct With<T>(PhantomData<T>);

/// A filter that checks if a component is **not** present.
pub struct Without<T>(PhantomData<T>);

/// A special filter that checks if any of the [filters](QueryFilter) evaluate to true. 
/// Nested Ors are not supported.
#[allow(private_bounds)]
pub struct Or<F: QueryFilter>(PhantomData<F>);

/// This trait defines what can be used as a filter in a query
pub(crate) trait QueryFilter {
    /// Parses `Self` and applies it to `filters`
    fn into_filters(filters: &mut Filters);
}

impl<T: 'static> QueryFilter for Changed<T> {
    fn into_filters(filters: &mut Filters) {
        filters.changed.push(TypeId::of::<T>())
    }
}

impl<T: 'static> QueryFilter for With<T> {
    fn into_filters(filters: &mut Filters) {
        filters.with.push(TypeId::of::<T>())
    }
}

impl<T: 'static> QueryFilter for Without<T> {
    fn into_filters(filters: &mut Filters) {
        filters.without.push(TypeId::of::<T>())
    }
}

impl<F: QueryFilter> QueryFilter for Or<F> {
    fn into_filters(filters: &mut Filters) {
        let or_filters = Filters::from::<F>();
        filters.or.push(or_filters); 
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

impl_query_filter!(A, B);
impl_query_filter!(A, B, C);
impl_query_filter!(A, B, C, D);
impl_query_filter!(A, B, C, D, E);

/// Struct to store parsed T query filters
#[derive(Debug)]
pub(crate) struct Filters {
    pub changed: Vec<TypeId>,
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

    /// Checks if any relevant `changed` filters are present
    pub fn has_changed_filters(&self) -> bool {
        !self.changed.is_empty() || self.or.iter().any(|f| 
            !f.changed.is_empty() && !f.matches_existence
        )
    }
}
