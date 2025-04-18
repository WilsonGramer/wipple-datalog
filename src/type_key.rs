use std::any::TypeId;

// Hack because `TypeId::of` isn't const stable
// (https://github.com/rust-lang/rust/issues/77125#issuecomment-2799071176)
#[derive(Debug, Clone, Copy)]
pub(crate) struct TypeKey(fn() -> TypeId);

impl TypeKey {
    pub(crate) const fn of<T: std::any::Any>() -> TypeKey {
        TypeKey(std::any::TypeId::of::<T>)
    }

    pub(crate) fn type_id(&self) -> TypeId {
        (self.0)()
    }
}

#[derive(Clone, Copy)]
pub struct Erased {
    _private: (),
}
