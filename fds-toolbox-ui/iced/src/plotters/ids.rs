
pub trait IdSource {
    type Id;
    type Iter<'a>: Iterator<Item = Self::Id> + 'a
    where
        Self: 'a;
    fn iter_ids(&self) -> Self::Iter<'_>;
}