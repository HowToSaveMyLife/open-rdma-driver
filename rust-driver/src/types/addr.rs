//! Type-safe address abstractions
//!
//! This module provides newtype wrappers for different address spaces used in the driver:
//! - `VirtAddr`: User-space virtual addresses (local process)
//! - `PhysAddr`: Physical addresses for DMA operations
//! - `RemoteAddr`: Remote virtual addresses for RDMA operations
//! - `CsrOffset`: Control and Status Register offsets
//! - `AlignedVirtAddr<N>`: Page-aligned virtual addresses (compile-time guarantee)
//! - `AlignedPhysAddr<N>`: Page-aligned physical addresses (compile-time guarantee)
//!
//! These types prevent accidental mixing of address spaces at compile time,
//! which could lead to serious hardware errors like DMA writing to wrong addresses,
//! using local addresses in RDMA operations, or invalid register accesses.

use std::fmt;

use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Virtual address in user-space memory
///
/// Represents a pointer to virtual memory that may be passed by applications
/// through ibverbs API. These addresses must be translated to physical addresses
/// before being used in DMA operations.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
#[repr(transparent)]
pub(crate) struct VirtAddr(u64);

impl VirtAddr {
    /// Creates a new virtual address
    #[inline]
    pub(crate) const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Returns the raw address value
    #[inline]
    pub(crate) const fn as_u64(self) -> u64 {
        self.0
    }

    /// Creates a virtual address from a raw pointer
    #[inline]
    #[allow(clippy::as_conversions)]
    pub(crate) fn from_ptr<T>(ptr: *const T) -> Self {
        Self(ptr as u64)
    }

    /// Converts to a raw pointer
    #[inline]
    #[allow(clippy::as_conversions)]
    pub(crate) fn as_ptr<T>(self) -> *const T {
        self.0 as *const T
    }

    /// Converts to a mutable raw pointer
    #[inline]
    #[allow(clippy::as_conversions)]
    pub(crate) fn as_mut_ptr<T>(self) -> *mut T {
        self.0 as *mut T
    }

    /// Adds an offset to the address
    #[inline]
    pub(crate) fn offset(self, offset: u64) -> Option<Self> {
        self.0.checked_add(offset).map(Self)
    }

    /// Checks if the address is aligned to the given alignment
    #[inline]
    pub(crate) const fn is_aligned_to(self, align: u64) -> bool {
        self.0 % align == 0
    }

    /// align_down to 2^N bytes
    #[inline]
    pub(crate) const fn to_alignd<const N: u8>(self) -> AlignedVirtAddr<N> {
        AlignedVirtAddr::align_down(self)
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VirtAddr(0x{:x})", self.0)
    }
}

impl fmt::LowerHex for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

/// Physical address for DMA operations
///
/// Represents a physical memory address that can be used by hardware for DMA.
/// These addresses are obtained by translating virtual addresses through
/// the `AddressResolver` trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub(crate) struct PhysAddr(u64);

impl PhysAddr {
    /// Creates a new physical address
    #[inline]
    pub(crate) const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Returns the raw address value
    #[inline]
    pub(crate) const fn as_u64(self) -> u64 {
        self.0
    }

    /// Adds an offset to the address
    #[inline]
    pub(crate) fn offset(self, offset: u64) -> Option<Self> {
        self.0.checked_add(offset).map(Self)
    }

    /// Checks if the address is aligned to the given alignment
    #[inline]
    pub(crate) const fn is_aligned_to(self, align: u64) -> bool {
        self.0 % align == 0
    }

    /// Splits a 64-bit physical address into low and high 32-bit parts
    ///
    /// This is useful for writing to hardware registers that accept
    /// 64-bit addresses as two 32-bit values.
    #[inline]
    #[allow(clippy::as_conversions)]
    pub(crate) fn split(self) -> (u32, u32) {
        let lo = (self.0 & 0xFFFF_FFFF) as u32;
        let hi = (self.0 >> 32) as u32;
        (lo, hi)
    }

    /// Combines low and high 32-bit parts into a 64-bit physical address
    #[inline]
    pub(crate) fn from_parts(lo: u32, hi: u32) -> Self {
        Self(u64::from(lo) | (u64::from(hi) << 32))
    }

