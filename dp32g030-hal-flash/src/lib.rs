#![no_std]

//! Acccess DP32G030 Flash peripheral via code in RAM.
//!
//! This crate contains a tiny, relocatable code chunk that can be
//! loaded into RAM to talk to the DP32G030 Flash peripheral.

use core::cell::UnsafeCell;

// helper to use include_bytes! to define an array (not a slice)
macro_rules! define_included_bytes {
    ($name:ident, $path:expr) => {
        const $name: [u8; include_bytes!($path).len()] = *include_bytes!($path);
    };
}

// include our code and recover the Header at the start.
define_included_bytes!(CODE, concat!(env!("OUT_DIR"), "/dp32g030-hal-flash.bin"));
const HEADER: Header = Header::from_code();

/// An entry in the [Header] pointing to a function.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HeaderEntry<F>(*const core::marker::PhantomData<F>);

impl<F> HeaderEntry<F> {
    /// Construct an entry from an offset into the [Code::data()].
    pub const fn from_offset(addr: usize) -> Self {
        Self(addr as *const core::marker::PhantomData<F>)
    }

    /// Get the offset of this function inside the [Code::data()].
    pub fn as_offset(&self) -> usize {
        self.0 as usize
    }
}

/// A header describing each function in our [Code].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Header {
    // note: if you re-order these structs, you must also re-order
    // and update offsets in Header::from_code().
    pub init: HeaderEntry<fn(u8)>,
    pub read_nvr: HeaderEntry<fn(u16, &mut [u8])>,
    pub erase: HeaderEntry<unsafe fn(*mut u32)>,
    pub program_word: HeaderEntry<unsafe fn(u32, *mut u32)>,
    pub program: HeaderEntry<unsafe fn(&[u32], *mut u32) -> bool>,
    pub read_nvr_apb: HeaderEntry<fn(u16) -> u32>,
}

// we never write to Header
unsafe impl Sync for Header {}

impl Header {
    // build a header from the included raw rode.
    const fn from_code() -> Self {
        Self {
            init: HeaderEntry::from_offset(Self::read_u32le(0) as usize),
            read_nvr: HeaderEntry::from_offset(Self::read_u32le(1) as usize),
            erase: HeaderEntry::from_offset(Self::read_u32le(2) as usize),
            program_word: HeaderEntry::from_offset(Self::read_u32le(3) as usize),
            program: HeaderEntry::from_offset(Self::read_u32le(4) as usize),
            read_nvr_apb: HeaderEntry::from_offset(Self::read_u32le(5) as usize),
        }
    }

    // dummy read for when CODE is not yet compiled
    #[cfg(feature = "intern-compile")]
    const fn read_u32le(_offset_words: usize) -> u32 {
        0
    }

    // real read for when CODE is compiled
    #[cfg(not(feature = "intern-compile"))]
    const fn read_u32le(offset_words: usize) -> u32 {
        let offset = offset_words * core::mem::size_of::<u32>();
        let val = CODE[offset] as u32
            | ((CODE[offset + 1] as u32) << 8)
            | ((CODE[offset + 2] as u32) << 16)
            | ((CODE[offset + 3] as u32) << 24);

        // two bytes is the smallest possible function
        assert!((val as usize) < CODE.len() - 2);
        val
    }
}

/// A chunk of code that lives in RAM and talks to the flash peripheral.
#[derive(Debug)]
#[repr(C, align(4))]
pub struct Code {
    // we don't use interior mutability, but we *do* want the linker
    // to put us somewhere where mutation is possible, i.e. not flash
    data: UnsafeCell<[u8; CODE.len()]>,
}

// we never write to Code itself, and writes to the flash peripheral
// are guarded by a critical section.
unsafe impl Sync for Code {}

impl Code {
    /// Create a new code chunk.
    ///
    /// This must live somewhere other than flash. Rust is smart
    /// enough to put it in an area of memory you can write to, which
    /// means "not flash". If you override linker sections here, make
    /// sure you don't accidentally put it in flash!
    ///
    /// Ultimately, this can be a static, on the stack, or on the heap.
    pub const fn new() -> Self {
        Self {
            data: UnsafeCell::new(CODE),
        }
    }

