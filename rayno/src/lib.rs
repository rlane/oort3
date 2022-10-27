// Copied from https://github.com/shssoichiro/oxipng/blob/master/src/rayon.rs and modified.
use std::cmp::Ordering;

pub mod prelude {
    pub use super::*;
}

pub trait ParallelIterator: Iterator + Sized {
    fn with_max_len(self, _l: usize) -> Self {
        self
    }

    fn reduce_with<OP>(mut self, op: OP) -> Option<Self::Item>
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        self.next().map(|a| self.fold(a, op))
    }

    fn map_with<F, T, R>(self, init: T, map_op: F) -> MapWith<Self, T, F>
    where
        F: Fn(&mut T, Self::Item) -> R + Sync + Send,
        T: Send + Clone,
        R: Send,
    {
        MapWith {
            base: self,
            item: init,
            map_op,
        }
    }
}

#[derive(Clone)]
pub struct MapWith<I: Iterator, T, F> {
    base: I,
    item: T,
    map_op: F,
}

impl<B, I: Iterator, T, F> Iterator for MapWith<I, T, F>
where
    F: Fn(&mut T, I::Item) -> B,
{
    type Item = B;

    fn next(&mut self) -> Option<B> {
        self.base.next().map(|x| (self.map_op)(&mut self.item, x))
    }
}

pub trait IntoParallelIterator {
    type Iter: Iterator<Item = Self::Item>;
    type Item: Send;
    fn into_par_iter(self) -> Self::Iter;
}

pub trait IntoParallelRefIterator<'data> {
    type Iter: Iterator<Item = Self::Item>;
    type Item: Send + 'data;
    fn par_iter(&'data self) -> Self::Iter;
}

impl<I: IntoIterator> IntoParallelIterator for I
where
    I::Item: Send,
{
    type Iter = I::IntoIter;
    type Item = I::Item;

    fn into_par_iter(self) -> Self::Iter {
        self.into_iter()
    }
}

impl<'data, I: 'data + ?Sized> IntoParallelRefIterator<'data> for I
where
    &'data I: IntoParallelIterator,
{
    type Iter = <&'data I as IntoParallelIterator>::Iter;
    type Item = <&'data I as IntoParallelIterator>::Item;

    fn par_iter(&'data self) -> Self::Iter {
        self.into_par_iter()
    }
}

impl<I: Iterator> ParallelIterator for I {}

pub fn join<A, B>(a: impl FnOnce() -> A, b: impl FnOnce() -> B) -> (A, B) {
    (a(), b())
}

pub fn spawn<A>(a: impl FnOnce() -> A) -> A {
    a()
}

pub trait ParallelSliceMut<T: Send> {
    fn as_parallel_slice_mut(&mut self) -> &mut [T];

    fn par_sort_by<F>(&mut self, compare: F)
    where
        F: Fn(&T, &T) -> Ordering + Sync,
    {
        self.as_parallel_slice_mut().sort_by(compare);
    }
}

impl<T: Send> ParallelSliceMut<T> for [T] {
    #[inline]
    fn as_parallel_slice_mut(&mut self) -> &mut [T] {
        self
    }
}
