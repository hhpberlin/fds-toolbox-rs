pub trait IdSource {
    type Id;
    type Iter<'a>: Iterator<Item = Self::Id> + 'a
    where
        Self: 'a;
    fn iter_ids(&self) -> Self::Iter<'_>;
}

impl<IdSrc: IdSource> IdSource for &IdSrc {
    type Id = IdSrc::Id;
    type Iter<'a> = IdSrc::Iter<'a>
    where
        Self: 'a;
    fn iter_ids(&self) -> Self::Iter<'_> {
        (*self).iter_ids()
    }
}