    /// align_down to 2^N bytes
    #[inline]
    pub(crate) const fn to_alignd<const N: u8>(self) -> AlignedPhysAddr<N> {
        AlignedPhysAddr::align_down(self)
    }
}

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PhysAddr(0x{:x})", self.0)
    }
}

impl fmt::LowerHex for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

/// Remote virtual address (RDMA target address)
///
/// Represents a virtual address in a remote machine's address space.
/// This is opaque to the local driver and is used as the target address
/// for RDMA Write/Read/Atomic operations. It cannot be dereferenced locally
/// as it exists in a different process's (often on a different machine)
/// virtual address space.
///
/// # Note
/// Unlike `VirtAddr`, `RemoteAddr` does not provide `as_ptr()` methods
/// since the address cannot be accessed locally.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
    Encode,
    Decode,
)]
#[repr(transparent)]
pub(crate) struct RemoteAddr(u64);

impl RemoteAddr {
    /// Creates a new remote address
    #[inline]
    pub(crate) const fn new(addr: u64) -> Self {
        Self(addr)
    }

    /// Returns the raw address value
    ///
    /// This is used for serialization into RDMA descriptors that will
    /// be sent over the network to the remote side.
    #[inline]
    pub(crate) const fn as_u64(self) -> u64 {
        self.0
    }

    /// Adds an offset to the remote address
    ///
    /// This can be used for calculating offsets within a remote memory region,
    /// though typically offset calculations should be done on the remote side.
    #[inline]
    pub(crate) fn offset(self, offset: u64) -> Option<Self> {
        self.0.checked_add(offset).map(Self)
    }

    /// Checks if the address is aligned to the given alignment
    #[inline]
    pub(crate) const fn is_aligned_to(self, align: u64) -> bool {
        self.0 % align == 0
    }
}

impl fmt::Display for RemoteAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RemoteAddr(0x{:x})", self.0)
    }
}

impl fmt::LowerHex for RemoteAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for RemoteAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

/// Control and Status Register offset
///
/// Represents an offset within the CSR address space. This type ensures
/// that CSR offsets cannot be accidentally used as memory addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct CsrOffset(usize);

impl CsrOffset {
    /// Creates a new CSR offset
    #[inline]
    pub(crate) const fn new(offset: usize) -> Self {
        Self(offset)
    }

    /// Returns the raw offset value
    #[inline]
    pub(crate) const fn as_usize(self) -> usize {
        self.0
    }

    /// Adds an offset to the CSR offset
    #[inline]
    pub(crate) fn offset(self, offset: usize) -> Option<Self> {
        self.0.checked_add(offset).map(Self)
    }
}

impl fmt::Display for CsrOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CsrOffset(0x{:x})", self.0)
    }
}

impl fmt::LowerHex for CsrOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::UpperHex for CsrOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0, f)
    }
}

// ============================================================================
// Aligned Address Types (Compile-time Alignment Guarantee)
// ============================================================================

/// Aligned virtual address with compile-time alignment guarantee
///
/// The generic parameter `N` specifies the alignment in **bits** (not bytes).
/// The actual alignment is `2^N` bytes.
///
/// # Examples
///
/// ```
/// use types::addr::{VirtAddr, AlignedVirtAddr};
///
/// // Create a 4KB-aligned virtual address (2^12 = 4096 bytes)
/// let addr = VirtAddr::new(0x1000);
/// let aligned = AlignedVirtAddr::<12>::new_checked(addr)?;
///
/// // Type-safe: compiler knows this address is 4KB-aligned
/// assert_eq!(aligned.as_u64(), 0x1000);
/// ```
///
/// # Type Safety
///
/// This type guarantees at compile time that:
/// - The address is aligned to `2^N` bytes
/// - Only 2^N alignments are allowed (enforced by bit shift)
/// - The alignment bits N must be <= 63 (max address space)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub(crate) struct AlignedVirtAddr<const N: u8>(VirtAddr);

impl<const N: u8> AlignedVirtAddr<N> {
    // Compile-time check: N must be valid (0 < N <= 63)
    const _ALIGNMENT_RANGE_CHECK: () =
        assert!(N > 0 && N <= 63, "Alignment bits must be in range 1..=63");

