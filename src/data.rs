/// Dataset iterators.
use crate::Device;
use crate::Tensor;

/// An iterator over a pair of tensors which have the same first dimension
/// size.
/// The typical use case is to iterate over batches. Each batch is a pair
/// containing a (potentially random) slice of each of the two input
/// tensors.
pub struct Iter2 {
    xs: Tensor,
    ys: Tensor,
    batch_index: i64,
    batch_size: i64,
    total_size: i64,
    device: Device,
    return_smaller_last_batch: bool,
}

impl Iter2 {
    /// Returns a new iterator.
    ///
    /// This takes as input two tensors which first dimension must match. The
    /// returned iterator can be used to range over mini-batches of data of
    /// specified size.
    ///
    /// # Arguments
    ///
    /// * `xs` - the features to be used by the model.
    /// * `ys` - the targets that the model attempts to predict.
    /// * `batch_size` - the size of batches to be returned.
    pub fn new(xs: &Tensor, ys: &Tensor, batch_size: i64) -> Iter2 {
        let total_size = xs.size()[0];
        if ys.size()[0] != total_size {
            panic!("different dimension for the two inputs {:?} {:?}", xs, ys)
        }
        Iter2 {
            xs: xs.shallow_clone(),
            ys: ys.shallow_clone(),
            batch_index: 0,
            batch_size,
            total_size,
            device: Device::Cpu,
            return_smaller_last_batch: false,
        }
    }

    /// Shuffles the dataset.
    ///
    /// The iterator would still run over the whole dataset but the order in
    /// which elements are grouped in mini-batches is randomized.
    pub fn shuffle(&mut self) -> &mut Iter2 {
        let index = Tensor::randperm(self.total_size, crate::kind::INT64_CPU);
        self.xs = self.xs.index_select(0, &index);
        self.ys = self.ys.index_select(0, &index);
        self
    }

    /// Transfers the mini-batches to a specified device.
    pub fn to_device(&mut self, device: Device) -> &mut Iter2 {
        self.device = device;
        self
    }

    /// When set, returns the last batch even if smaller than the batch size.
    pub fn return_smaller_last_batch(&mut self) -> &mut Iter2 {
        self.return_smaller_last_batch = true;
        self
    }
}

impl Iterator for Iter2 {
    type Item = (Tensor, Tensor);

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.batch_index * self.batch_size;
        let size = std::cmp::min(self.batch_size, self.total_size - start);
        if size <= 0 || (!self.return_smaller_last_batch && size < self.batch_size) {
            None
        } else {
            self.batch_index = self.batch_index + 1;
            Some((
                self.xs.narrow(0, start, size).to_device(self.device),
                self.ys.narrow(0, start, size).to_device(self.device),
            ))
        }
    }
}