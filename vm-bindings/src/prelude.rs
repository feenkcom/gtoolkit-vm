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
#![allow(dead_code)]

use std::hash::{Hash, Hasher};
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::{mem, ptr, slice};
// Re-export TryFrom / TryInto to make them available in all modules that use prelude::*.
pub use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;

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

pub(crate) trait IntoOption {
    type Target;
    fn into_option(self) -> Option<Self::Target>;
}

impl<T> IntoOption for *const T {
    type Target = *const T;

    fn into_option(self) -> Option<Self::Target> {
        if !self.is_null() {
            Some(self)
        } else {
            None
        }
    }
}

impl<T> IntoOption for *mut T {
    type Target = ptr::NonNull<T>;

    fn into_option(self) -> Option<Self::Target> {
        ptr::NonNull::new(self)
    }
}

impl IntoOption for bool {
    type Target = ();

    fn into_option(self) -> Option<Self::Target> {
        if self {
            Some(())
        } else {
            None
        }
    }
}

pub(crate) trait IfBoolSome {
    fn if_true_some<V>(self, v: V) -> Option<V>;
    fn if_false_some<V>(self, v: V) -> Option<V>;
    fn if_true_then_some<V>(self, f: impl FnOnce() -> V) -> Option<V>;
    fn if_false_then_some<V>(self, f: impl FnOnce() -> V) -> Option<V>;
}

impl IfBoolSome for bool {
    fn if_true_some<V>(self, v: V) -> Option<V> {
        self.into_option().and(Some(v))
    }

    fn if_false_some<V>(self, v: V) -> Option<V> {
        (!self).if_true_some(v)
    }

    fn if_true_then_some<V>(self, f: impl FnOnce() -> V) -> Option<V> {
        self.into_option().map(|()| f())
    }

    fn if_false_then_some<V>(self, f: impl FnOnce() -> V) -> Option<V> {
        (!self).into_option().map(|()| f())
    }
}

/// Trait that enables access to a native representation of a wrapper type.
pub trait NativeAccess<N> {
    /// Provides shared access to the native type of the wrapper.
    fn native(&self) -> &N;

    /// Provides exclusive access to the native type of the wrapper.
    fn native_mut(&mut self) -> &mut N;

    // Returns a ptr to the native mutable value.
    unsafe fn native_mut_force(&self) -> *mut N {
        self.native() as *const N as *mut N
    }
}

/// Implements Drop for native types we can not implement Drop for.
pub trait NativeDrop {
    fn drop(&mut self);
}

/// Clone for bindings types we can not implement Clone for.
pub trait NativeClone {
    fn clone(&self) -> Self;
}

/// Even though some types may have value semantics, equality
/// comparison may need to be customized.
pub trait NativePartialEq {
    fn eq(&self, rhs: &Self) -> bool;
}

/// Implements Hash for the native type so that the wrapper type
/// can derive it from.
pub trait NativeHash {
    fn hash<H: Hasher>(&self, state: &mut H);
}

/// Wraps a native type that can be represented and used in Rust memory.
///
/// This type requires the trait `NativeDrop` to be implemented.
#[repr(transparent)]
pub struct Handle<N: NativeDrop>(
    N,
    // `*const` is needed to suppress automatic Send and Sync derivation, which happens when the
    // underlying type generated by bindgen is Send and Sync.
    PhantomData<*const ()>,
);

impl<N: NativeDrop> AsRef<Handle<N>> for Handle<N> {
    fn as_ref(&self) -> &Self {
        &self
    }
}

impl<N: NativeDrop> Handle<N> {
    /// Wrap a native instance into a handle.
    pub(crate) fn from_native_c(n: N) -> Self {
        Handle(n, PhantomData)
    }

    /// Create a reference to the Rust wrapper from a reference to the native type.
    pub(crate) fn from_native_ref(n: &N) -> &Self {
        unsafe { transmute_ref(n) }
    }

    /// Create a mutable reference to the Rust wrapper from a reference to the native type.
    pub(crate) fn from_native_ref_mut(n: &mut N) -> &mut Self {
        unsafe { transmute_ref_mut(n) }
    }