    /// Alignment in bytes (2^N)
    pub(crate) const ALIGNMENT_BYTES: u64 = 1u64 << N;

    /// Alignment mask for fast alignment operations
    const ALIGNMENT_MASK: u64 = Self::ALIGNMENT_BYTES - 1;

    /// Creates an aligned virtual address, checking alignment at runtime
    ///
    /// Returns `None` if the address is not aligned to `2^N` bytes.
    #[inline]
    pub(crate) const fn new_checked(addr: VirtAddr) -> Option<Self> {
        if addr.is_aligned_to(Self::ALIGNMENT_BYTES) {
            Some(Self(addr))
        } else {
            None
        }
    }

    /// Creates an aligned virtual address from a raw u64 value
    #[inline]
    pub(crate) const fn from_u64(val: u64) -> Option<Self> {
        Self::new_checked(VirtAddr::new(val))
    }

    /// Creates an aligned virtual address from a pointer
    #[inline]
    pub(crate) fn from_ptr<T>(ptr: *const T) -> Option<Self> {
        Self::new_checked(VirtAddr::from_ptr(ptr))
    }

    /// Creates an aligned virtual address without checking alignment
    ///
    /// # Safety
    ///
    /// The caller must ensure that `addr` is aligned to `2^N` bytes.
    /// Violating this may lead to undefined behavior in downstream code
    /// that relies on the alignment guarantee.
    #[inline]
    #[allow(unsafe_code)]
    pub(crate) const unsafe fn new_unchecked(addr: VirtAddr) -> Self {
        // Note: debug_assert! cannot be used in const fn
        // Callers must ensure alignment manually
        Self(addr)
    }

    /// Aligns down to the nearest aligned address
    ///
    /// This always succeeds and returns a valid aligned address.
    #[inline]
    pub(crate) const fn align_down(addr: VirtAddr) -> Self {
        let raw = addr.as_u64();
        let aligned = raw & !Self::ALIGNMENT_MASK;
        Self(VirtAddr::new(aligned))
    }

    /// Attempts to align up to the nearest aligned address
    ///
    /// Returns `None` if alignment would cause overflow.
    #[inline]
    pub(crate) const fn align_up(addr: VirtAddr) -> Option<Self> {
        let raw = addr.as_u64();
        let aligned = match raw.checked_add(Self::ALIGNMENT_MASK) {
            Some(val) => val & !Self::ALIGNMENT_MASK,
            None => return None,
        };
        Some(Self(VirtAddr::new(aligned)))
    }

    /// Returns the underlying unaligned virtual address
    #[inline]
    pub(crate) const fn into_inner(self) -> VirtAddr {
        self.0
    }

    /// Returns the raw address value
    #[inline]
    pub(crate) const fn as_u64(self) -> u64 {
        self.0.as_u64()
    }

    /// Converts to a raw pointer
    #[inline]
    pub(crate) fn as_ptr<T>(self) -> *const T {
        self.0.as_ptr()
    }

    /// Converts to a mutable raw pointer
    #[inline]
    pub(crate) fn as_mut_ptr<T>(self) -> *mut T {
        self.0.as_mut_ptr()
    }

    /// Adds an aligned offset, maintaining alignment guarantee
    ///
    /// The offset must be a multiple of `2^N` bytes to maintain alignment.
    /// Returns `None` if overflow occurs.
    #[inline]
    #[allow(unsafe_code)]
    pub(crate) fn offset_aligned(self, offset: u64) -> Option<Self> {
        debug_assert!(
            offset % Self::ALIGNMENT_BYTES == 0,
            "Offset 0x{:x} is not aligned to {} bytes",
            offset,
            Self::ALIGNMENT_BYTES
        );
        let new_addr = self.0.offset(offset)?;
        // SAFETY: Both base and offset are aligned to 2^N
        Some(unsafe { Self::new_unchecked(new_addr) })
    }

    /// Adds an unaligned offset, losing alignment guarantee
    ///
    /// Returns an unaligned `VirtAddr` since the result may not be aligned.
    #[inline]
    pub(crate) fn offset(self, offset: u64) -> Option<VirtAddr> {
        self.0.offset(offset)
    }
}

