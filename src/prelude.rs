//MIT License
//
// Copyright (c) 2019 LongYinan & Armin Sander
// Copyright (c) 2019 rust-skia Contributors
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

// See https://github.com/rust-skia/rust-skia
// Licence https://github.com/rust-skia/rust-skia/blob/master/LICENSE

use std::mem;
use std::mem::MaybeUninit;

/// Swiss army knife to convert any reference into any other.
pub(crate) unsafe fn transmute_ref<FromT, ToT>(from: &FromT) -> &ToT {
    // TODO: can we do this statically for all instantiations of transmute_ref?
    debug_assert_eq!(mem::size_of::<FromT>(), mem::size_of::<ToT>());
    &*(from as *const FromT as *const ToT)
}

pub(crate) unsafe fn transmute_ref_mut<FromT, ToT>(from: &mut FromT) -> &mut ToT {
    // TODO: can we do this statically for all instantiations of transmute_ref_mut?
    debug_assert_eq!(mem::size_of::<FromT>(), mem::size_of::<ToT>());
    &mut *(from as *mut FromT as *mut ToT)
}

/// Constructs a C++ object in place by calling a lambda that is meant to initialize
/// the pointer to the Rust memory provided as a pointer.
pub(crate) fn construct<N>(construct: impl FnOnce(*mut N)) -> N {
    let mut instance = MaybeUninit::uninit();
    construct(instance.as_mut_ptr());
    unsafe { instance.assume_init() }
}

/// Trait to use native types that as a rust type
/// _inplace_ with the same size and field layout.
pub trait NativeTransmutable<NT: Sized>: Sized {
    /// Provides access to the native value through a
    /// transmuted reference to the Rust value.
    fn native(&self) -> &NT {
        unsafe { transmute_ref(self) }
    }

    /// Provides mutable access to the native value through a
    /// transmuted reference to the Rust value.
    fn native_mut(&mut self) -> &mut NT {
        unsafe { transmute_ref_mut(self) }
    }

    /// Copies the native value to an equivalent Rust value.
    ///
    /// The `_c` suffix is to remind callers that functions that return a native value from a C++
    /// ABI can't be used. For example, C++ member functions must be wrapped in a extern "C" function.
    fn from_native_c(nt: NT) -> Self {
        let r = unsafe { mem::transmute_copy::<NT, Self>(&nt) };
        // don't drop, the Rust type takes over.
        mem::forget(nt);
        r
    }

    /// Copies the rust type to an equivalent instance of the native type.
    fn into_native(self) -> NT {
        let r = unsafe { mem::transmute_copy::<Self, NT>(&self) };
        // don't drop, the native type takes over.
        mem::forget(self);
        r
    }

    /// Provides access to the Rust value through a
    /// transmuted reference to the native value.
    fn from_native_ref(nt: &NT) -> &Self {
        unsafe { transmute_ref(nt) }
    }

    /// Provides access to the Rust value through a
    /// transmuted reference to the native mutable value.
    fn from_native_ref_mut(nt: &mut NT) -> &mut Self {
        unsafe { transmute_ref_mut(nt) }
    }

    /// Runs a test that proves that the native and the rust
    /// type are of the same size.
    fn test_layout() {
        assert_eq!(mem::size_of::<Self>(), mem::size_of::<NT>());
    }

    fn construct(construct: impl FnOnce(*mut NT)) -> Self {
        Self::from_native_c(self::construct(construct))
    }
}
