
use crate::{
    descriptors::simple_nic::{SimpleNicRxQueueDesc, SimpleNicTxQueueDesc},
    ringbuf::DescRingBuffer,
};

/// A transmit queue for the simple NIC device.
pub(crate) struct SimpleNicTxQueue {
    /// Inner queue
    inner: DescRingBuffer,
}

impl SimpleNicTxQueue {
    pub(crate) fn new(inner: DescRingBuffer) -> Self {
        Self { inner }
    }

    pub(crate) fn push(&mut self, desc: SimpleNicTxQueueDesc) -> bool {
        self.inner.push(&desc)
    }

    pub(crate) fn head(&self) -> u32 {
        self.inner.head() as u32
    }

    pub(crate) fn set_tail(&mut self, tail: u32) {
        self.inner.set_tail(tail);
    }

    pub(crate) fn remaining(&self) -> usize {
        self.inner.remaining()
    }
}

/// A receive queue for the simple NIC device.
pub(crate) struct SimpleNicRxQueue {
    /// Inner queue
    inner: DescRingBuffer,
}

impl SimpleNicRxQueue {
    pub(crate) fn new(inner: DescRingBuffer) -> Self {
        Self { inner }
    }

    pub(crate) fn pop(&mut self) -> Option<SimpleNicRxQueueDesc> {
        self.inner.pop()
    }
}
