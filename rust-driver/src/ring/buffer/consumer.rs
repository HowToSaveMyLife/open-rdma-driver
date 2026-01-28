use crate::ring::csr::ring_csr::ReaderOps;
use crate::ring::{
    csr::RingCsr,
    traits::{DeviceAdaptor, FromRingBytes, RingSpecToHost},
};
use std::{
    io,
    sync::atomic::{fence, Ordering},
};

use super::desc_ring::DmaBuffer;

// ============================================================================
// Consumer Ring (Card → Host)
// ============================================================================

/// Consumer-side ring buffer that reads descriptors produced by hardware.
///
/// # Type Parameters
/// Same as ProducerRing but for `RingSpecToHost` direction
///
/// # Synchronization Model
/// - **Hardware manages**: head (producer index, read via CSR)
/// - **Software manages**: `cached_tail` (consumer index)
/// - **Memory ordering**: Acquire fence after reading descriptors
///
/// # Empty vs Full Distinction
/// Uses same monotonic counter strategy as ProducerRing:
/// - **Empty**: `hw_head == cached_tail` (available = 0)
/// - **Full**: `hw_head - cached_tail == BUF_SIZE` (available = BUF_SIZE)
/// - Available elements: `hw_head.wrapping_sub(cached_tail)`
///
/// WARN: 读取的时候不会看 head ptr，而只是看 tail ptr 指向的 element 的标志位是否到达
pub(crate) struct ConsumerRing<Dev, Spec, const BUF_SIZE_EXP: u8>
where
    Dev: DeviceAdaptor,
    Spec: RingSpecToHost,
{
    /// DMA buffer for descriptors (stores bytes representation)
    buffer: DmaBuffer<<Spec::Element as FromRingBytes>::Bytes>,

    /// CSR ring handle for head/tail synchronization
    csr_ring: RingCsr<Dev, Spec>,

    /// Cached local tail (software consumer pointer)
    cached_tail: u32,

    /// Cached hardware head (hardware producer pointer)
    cached_hw_head: u32,
}

