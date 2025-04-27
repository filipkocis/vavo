use super::prelude::*;

impl<'a, C: Component> Mut<'a, C> {
    /// Same as `deref_mut()` but without the change detection.
    #[inline]
    pub fn deref_mut_no_change(&mut self) -> &mut C {
        let raw = self.0.raw() as *mut C;
        unsafe { &mut *raw }
    }
}

impl<R: Resource> ResMut<R> {
    /// Same as `deref_mut()` but without the change detection.
    #[inline]
    pub fn deref_mut_no_change(&mut self) -> &mut R {
        let raw = self.0.raw() as *mut R;
        unsafe { &mut *raw }
    }
}

pub trait ChangeDetection {
    /// Returns the tick of when the component was last changed.
    fn changed_at(&self) -> u64;
    /// Returns the tick of when the component was added.
    fn added_at(&self) -> u64;
    /// Returns whether the component has changed since the last time the system ran.
    fn has_changed(&self) -> bool;
    /// Returns whether the component was added since the last time the system ran.
    fn was_added(&self) -> bool;
}

macro_rules! impl_change_detection {
    // Foo<'lt, T> -> Foo<T>
    ($name:ident<$($lt:lifetime, )*$id:ident: $ty:ident>) => {
        impl<$($lt, )* $id: $ty> ChangeDetection for $name<$($lt, )* $id> {
            #[inline]
            fn changed_at(&self) -> u64 {
                self.0.stamp().changed()
            }

            #[inline]
            fn added_at(&self) -> u64 {
                self.0.stamp().added()
            }

            #[inline]
            fn has_changed(&self) -> bool {
                self.changed_at() > self.0.stamp().last_run()
            }

            #[inline]
            fn was_added(&self) -> bool {
                self.added_at() > self.0.stamp().last_run()
            }
        }
    };
}

impl_change_detection!(Ref<'a, C: Component>);
impl_change_detection!(Mut<'a, C: Component>);
impl_change_detection!(Res<R: Resource>);
impl_change_detection!(ResMut<R: Resource>);