impl<const N: u8> fmt::Display for AlignedVirtAddr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AlignedVirtAddr<{}>(0x{:x})", N, self.0.as_u64())
    }
}

impl<const N: u8> fmt::LowerHex for AlignedVirtAddr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0 .0, f)
    }
}

impl<const N: u8> fmt::UpperHex for AlignedVirtAddr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0 .0, f)
    }
}

/// Aligned physical address with compile-time alignment guarantee
///
/// The generic parameter `N` specifies the alignment in **bits** (not bytes).
/// The actual alignment is `2^N` bytes.
///
/// # Examples
///
/// ```
/// use types::addr::{PhysAddr, AlignedPhysAddr};
///
/// // Create a 2MB-aligned physical address (2^21 = 2097152 bytes)
/// let addr = PhysAddr::new(0x20_0000);
/// let aligned = AlignedPhysAddr::<21>::new_checked(addr)?;
///
/// // Split for hardware register writes
/// let (lo, hi) = aligned.split();
/// ```
///
/// # Type Safety
///
/// This type is critical for:
/// - DMA buffer allocation (requires page alignment)
/// - Hardware register writes (CSR base addresses)
/// - Memory translation table (MTT/PGT) entries
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
pub(crate) struct AlignedPhysAddr<const N: u8>(PhysAddr);

impl<const N: u8> AlignedPhysAddr<N> {
    const _ALIGNMENT_RANGE_CHECK: () =
        assert!(N > 0 && N <= 63, "Alignment bits must be in range 1..=63");

    pub(crate) const ALIGNMENT_BYTES: u64 = 1u64 << N;
    const ALIGNMENT_MASK: u64 = Self::ALIGNMENT_BYTES - 1;

    #[inline]
    pub(crate) const fn new_checked(addr: PhysAddr) -> Option<Self> {
        if addr.is_aligned_to(Self::ALIGNMENT_BYTES) {
            Some(Self(addr))
        } else {
            None
        }
    }

    #[inline]
    pub(crate) const fn from_u64(val: u64) -> Option<Self> {
        Self::new_checked(PhysAddr::new(val))
    }

    #[inline]
    #[allow(unsafe_code)]
    pub(crate) const unsafe fn new_unchecked(addr: PhysAddr) -> Self {
        // Note: debug_assert! cannot be used in const fn
        // Callers must ensure alignment manually
        Self(addr)
    }

    #[inline]
    pub(crate) const fn align_down(addr: PhysAddr) -> Self {
        let raw = addr.as_u64();
        let aligned = raw & !Self::ALIGNMENT_MASK;
        Self(PhysAddr::new(aligned))
    }

    #[inline]
    pub(crate) const fn align_up(addr: PhysAddr) -> Option<Self> {
        let raw = addr.as_u64();
        let aligned = match raw.checked_add(Self::ALIGNMENT_MASK) {
            Some(val) => val & !Self::ALIGNMENT_MASK,
            None => return None,
        };
        Some(Self(PhysAddr::new(aligned)))
    }

    #[inline]
    pub(crate) const fn into_inner(self) -> PhysAddr {
        self.0
    }

    #[inline]
    pub(crate) const fn as_u64(self) -> u64 {
        self.0.as_u64()
    }

    /// Splits into low and high 32-bit parts (for CSR register writes)
    #[inline]
    pub(crate) fn split(self) -> (u32, u32) {
        self.0.split()
    }

    #[inline]
    #[allow(unsafe_code)]
    pub(crate) fn offset_aligned(self, offset: u64) -> Option<Self> {
        debug_assert!(
            offset % Self::ALIGNMENT_BYTES == 0,
            "Offset 0x{:x} is not aligned to {} bytes",
            offset,
            Self::ALIGNMENT_BYTES
        );
        let new_addr = self.0.offset(offset)?;
        Some(unsafe { Self::new_unchecked(new_addr) })
    }

    #[inline]
    pub(crate) fn offset(self, offset: u64) -> Option<PhysAddr> {
        self.0.offset(offset)
    }
}

