//! Type-safe abstractions for hardware driver
//!
//! This module provides newtype wrappers and type-safe interfaces
//! for various hardware concepts used throughout the driver.

pub(crate) mod addr;

pub(crate) use addr::{
    AlignedPhysAddr, AlignedVirtAddr, CacheAlignedPhysAddr, CacheAlignedVirtAddr, CsrOffset,
    GigaPageAlignedPhysAddr, GigaPageAlignedVirtAddr, HugePageAlignedPhysAddr,
    HugePageAlignedVirtAddr, PageAlignedPhysAddr, PageAlignedVirtAddr, PhysAddr, RemoteAddr,
    VirtAddr,
};