    /// Converts a pointer to a native value into a pointer to the Rust value.
    pub(crate) fn from_native_ptr(np: *const N) -> *const Self {
        np as _
    }

    /// Converts a pointer to a mutable native value into a pointer to the mutable Rust value.
    #[allow(unused)]
    pub(crate) fn from_native_ptr_mut(np: *mut N) -> *mut Self {
        np as _
    }

    /// Constructs a C++ object in place by calling a
    /// function that expects a pointer that points to
    /// uninitialized memory of the native type.
    pub(crate) fn construct(construct: impl FnOnce(*mut N)) -> Self {
        Self::try_construct(|i| {
            construct(i);
            true
        })
        .unwrap()
    }

    pub(crate) fn try_construct(construct: impl FnOnce(*mut N) -> bool) -> Option<Self> {
        self::try_construct(construct).map(Self::from_native_c)
    }

    /// Replaces the native instance with the one from this Handle, and returns the replaced one
    /// wrapped in a Rust Handle without dropping either one.
    pub(crate) fn replace_native(mut self, native: &mut N) -> Self {
        mem::swap(&mut self.0, native);
        self
    }

    /// Consumes the wrapper and returns the native type.
    pub(crate) fn into_native(mut self) -> N {
        let r = mem::replace(&mut self.0, unsafe { mem::zeroed() });
        mem::forget(self);
        r
    }
}

pub(crate) trait ReplaceWith<Other> {
    fn replace_with(&mut self, other: Other) -> Other;
}

impl<N: NativeDrop> ReplaceWith<Handle<N>> for N {
    fn replace_with(&mut self, other: Handle<N>) -> Handle<N> {
        other.replace_native(self)
    }
}

/// Constructs a C++ object in place by calling a lambda that is meant to initialize
/// the pointer to the Rust memory provided as a pointer.
pub(crate) fn construct<N>(construct: impl FnOnce(*mut N)) -> N {
    try_construct(|i| {
        construct(i);
        true
    })
    .unwrap()
}

pub(crate) fn try_construct<N>(construct: impl FnOnce(*mut N) -> bool) -> Option<N> {
    let mut instance = MaybeUninit::uninit();
    construct(instance.as_mut_ptr()).if_true_then_some(|| unsafe { instance.assume_init() })
}

impl<N: NativeDrop> Drop for Handle<N> {
    fn drop(&mut self) {
        self.0.drop()
    }
}

impl<N: NativeDrop> NativeAccess<N> for Handle<N> {
    fn native(&self) -> &N {
        &self.0
    }

    fn native_mut(&mut self) -> &mut N {
        &mut self.0
    }
}

impl<N: NativeDrop + NativeClone> Clone for Handle<N> {
    fn clone(&self) -> Self {
        Self::from_native_c(self.0.clone())
    }
}

impl<N: NativeDrop + NativePartialEq> PartialEq for Handle<N> {
    fn eq(&self, rhs: &Self) -> bool {
        self.native().eq(rhs.native())
    }
}

impl<N: NativeDrop + NativeHash> Hash for Handle<N> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.native().hash(state);
    }
}

pub(crate) trait NativeSliceAccess<N: NativeDrop> {
    fn native(&self) -> &[N];
    fn native_mut(&mut self) -> &mut [N];
}

impl<N: NativeDrop> NativeSliceAccess<N> for [Handle<N>] {
    fn native(&self) -> &[N] {
        let ptr = self
            .first()
            .map(|f| f.native() as *const N)
            .unwrap_or(ptr::null());
        unsafe { slice::from_raw_parts(ptr, self.len()) }
    }

    fn native_mut(&mut self) -> &mut [N] {
        let ptr = self
            .first_mut()
            .map(|f| f.native_mut() as *mut N)
            .unwrap_or(ptr::null_mut());
        unsafe { slice::from_raw_parts_mut(ptr, self.len()) }
    }
}

/// A trait that supports retrieving a pointer from an Option<Handle<Native>>.
/// Returns a null pointer if the Option is None.
pub(crate) trait NativePointerOrNull<N> {
    fn native_ptr_or_null(&self) -> *const N;
    unsafe fn native_ptr_or_null_mut_force(&self) -> *mut N;
}