impl<const N: u8> fmt::Display for AlignedPhysAddr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AlignedPhysAddr<{}>(0x{:x})", N, self.0.as_u64())
    }
}

impl<const N: u8> fmt::LowerHex for AlignedPhysAddr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0 .0, f)
    }
}

impl<const N: u8> fmt::UpperHex for AlignedPhysAddr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::UpperHex::fmt(&self.0 .0, f)
    }
}

// ============================================================================
// Type Aliases for Common Alignments
// ============================================================================

// Page-aligned addresses (depends on feature flag)
#[cfg(feature = "page_size_4k")]
pub(crate) type PageAlignedVirtAddr = AlignedVirtAddr<12>; // 2^12 = 4096 bytes
#[cfg(feature = "page_size_4k")]
pub(crate) type PageAlignedPhysAddr = AlignedPhysAddr<12>;

#[cfg(feature = "page_size_2m")]
pub(crate) type PageAlignedVirtAddr = AlignedVirtAddr<21>; // 2^21 = 2MB
#[cfg(feature = "page_size_2m")]
pub(crate) type PageAlignedPhysAddr = AlignedPhysAddr<21>;

// Explicit size aliases (independent of feature flags)
pub(crate) type HugePageAlignedVirtAddr = AlignedVirtAddr<21>; // 2MB
pub(crate) type HugePageAlignedPhysAddr = AlignedPhysAddr<21>;

pub(crate) type GigaPageAlignedVirtAddr = AlignedVirtAddr<30>; // 1GB
pub(crate) type GigaPageAlignedPhysAddr = AlignedPhysAddr<30>;

pub(crate) type Aligned4KVirtAddr = AlignedVirtAddr<12>; // 4KB
pub(crate) type Aligned4KPhysAddr = AlignedPhysAddr<12>;

