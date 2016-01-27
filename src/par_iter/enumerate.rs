use super::*;
use super::internal::*;

pub struct Enumerate<M> {
    base: M,
}

impl<M> Enumerate<M> {
    pub fn new(base: M) -> Enumerate<M> {
        Enumerate { base: base }
    }
}

impl<M> ParallelIterator for Enumerate<M>
    where M: IndexedParallelIterator,
{
    type Item = (usize, M::Item);

    fn drive_unindexed<'c, C: UnindexedConsumer<'c, Item=Self::Item>>(self,
                                                                       consumer: C,
                                                                       shared: &'c C::Shared)
                                                                       -> C::Result {
        bridge(self, consumer, &shared)
    }
}

unsafe impl<M> BoundedParallelIterator for Enumerate<M>
    where M: IndexedParallelIterator,
{
    fn upper_bound(&mut self) -> usize {
        self.len()
    }

    fn drive<'c, C: Consumer<'c, Item=Self::Item>>(self,
                                                   consumer: C,
                                                   shared: &'c C::Shared)
                                                   -> C::Result {
        bridge(self, consumer, &shared)
    }
}

unsafe impl<M> ExactParallelIterator for Enumerate<M>
    where M: IndexedParallelIterator,
{
    fn len(&mut self) -> usize {
        self.base.len()
    }
}

impl<M> IndexedParallelIterator for Enumerate<M>
    where M: IndexedParallelIterator,
{
    fn into_producer<'p>(&'p mut self) -> (ProducerFor<'p, Self>, SharedFor<'p, Self>) {
        let (base, shared) = self.base.into_producer();
        (EnumerateProducer { base: base, offset: 0 }, shared)
    }
}

impl<'p, M> ProducerType<'p> for Enumerate<M>
    where M: IndexedParallelIterator,
{
    type Producer = EnumerateProducer<<M as ProducerType<'p>>::Producer>;
    type ProducedItem = <Self::Producer as Producer<'p>>::Item;
}

///////////////////////////////////////////////////////////////////////////
// Producer implementation

pub struct EnumerateProducer<M> {
    base: M,
    offset: usize,
}

impl<'p, M> Producer<'p> for EnumerateProducer<M>
    where M: Producer<'p>
{
    type Item = (usize, M::Item);
    type Shared = M::Shared;

    fn cost(&mut self, shared: &Self::Shared, items: usize) -> f64 {
        self.base.cost(shared, items) // enumerating is basically free
    }

    unsafe fn split_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.base.split_at(index);
        (EnumerateProducer { base: left, offset: self.offset },
         EnumerateProducer { base: right, offset: self.offset + index })
    }

    unsafe fn produce(&mut self, shared: &Self::Shared) -> (usize, M::Item) {
        let item = self.base.produce(shared);
        let index = self.offset;
        self.offset += 1;
        (index, item)
    }
}