pub(crate) trait NativePointerOrNullMut<N> {
    fn native_ptr_or_null_mut(&mut self) -> *mut N;
}

impl<H, N> NativePointerOrNull<N> for Option<&H>
where
    H: NativeAccess<N>,
{
    fn native_ptr_or_null(&self) -> *const N {
        match self {
            Some(handle) => handle.native(),
            None => ptr::null(),
        }
    }

    unsafe fn native_ptr_or_null_mut_force(&self) -> *mut N {
        match self {
            Some(handle) => handle.native_mut_force(),
            None => ptr::null_mut(),
        }
    }
}

impl<H, N> NativePointerOrNullMut<N> for Option<&mut H>
where
    H: NativeAccess<N>,
{
    fn native_ptr_or_null_mut(&mut self) -> *mut N {
        match self {
            Some(handle) => handle.native_mut(),
            None => ptr::null_mut(),
        }
    }
}

pub(crate) trait NativePointerOrNullMut2<N> {
    fn native_ptr_or_null_mut(&mut self) -> *mut N;
}

pub(crate) trait NativePointerOrNull2<N> {
    fn native_ptr_or_null(&self) -> *const N;
}

impl<H, N> NativePointerOrNull2<N> for Option<&H>
where
    H: NativeTransmutable<N>,
{
    fn native_ptr_or_null(&self) -> *const N {
        match self {
            Some(handle) => handle.native(),
            None => ptr::null(),
        }
    }
}

impl<H, N> NativePointerOrNullMut2<N> for Option<&mut H>
where
    H: NativeTransmutable<N>,
{
    fn native_ptr_or_null_mut(&mut self) -> *mut N {
        match self {
            Some(handle) => handle.native_mut(),
            None => ptr::null_mut(),
        }
    }
}

/// A wrapper type that represents a native type with a pointer to
/// the native object.
#[repr(transparent)]
pub struct RefHandle<N: NativeDrop>(ptr::NonNull<N>);

impl<N: NativeDrop> Drop for RefHandle<N> {
    fn drop(&mut self) {
        self.native_mut().drop()
    }
}

impl<N: NativeDrop> NativeAccess<N> for RefHandle<N> {
    fn native(&self) -> &N {
        unsafe { self.0.as_ref() }
    }
    fn native_mut(&mut self) -> &mut N {
        unsafe { self.0.as_mut() }
    }
}

impl<N: NativeDrop> RefHandle<N> {
    /// Creates a RefHandle from a native pointer.
    ///
    /// From this time on, the handle owns the object that the pointer points
    /// to and will call its NativeDrop implementation if it goes out of scope.
    pub(crate) fn from_ptr(ptr: *mut N) -> Option<Self> {
        ptr::NonNull::new(ptr).map(Self)
    }

    pub(crate) fn into_ptr(self) -> *mut N {
        let p = self.0.as_ptr();
        mem::forget(self);
        p
    }
}

/// A trait that consumes self and converts it to a ptr to the native type.
pub(crate) trait IntoPtr<N> {
    fn into_ptr(self) -> *mut N;
}

/// A trait that consumes self and converts it to a ptr to the native type or null.
pub(crate) trait IntoPtrOrNull<N> {
    fn into_ptr_or_null(self) -> *mut N;
}

/// Tag the type to automatically implement get() functions for
/// all Index implementations.
pub trait IndexGet {}

/// Tag the type to automatically implement get() and set() functions
/// for all Index & IndexMut implementation for that type.
pub trait IndexSet {}

pub trait IndexGetter<I, O: Copy> {
    fn get(&self, index: I) -> O;
}

impl<T, I, O: Copy> IndexGetter<I, O> for T
where
    T: Index<I, Output = O> + IndexGet,
{
    fn get(&self, index: I) -> O {
        self[index]
    }
}

pub trait IndexSetter<I, O: Copy> {
    fn set(&mut self, index: I, value: O) -> &mut Self;
}

