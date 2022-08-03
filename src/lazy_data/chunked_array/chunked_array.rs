use std::marker::PhantomData;

pub struct ChunkedArray<Dim, T> {
    data: Vec<T>,
    chunk_size: usize,
    _dim: PhantomData<Dim>,
}

pub struct I1;
pub struct I2;
pub struct I3;
pub struct I4;