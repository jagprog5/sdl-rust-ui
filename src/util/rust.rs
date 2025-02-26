use std::ops::{Deref, DerefMut};

/// give a lifetime which is a subset of the existing lifetime
pub fn reborrow<'in_life, 'out_life, T: ?Sized>(something: &'in_life mut T) -> &'out_life mut T
where
    'in_life: 'out_life,
{
    &mut *something
}

/// references to cell or value
pub enum CellRefOrCell<'a, T> {
    Ref(&'a std::cell::Cell<T>),
    Cell(std::cell::Cell<T>),
}

// revisit. perhaps lang improvements will help? SFINAE. conflicts with From<&'a
// std::cell::Cell<T>>
//
// impl<'a, T> From<&T> for CellRefOrCell<'a, T::Owned> where T: ToOwned +
// ?Sized, { fn from(value: &T) -> Self {
//     CellRefOrCell::Cell(std::cell::Cell::new(value.to_owned())) } }

// might be more generic via above commented out later
impl<'a> From<&str> for CellRefOrCell<'a, String> {
    fn from(value: &str) -> Self {
        CellRefOrCell::from(value.to_owned())
    }
}
impl<'a> From<String> for CellRefOrCell<'a, String> {
    fn from(value: String) -> Self {
        CellRefOrCell::Cell(std::cell::Cell::new(value))
    }
}

impl<'a, T> From<&'a std::cell::Cell<T>> for CellRefOrCell<'a, T> {
    fn from(value: &'a std::cell::Cell<T>) -> Self {
        CellRefOrCell::Ref(value)
    }
}

impl<T> From<std::cell::Cell<T>> for CellRefOrCell<'_, T> {
    fn from(value: std::cell::Cell<T>) -> Self {
        CellRefOrCell::Cell(value)
    }
}

impl<'a, T: Copy> CellRefOrCell<'a, T> {
    pub fn get(&self) -> T {
        match self {
            CellRefOrCell::Ref(cell) => cell.get(),
            CellRefOrCell::Cell(cell) => cell.get(),
        }
    }
}

impl<'a, T: Default> CellRefOrCell<'a, T> {
    pub fn take(&self) -> T {
        match self {
            CellRefOrCell::Ref(r) => r.take(),
            CellRefOrCell::Cell(b) => b.take(),
        }
    }

    pub fn scope_take(&self) -> ScopeTake<'_, T> {
        ScopeTake {
            source: self,
            holder: self.take(),
        }
    }
}

impl<'a, T> CellRefOrCell<'a, T> {
    pub fn replace(&self, value: T) -> T {
        match self {
            CellRefOrCell::Ref(cell) => cell.replace(value),
            CellRefOrCell::Cell(cell) => cell.replace(value),
        }
    }

    pub fn set(&self, value: T) {
        match self {
            CellRefOrCell::Ref(r) => r.set(value),
            CellRefOrCell::Cell(b) => b.set(value),
        }
    }
}

/// raii over ref to contents in CellRefOrCell. takes content and puts it back
/// when dropped
pub struct ScopeTake<'a, T: Default> {
    source: &'a CellRefOrCell<'a, T>,
    holder: T,
}

impl<'a, T: Default> Deref for ScopeTake<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.holder
    }
}

impl<'a, T: Default> DerefMut for ScopeTake<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.holder
    }
}

impl<'a, T: Default> Drop for ScopeTake<'a, T> {
    fn drop(&mut self) {
        self.source.set(std::mem::take(&mut self.holder));
    }
}
