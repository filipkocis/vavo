use std::{any::TypeId, marker::PhantomData};

/// A filter that checks if a component is marked as changed in the current frame.
pub struct Changed<T>(PhantomData<T>);

/// A filter that checks if a component is present.
pub struct With<T>(PhantomData<T>);

/// A filter that checks if a component is **not** present.
pub struct Without<T>(PhantomData<T>);

/// This trait defines what can be applied as a filter to a query
pub(crate) trait QueryFilter {
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
pub(crate) struct Filters {
    pub changed: Vec<TypeId>,
    pub with: Vec<TypeId>,
    pub without: Vec<TypeId>,
    pub empty: bool,
}

impl Filters {
    pub fn new() -> Self {
        Self {
            changed: Vec::new(),
            with: Vec::new(),
            without: Vec::new(),
            empty: true,
        }
    }

    pub fn from<T: QueryFilter>() -> Filters {
        let mut filters = Filters::new();
        filters.add::<T>();
        filters
    }

    pub fn add<T: QueryFilter>(&mut self) {
        T::into_filters(self);
        self.empty = false;
    }
}
