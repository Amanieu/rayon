use super::*;
use super::internal::*;

pub struct SliceIter<'data, T: 'data + Sync> {
    slice: &'data [T]
}

impl<'data, T: Sync> IntoParallelIterator for &'data [T] {
    type Item = &'data T;
    type Iter = SliceIter<'data, T>;

    fn into_par_iter(self) -> Self::Iter {
        SliceIter { slice: self }
    }
}

impl<'data, T: Sync + 'data> IntoParallelRefIterator<'data> for [T] {
    type Item = T;
    type Iter = SliceIter<'data, T>;

    fn par_iter(&'data self) -> Self::Iter {
        self.into_par_iter()
    }
}

impl<'data, T: Sync + 'data> ParallelIterator for SliceIter<'data, T> {
    type Item = &'data T;

    fn drive_unindexed<'c, C: UnindexedConsumer<'c, Item=Self::Item>>(self,
                                                                      consumer: C,
                                                                      shared: &'c C::Shared)
                                                                      -> C::Result {
        bridge(self, consumer, &shared)
    }
}

unsafe impl<'data, T: Sync + 'data> BoundedParallelIterator for SliceIter<'data, T> {
    fn upper_bound(&mut self) -> usize {
        ExactParallelIterator::len(self)
    }

    fn drive<'c, C: Consumer<'c, Item=Self::Item>>(self,
                                                   consumer: C,
                                                   shared: &'c C::Shared)
                                                   -> C::Result {
        bridge(self, consumer, &shared)
    }
}

unsafe impl<'data, T: Sync + 'data> ExactParallelIterator for SliceIter<'data, T> {
    fn len(&mut self) -> usize {
        self.slice.len()
    }
}

impl<'data, T: Sync + 'data> IndexedParallelIterator for SliceIter<'data, T> {
    fn into_producer<'p>(&'p mut self) -> (<Self as ProducerType<'p>>::Producer,
                                           <<Self as ProducerType<'p>>::Producer as Producer>::Shared) {
        (SliceProducer { slice: self.slice }, ())
    }
}

impl<'p, 'data, T: Sync + 'data> ProducerType<'p> for SliceIter<'data, T> {
    type Producer = SliceProducer<'data, T>;
    type ProducedItem = <Self::Producer as Producer<'p>>::Item;
}

///////////////////////////////////////////////////////////////////////////

pub struct SliceProducer<'data, T: 'data + Sync> {
    slice: &'data [T]
}

impl<'p, 'data, T: 'data + Sync> Producer<'p> for SliceProducer<'data, T>
{
    type Item = &'p T;
    type Shared = ();

    fn cost(&mut self, _shared: &Self::Shared, len: usize) -> f64 {
        len as f64
    }

    unsafe fn split_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.slice.split_at(index);
        (SliceProducer { slice: left }, SliceProducer { slice: right })
    }

    unsafe fn produce(&mut self, _: &()) -> &'data T {
        let (head, tail) = self.slice.split_first().unwrap();
        self.slice = tail;
        head
    }
}