impl<T, I, O: Copy> IndexSetter<I, O> for T
where
    T: IndexMut<I, Output = O> + IndexSet,
{
    fn set(&mut self, index: I, value: O) -> &mut Self {
        self[index] = value;
        self
    }
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

    /// Converts a pointer to a native value into a pointer to the Rust value.
    fn from_native_ptr(np: *const NT) -> *const Self {
        np as _
    }

    /// Converts a pointer to a mutable native value into a pointer to the mutable Rust value.
    fn from_native_ptr_mut(np: *mut NT) -> *mut Self {
        np as _
    }

    /// Runs a test that proves that the native and the rust
    /// type are of the same size.
    fn test_layout() {
        assert_eq!(mem::size_of::<Self>(), mem::size_of::<NT>());
    }

    fn construct(construct: impl FnOnce(*mut NT)) -> Self {
        Self::try_construct(|i| {
            construct(i);
            true
        })
        .unwrap()
    }

    fn try_construct(construct: impl FnOnce(*mut NT) -> bool) -> Option<Self> {
        self::try_construct(construct).map(Self::from_native_c)
    }
}

pub(crate) trait NativeTransmutableSliceAccess<NT: Sized> {
    fn native(&self) -> &[NT];
    fn native_mut(&mut self) -> &mut [NT];
}

impl<NT, ElementT> NativeTransmutableSliceAccess<NT> for [ElementT]
where
    ElementT: NativeTransmutable<NT>,
{
    fn native(&self) -> &[NT] {
        unsafe { &*(self as *const [ElementT] as *const [NT]) }
    }

    fn native_mut(&mut self) -> &mut [NT] {
        unsafe { &mut *(self as *mut [ElementT] as *mut [NT]) }
    }
}

impl<NT, RustT> NativeTransmutable<Option<NT>> for Option<RustT> where RustT: NativeTransmutable<NT> {}

impl<NT, RustT> NativeTransmutable<Option<&[NT]>> for Option<&[RustT]> where
    RustT: NativeTransmutable<NT>
{
}

pub(crate) trait NativeTransmutableOptionSliceAccessMut<NT: Sized> {
    fn native_mut(&mut self) -> &mut Option<&mut [NT]>;
}

impl<NT, RustT> NativeTransmutableOptionSliceAccessMut<NT> for Option<&mut [RustT]>
where
    RustT: NativeTransmutable<NT>,
{
    fn native_mut(&mut self) -> &mut Option<&mut [NT]> {
        unsafe { transmute_ref_mut(self) }
    }
}

//
// Convenience functions to access Option<&[]> as optional ptr (opt_ptr)
// that may be null.
//

pub(crate) trait AsPointerOrNull<PointerT> {
    fn as_ptr_or_null(&self) -> *const PointerT;
}

pub(crate) trait AsPointerOrNullMut<PointerT>: AsPointerOrNull<PointerT> {
    fn as_ptr_or_null_mut(&mut self) -> *mut PointerT;
}

impl<E> AsPointerOrNull<E> for Option<E> {
    fn as_ptr_or_null(&self) -> *const E {
        match self {
            Some(e) => e,
            None => ptr::null(),
        }
    }
}

impl<E> AsPointerOrNullMut<E> for Option<E> {
    fn as_ptr_or_null_mut(&mut self) -> *mut E {
        match self {
            Some(e) => e,
            None => ptr::null_mut(),
        }
    }
}

impl<E> AsPointerOrNull<E> for Option<&[E]> {
    fn as_ptr_or_null(&self) -> *const E {
        match self {
            Some(slice) => slice.as_ptr(),
            None => ptr::null(),
        }
    }
}

impl<E> AsPointerOrNull<E> for Option<&mut [E]> {
    fn as_ptr_or_null(&self) -> *const E {
        match self {
            Some(slice) => slice.as_ptr(),
            None => ptr::null(),
        }
    }
}

impl<E> AsPointerOrNullMut<E> for Option<&mut [E]> {
    fn as_ptr_or_null_mut(&mut self) -> *mut E {
        match self {
            Some(slice) => slice.as_mut_ptr(),
            None => ptr::null_mut(),
        }
    }
}