// Cache line alignment (common for DMA descriptors)
pub(crate) type CacheAlignedVirtAddr = AlignedVirtAddr<6>; // 2^6 = 64 bytes
pub(crate) type CacheAlignedPhysAddr = AlignedPhysAddr<6>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virt_addr_basic() {
        let addr = VirtAddr::new(0x1000);
        assert_eq!(addr.as_u64(), 0x1000);
    }

    #[test]
    fn test_virt_addr_alignment() {
        let addr = VirtAddr::new(0x1000);
        assert!(addr.is_aligned_to(0x1000));
        assert!(addr.is_aligned_to(0x100));
        assert!(!addr.is_aligned_to(0x2000));
    }

    #[test]
    fn test_phys_addr_split() {
        let addr = PhysAddr::new(0x1234_5678_9ABC_DEF0);
        let (lo, hi) = addr.split();
        assert_eq!(lo, 0x9ABC_DEF0);
        assert_eq!(hi, 0x1234_5678);
        assert_eq!(PhysAddr::from_parts(lo, hi), addr);
    }

    #[test]
    fn test_offset() {
        let virt = VirtAddr::new(0x1000);
        assert_eq!(virt.offset(0x100), Some(VirtAddr::new(0x1100)));

        let phys = PhysAddr::new(0x2000);
        assert_eq!(phys.offset(0x200), Some(PhysAddr::new(0x2200)));

        let csr = CsrOffset::new(0x100);
        assert_eq!(csr.offset(0x10), Some(CsrOffset::new(0x110)));
    }

    #[test]
    fn test_offset_overflow() {
        let virt = VirtAddr::new(u64::MAX);
        assert_eq!(virt.offset(1), None);

        let csr = CsrOffset::new(usize::MAX);
        assert_eq!(csr.offset(1), None);
    }

    #[test]
    fn test_remote_addr_basic() {
        let addr = RemoteAddr::new(0x7000_0000);
        assert_eq!(addr.as_u64(), 0x7000_0000);
    }

    #[test]
    fn test_remote_addr_offset() {
        let addr = RemoteAddr::new(0x1000);
        assert_eq!(addr.offset(0x500), Some(RemoteAddr::new(0x1500)));

        let addr_max = RemoteAddr::new(u64::MAX);
        assert_eq!(addr_max.offset(1), None);
    }

    #[test]
    fn test_remote_addr_alignment() {
        let addr = RemoteAddr::new(0x4000);
        assert!(addr.is_aligned_to(0x1000));
        assert!(addr.is_aligned_to(0x4000));
        assert!(!addr.is_aligned_to(0x8000));
    }

    // ========================================================================
    // Aligned Address Type Tests
    // ========================================================================

    #[test]
    fn test_aligned_virt_addr_new_checked() {
        // 4KB aligned (2^12 = 4096)
        type Aligned4K = AlignedVirtAddr<12>;

        // Aligned address should succeed
        let addr = VirtAddr::new(0x1000);
        assert!(Aligned4K::new_checked(addr).is_some());

        // Unaligned address should fail
        let addr = VirtAddr::new(0x1001);
        assert!(Aligned4K::new_checked(addr).is_none());

        // Zero is aligned to everything
        let addr = VirtAddr::new(0);
        assert!(Aligned4K::new_checked(addr).is_some());
    }

    #[test]
    fn test_aligned_phys_addr_new_checked() {
        // 2MB aligned (2^21)
        type Aligned2M = AlignedPhysAddr<21>;

        let addr = PhysAddr::new(0x20_0000);
        assert!(Aligned2M::new_checked(addr).is_some());

        let addr = PhysAddr::new(0x20_0001);
        assert!(Aligned2M::new_checked(addr).is_none());
    }

    #[test]
    fn test_aligned_addr_align_down() {
        type Aligned4K = AlignedVirtAddr<12>;

        // Already aligned - no change
        let addr = VirtAddr::new(0x1000);
        let aligned = Aligned4K::align_down(addr);
        assert_eq!(aligned.as_u64(), 0x1000);

        // Unaligned - round down
        let addr = VirtAddr::new(0x1234);
        let aligned = Aligned4K::align_down(addr);
        assert_eq!(aligned.as_u64(), 0x1000);

        let addr = VirtAddr::new(0x1FFF);
        let aligned = Aligned4K::align_down(addr);
        assert_eq!(aligned.as_u64(), 0x1000);
    }

    #[test]
    fn test_aligned_addr_align_up() {
        type Aligned4K = AlignedVirtAddr<12>;

        // Already aligned - no change
        let addr = VirtAddr::new(0x1000);
        let aligned = Aligned4K::align_up(addr).unwrap();
        assert_eq!(aligned.as_u64(), 0x1000);

        // Unaligned - round up
        let addr = VirtAddr::new(0x1001);
        let aligned = Aligned4K::align_up(addr).unwrap();
        assert_eq!(aligned.as_u64(), 0x2000);

        let addr = VirtAddr::new(0x1FFF);
        let aligned = Aligned4K::align_up(addr).unwrap();
        assert_eq!(aligned.as_u64(), 0x2000);

        // Overflow check
        let addr = VirtAddr::new(u64::MAX);
        assert!(Aligned4K::align_up(addr).is_none());
    }

    #[test]
    fn test_aligned_addr_conversions() {
        type Aligned4K = AlignedVirtAddr<12>;

        let addr = VirtAddr::new(0x1000);
        let aligned = Aligned4K::new_checked(addr).unwrap();

        // Test conversions
        assert_eq!(aligned.as_u64(), 0x1000);
        assert_eq!(aligned.into_inner(), addr);

        // Test from_u64
        let aligned2 = Aligned4K::from_u64(0x2000).unwrap();
        assert_eq!(aligned2.as_u64(), 0x2000);

        assert!(Aligned4K::from_u64(0x2001).is_none());
    }

    #[test]
    fn test_aligned_phys_addr_split() {
        type Aligned2M = AlignedPhysAddr<21>;

        let addr = PhysAddr::new(0x1234_5678_0020_0000);
        let aligned = Aligned2M::new_checked(addr).unwrap();

        let (lo, hi) = aligned.split();
        assert_eq!(lo, 0x0020_0000);
        assert_eq!(hi, 0x1234_5678);
    }

    #[test]
    fn test_aligned_addr_offset_aligned() {
        type Aligned4K = AlignedVirtAddr<12>;

        let base = Aligned4K::from_u64(0x1000).unwrap();

        // Aligned offset maintains alignment
        let next = base.offset_aligned(0x1000).unwrap();
        assert_eq!(next.as_u64(), 0x2000);

        // Multiple pages
        let next = base.offset_aligned(0x3000).unwrap();
        assert_eq!(next.as_u64(), 0x4000);

        // Overflow check
        let base = Aligned4K::from_u64(u64::MAX - 0xFFF).unwrap();
        assert!(base.offset_aligned(0x1000).is_none());
    }

    #[test]
    #[should_panic(expected = "is not aligned")]
    #[cfg(debug_assertions)]
    fn test_aligned_addr_offset_aligned_unaligned_panic() {
        type Aligned4K = AlignedVirtAddr<12>;
        let base = Aligned4K::from_u64(0x1000).unwrap();
        // Unaligned offset should panic in debug mode
        let _ = base.offset_aligned(0x1001);
    }

    #[test]
    fn test_aligned_addr_offset_unaligned() {
        type Aligned4K = AlignedVirtAddr<12>;

        let base = Aligned4K::from_u64(0x1000).unwrap();

        // Unaligned offset returns VirtAddr (not aligned)
        let result = base.offset(0x100).unwrap();
        assert_eq!(result.as_u64(), 0x1100);
        assert!(!result.is_aligned_to(0x1000));
    }

    #[test]
    fn test_page_aligned_type_aliases() {
        // Test that type aliases compile and work correctly
        #[cfg(feature = "page_size_4k")]
        {
            let addr = PageAlignedVirtAddr::from_u64(0x1000).unwrap();
            assert_eq!(addr.as_u64(), 0x1000);
            assert_eq!(PageAlignedVirtAddr::ALIGNMENT_BYTES, 4096);
        }

        #[cfg(feature = "page_size_2m")]
        {
            let addr = PageAlignedVirtAddr::from_u64(0x20_0000).unwrap();
            assert_eq!(addr.as_u64(), 0x20_0000);
            assert_eq!(PageAlignedVirtAddr::ALIGNMENT_BYTES, 2 * 1024 * 1024);
        }

        // Explicit aliases always work
        let huge = HugePageAlignedVirtAddr::from_u64(0x20_0000).unwrap();
        assert_eq!(huge.as_u64(), 0x20_0000);
        assert_eq!(HugePageAlignedVirtAddr::ALIGNMENT_BYTES, 2 * 1024 * 1024);
    }

    #[test]
    fn test_cache_aligned_addresses() {
        // 64-byte cache line alignment (2^6)
        let addr = CacheAlignedVirtAddr::from_u64(0x40).unwrap();
        assert_eq!(addr.as_u64(), 0x40);
        assert_eq!(CacheAlignedVirtAddr::ALIGNMENT_BYTES, 64);

        // Unaligned to cache line
        assert!(CacheAlignedVirtAddr::from_u64(0x41).is_none());
    }

    #[test]
    fn test_different_alignment_levels() {
        // Test that different alignment levels are distinct types
        let addr4k = Aligned4KVirtAddr::from_u64(0x1000).unwrap();
        let addr2m = HugePageAlignedVirtAddr::from_u64(0x20_0000).unwrap();
        let addr1g = GigaPageAlignedVirtAddr::from_u64(0x4000_0000).unwrap();

        assert_eq!(addr4k.as_u64(), 0x1000);
        assert_eq!(addr2m.as_u64(), 0x20_0000);
        assert_eq!(addr1g.as_u64(), 0x4000_0000);

        assert_eq!(Aligned4KVirtAddr::ALIGNMENT_BYTES, 4096);
        assert_eq!(HugePageAlignedVirtAddr::ALIGNMENT_BYTES, 2 * 1024 * 1024);
        assert_eq!(GigaPageAlignedVirtAddr::ALIGNMENT_BYTES, 1024 * 1024 * 1024);
    }

    #[test]
    fn test_aligned_addr_display() {
        type Aligned4K = AlignedVirtAddr<12>;
        let addr = Aligned4K::from_u64(0x1000).unwrap();
        let s = format!("{}", addr);
        assert!(s.contains("AlignedVirtAddr"));
        assert!(s.contains("12"));
        assert!(s.contains("0x1000"));
    }
}