    /// Get the raw data representing this code.
    pub const fn data(&self) -> &[u8] {
        unsafe { (*self.data.get().cast_const()).as_slice() }
    }

    /// Get the header describing this code.
    pub const fn header(&self) -> &Header {
        &HEADER
    }
}

// if we share the same target as the blobs, add some helper functions to
// use real bona-fide function pointers
#[cfg(all(target_arch = "arm", target_os = "none"))]
mod same_target {
    use super::*;

    impl<F> HeaderEntry<F> {
        /// Create a header entry from a function pointer.
        ///
        /// # Safety
        ///
        /// This assumes all code starts at address 0x0000,
        /// and this does not check the function pointer's type.
        ///
        /// For a safer interface, please use [header!].
        #[cfg(feature = "intern-compile")]
        pub const unsafe fn from_function_unchecked(ptr: *const ()) -> Self {
            Self(ptr as *const core::marker::PhantomData<F>)
        }

        /// Check a function pointer against this entry's type.
        ///
        /// # Safety
        ///
        /// This assumes `F` is a function pointer type -- or at
        /// least, a type that does not implement Drop.
        ///
        /// For a safer interface, please use [header!].
        #[cfg(feature = "intern-compile")]
        pub const unsafe fn check(self, f: F) -> Self {
            core::mem::forget(f);
            self
        }

        /// Combine this entry with the raw code slice to produce a function pointer.
        ///
        /// # Safety
        ///
        /// For this to work, `&base[self.as_offset()]` must point to
        /// a valid function of type `F`, using the same rust ABI as
        /// your current code.
        pub unsafe fn as_function(&self, base: &[u8]) -> F {
            let p = ((base.as_ptr() as usize) + self.as_offset()) as *const ();
            core::mem::transmute_copy::<*const (), F>(&p)
        }
    }

    /// Construct a header describing the raw code slice.
    ///
    /// This is used inside the binary containing the code compiled
    /// into the slice, and produces a `HEADER` symbol in section
    /// `.header`. For this to work, the entire binary must be
    /// compiled with a 0x0000 base address.
    #[cfg(feature = "intern-compile")]
    #[macro_export]
    macro_rules! header {
        {$($field:ident $(: $value:expr)?),*$(,)?} => {
            #[link_section = ".header"]
            #[no_mangle]
            static HEADER: $crate::Header = unsafe {
                $crate::Header {
                    $( $field: $crate::header!(@value, $field $(: $value)?), )*
                }
            };
        };

        (@value, $field:ident) => {
            $crate::header!(@value, $field : $field)
        };

        (@value, $field:ident : $value:expr) => {
            unsafe { $crate::HeaderEntry::from_function_unchecked($value as *const ()).check($value) }
        };
    }

    impl Code {
        // resolve a header entry into a function pointer
        unsafe fn resolve<F>(&self, f: &HeaderEntry<F>) -> F {
            f.as_function(self.data())
        }

        pub fn init(&self, clock_mhz: u8) {
            unsafe { self.resolve(&HEADER.init)(clock_mhz) }
        }

        pub fn read_nvr(&self, src: u16, dest: &mut [u8]) {
            unsafe { self.resolve(&HEADER.read_nvr)(src, dest) }
        }

        pub unsafe fn erase(&self, sector: *mut u32) {
            self.resolve(&HEADER.erase)(sector)
        }

        pub unsafe fn program_word(&self, word: u32, dest: *mut u32) {
            self.resolve(&HEADER.program_word)(word, dest)
        }

        pub unsafe fn program(&self, src: &[u32], dest: *mut u32) -> bool {
            self.resolve(&HEADER.program)(src, dest)
        }

        pub fn read_nvr_apb(&self, src: u16) -> u32 {
            unsafe { self.resolve(&HEADER.read_nvr_apb)(src) }
        }
    }
}