impl<Dev, Spec, const BUF_SIZE_EXP: u8> ConsumerRing<Dev, Spec, BUF_SIZE_EXP>
where
    Dev: DeviceAdaptor,
    Spec: RingSpecToHost,
    Spec::Element: FromRingBytes,
{
    const BUF_SIZE: u32 = 1 << BUF_SIZE_EXP;
    const BUF_SIZE_MASK: u32 = Self::BUF_SIZE - 1;
    /// Create a new consumer ring
    ///
    /// # Arguments
    /// * `buffer` - DMA buffer (must have capacity >= BUF_SIZE)
    /// * `csr_ring` - CSR ring handle for hardware synchronization
    ///
    /// # Errors
    /// Returns an error if CSR write fails
    ///
    /// # Panics
    /// Panics if buffer capacity less than BUF_SIZE
    pub(crate) fn new(
        buffer: DmaBuffer<<Spec::Element as FromRingBytes>::Bytes>,
        csr_ring: RingCsr<Dev, Spec>,
    ) -> io::Result<Self> {
        assert!(
            buffer.capacity() >= Self::BUF_SIZE,
            "buffer capacity mismatch"
        );

        // Write physical address to hardware CSR
        csr_ring.write_base_addr(buffer.phys_addr())?;

        log::debug!(
            "ConsumerRing: buffer write base addr with Sepc {} , pa=0x{:x}, capacity={}",
            std::any::type_name::<Spec>(),
            buffer.phys_addr(),
            buffer.capacity()
        );

        Ok(Self {
            buffer,
            csr_ring,
            cached_tail: 0,
            cached_hw_head: 0,
        })
    }

    /// Get number of available elements to consume
    pub(crate) fn available(&mut self) -> io::Result<usize> {
        let hw_head = self.csr_ring.read_head()?;
        self.cached_hw_head = hw_head;

        Ok(hw_head.wrapping_sub(self.cached_tail) as usize)
    }

    fn read_and_advance(&mut self) -> <Spec::Element as FromRingBytes>::Bytes {
        let index = self.tail() & Self::BUF_SIZE_MASK;
        let ret = self.buffer.read(index);
        self.buffer.zero(index);
        self.cached_tail = self.cached_tail.wrapping_add(1);
        ret
    }

    fn write_tail_csr(&mut self) -> io::Result<()> {
        self.csr_ring.write_tail(self.cached_tail)
    }

    fn read_head_csr(&mut self) -> io::Result<u32> {
        let hw_head = self.csr_ring.read_head()?;
        self.cached_hw_head = hw_head;
        Ok(hw_head)
    }

    /// Pop single element with validation
    ///
    /// # Returns
    /// - `Ok(Some(value))` if valid element available
    /// - `Ok(None)` if no elements or validation failed
    /// - `Err(_)` on CSR error
    pub(crate) fn try_pop(&mut self) -> io::Result<Option<Spec::Element>> {
        let idx_first = self.tail() & Self::BUF_SIZE_MASK;

        let first_element = self.buffer.read(idx_first);

        if Spec::Element::is_valid(&first_element) {
            if Spec::Element::has_next(&first_element) {
                let idx_next = idx_first.wrapping_add(1) & Self::BUF_SIZE_MASK;
                let second_element = self.buffer.read(idx_next);
                if Spec::Element::is_valid(&second_element) {
                    fence(Ordering::Acquire);
                    let a = self.read_and_advance();
                    let b = self.read_and_advance();
                    self.write_tail_csr()?;
                    Ok(Spec::Element::from_bytes(&[a, b]))
                } else {
                    Ok(None)
                }
            } else {
                fence(Ordering::Acquire);
                let a = self.read_and_advance();
                self.write_tail_csr()?;
                Ok(Spec::Element::from_bytes(&[a]))
            }
        } else {
            Ok(None)
        }
        // todo!()
        // if(<Spec as RingSpecToHost>::Element)
        // if()

        // let idx_next = idx_first.wrapping_add(1) & RING_BUF_LEN_MASK;
        // let value_first = self.read_index(idx_first);
        // let value_next = self.read_index(idx_next);

        // match (
        //     cond(&value_first),
        //     cond(&value_next),
        //     require_next(&value_first),
        // ) {
        //     (true, true, true) => {
        //         fence(Ordering::Acquire);
        //         let value_first = self.read_and_advance(idx_first);
        //         let value_next = self.read_and_advance(idx_next);
        //         (Some(value_first), Some(value_next))
        //     }
        //     (true, _, false) => {
        //         fence(Ordering::Acquire);
        //         let value_first = self.read_and_advance(idx_first);
        //         (Some(value_first), None)
        //     }
        //     (true, false, true) | (false, _, _) => (None, None),
        // }
    }

    /// Batch pop operation
    ///
    /// Pops up to `max_count` valid elements in a single operation.
    /// Stops at the first invalid descriptor.
    // pub(crate) fn pop_batch(&mut self, max_count: usize) -> io::Result<Vec<Spec::Bytes>> {
    //     let available = self.available()?;
    //     let count = available.min(max_count);
    //     let mut results = Vec::with_capacity(count);

    //     for _ in 0..count {
    //         let index = (self.cached_tail as usize) & Self::BUF_SIZE_MASK;
    //         let bytes = self.buffer.read(index);

    //         // Check if descriptor is valid
    //         if !Spec::Element::is_valid(&bytes) {
    //             break;
    //         }

    //         // Deserialize (always succeeds)
    //         let value = Spec::Element::from_bytes(bytes);
    //         results.push(value);
    //         self.buffer.zero(index);
    //         self.cached_tail = self.cached_tail.wrapping_add(1);
    //     }

    //     if !results.is_empty() {
    //         fence(Ordering::Acquire);
    //         self.csr_ring.write_tail(self.cached_tail)?;
    //     }

    //     Ok(results)
    // }

    pub(crate) fn tail(&self) -> u32 {
        self.cached_tail
    }

    pub(crate) fn cached_head(&self) -> u32 {
        self.cached_hw_head
    }
}