impl<E> AsPointerOrNull<E> for Option<&Vec<E>> {
    fn as_ptr_or_null(&self) -> *const E {
        match self {
            Some(v) => v.as_ptr(),
            None => ptr::null(),
        }
    }
}

impl<E> AsPointerOrNull<E> for Option<Vec<E>> {
    fn as_ptr_or_null(&self) -> *const E {
        match self {
            Some(v) => v.as_ptr(),
            None => ptr::null(),
        }
    }
}

impl<E> AsPointerOrNullMut<E> for Option<Vec<E>> {
    fn as_ptr_or_null_mut(&mut self) -> *mut E {
        match self {
            Some(v) => v.as_mut_ptr(),
            None => ptr::null_mut(),
        }
    }
}

// Wraps a handle so that the Rust's borrow checker assumes it represents
// something that borrows something else.
#[repr(transparent)]
pub struct Borrows<'a, H>(H, PhantomData<&'a ()>);

impl<'a, H> Deref for Borrows<'a, H> {
    type Target = H;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO: this is most likely unsafe because someone could replace the
// value the reference is pointing to.
impl<'a, H> DerefMut for Borrows<'a, H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, H> Borrows<'a, H> {
    /// Notify that the borrowed dependency is not referred to anymore and return the handle.
    /// # Safety
    /// The borrowed dependency must be removed before calling `release()`.
    pub unsafe fn release(self) -> H {
        self.0
    }
}

pub(crate) trait BorrowsFrom: Sized {
    fn borrows<D: ?Sized>(self, _dep: &D) -> Borrows<Self>;
}

impl<T: Sized> BorrowsFrom for T {
    fn borrows<D: ?Sized>(self, _dep: &D) -> Borrows<Self> {
        Borrows(self, PhantomData)
    }
}

/// Declares a base class for a native type.
pub trait NativeBase<Base> {
    fn base(&self) -> &Base {
        unsafe { &*(self as *const Self as *const Base) }
    }

    fn base_mut(&mut self) -> &mut Base {
        unsafe { &mut *(self as *mut Self as *mut Base) }
    }
}

pub struct Sendable<H: ConditionallySend>(H);
unsafe impl<H: ConditionallySend> Send for Sendable<H> {}

impl<H: ConditionallySend> Sendable<H> {
    pub fn unwrap(self) -> H {
        self.0
    }
}

pub trait ConditionallySend: Sized {
    /// Returns `true` if the handle can be sent to another thread.
    fn can_send(&self) -> bool;
    /// Wrap the handle in a type that can be sent to another thread and unwrapped there.
    ///
    /// Guaranteed to succeed of can_send() returns `true`.
    fn wrap_send(self) -> Result<Sendable<Self>, Self>;
}

/// Functions that are (supposedly) _safer_ variants of the ones Rust provides.
pub(crate) mod safer {
    use core::slice;
    use std::ptr;

    /// Invokes [slice::from_raw_parts] with the `ptr` only when `len` != 0, otherwise passes
    /// `ptr::NonNull::dangling()` as recommended.
    ///
    /// Panics if `len` != 0 and `ptr` is `null`.
    pub unsafe fn from_raw_parts<'a, T>(ptr: *const T, len: usize) -> &'a [T] {
        let ptr = if len == 0 {
            ptr::NonNull::dangling().as_ptr()
        } else {
            assert!(!ptr.is_null());
            ptr
        };
        slice::from_raw_parts(ptr, len)
    }

    /// Invokes [slice::from_raw_parts_mut] with the `ptr` only if `len` != 0, otherwise passes
    /// `ptr::NonNull::dangling()` as recommended.
    ///
    /// Panics if `len` != 0 and `ptr` is `null`.
    pub unsafe fn from_raw_parts_mut<'a, T>(ptr: *mut T, len: usize) -> &'a mut [T] {
        let ptr = if len == 0 {
            ptr::NonNull::dangling().as_ptr() as *mut _
        } else {
            assert!(!ptr.is_null());
            ptr
        };
        slice::from_raw_parts_mut(ptr, len)
    }
}
