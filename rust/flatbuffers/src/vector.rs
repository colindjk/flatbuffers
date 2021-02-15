/*
 * Copyright 2018 Google Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::fmt::{Debug, Formatter, Result};
use std::iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator};
use std::marker::PhantomData;
use std::mem::size_of;
use std::slice::from_raw_parts;

use crate::buffer::Buffer;
use crate::endian_scalar::read_scalar_at;
#[cfg(target_endian = "little")]
use crate::endian_scalar::EndianScalar;
use crate::follow::Follow;
use crate::primitives::*;

pub struct Vector<B, T>(B, usize, PhantomData<T>);

impl<B, T> Default for Vector<B, T> where B: Buffer {
    fn default() -> Self {
        // Static, length 0 vector.
        // Note that derived default causes UB due to issues in read_scalar_at /facepalm.
        Self(B::from_static_slice(&[0; core::mem::size_of::<UOffsetT>()]), 0, Default::default())
    }
}

impl<B, T> Debug for Vector<B, T>
where
    B: Buffer,
    T: Follow<B>,
    <T as Follow<B>>::Inner: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

// We cannot use derive for these two impls, as it would only implement Copy
// and Clone for `T: Copy` and `T: Clone` respectively. However `Vector<'a, T>`
// can always be copied, no matter that `T` you have.
impl<B, T> Clone for Vector<B, T> where B: Buffer {
    fn clone(&self) -> Self {
        // TODO(colindjk) Is this kosher? (does it deep copy buf?)
        let Vector(ref buf, size, phan) = *self;
        Vector(buf.shallow_copy(), size, phan)
    }
}

impl<B, T> Vector<B, T> where B: Buffer {
    #[inline(always)]
    pub fn new(buf: B, loc: usize) -> Self {
        Vector {
            0: buf,
            1: loc,
            2: PhantomData,
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        read_scalar_at::<UOffsetT>(&self.0, self.1) as usize
    }
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<B: Buffer, T: Follow<B>> Vector<B, T> {
    #[inline(always)]
    pub fn get(&self, idx: usize) -> T::Inner {
        debug_assert!(idx < read_scalar_at::<u32>(&self.0, self.1) as usize);
        let sz = size_of::<T>();
        debug_assert!(sz > 0);
        T::follow(self.0.shallow_copy(), self.1 as usize + SIZE_UOFFSET + sz * idx)
    }

    #[inline(always)]
    pub fn iter(&self) -> VectorIter<B, T> {
        VectorIter::new(self.clone())
    }
}

pub trait SafeSliceAccess {}
impl<'a, B, T: SafeSliceAccess> Vector<B, T> where B: Buffer {
    pub fn safe_slice(self) -> &'a [T] {
        let buf = self.0;
        let loc = self.1;
        let sz = size_of::<T>();
        debug_assert!(sz > 0);
        let len = read_scalar_at::<UOffsetT>(&buf, loc) as usize;
        let data_buf = &buf[loc + SIZE_UOFFSET..loc + SIZE_UOFFSET + len * sz];
        let ptr = data_buf.as_ptr() as *const T;
        // FIXME(colindjk) Buffer<T> this is very unsafe. Move this logic to Buffer impl.
        let s: &'a [T] = unsafe { from_raw_parts(ptr, len) };
        s
    }
}

impl SafeSliceAccess for u8 {}
impl SafeSliceAccess for i8 {}
impl SafeSliceAccess for bool {}

// TODO(caspern): Get rid of this. Conditional compliation is unnecessary complexity.
// Vectors of primitives just don't work on big endian machines!!!
#[cfg(target_endian = "little")]
mod le_safe_slice_impls {
    impl super::SafeSliceAccess for u16 {}
    impl super::SafeSliceAccess for u32 {}
    impl super::SafeSliceAccess for u64 {}

    impl super::SafeSliceAccess for i16 {}
    impl super::SafeSliceAccess for i32 {}
    impl super::SafeSliceAccess for i64 {}

    impl super::SafeSliceAccess for f32 {}
    impl super::SafeSliceAccess for f64 {}
}

#[cfg(target_endian = "little")]
pub use self::le_safe_slice_impls::*;

pub fn follow_cast_ref<'a, T: Sized + 'a>(buf: &'a [u8], loc: usize) -> &'a T {
    let sz = size_of::<T>();
    let buf = &buf[loc..loc + sz];
    let ptr = buf.as_ptr() as *const T;
    unsafe { &*ptr }
}

impl<'a, B> Follow<B> for &'a str where B: 'a + Buffer {
    type Inner = B::BufferString;
    fn follow(buf: B, loc: usize) -> Self::Inner {
        let len = read_scalar_at::<UOffsetT>(&buf, loc) as usize;
        // TODO(colindjk) add unchecked impl for performance purposes. 
        buf.slice(loc + SIZE_UOFFSET..loc + SIZE_UOFFSET + len)
            .unwrap_or(B::empty())
            .buffer_str()
            .unwrap_or(B::empty_str())
    }
}

#[cfg(target_endian = "little")]
fn follow_slice_helper<'a, B: Buffer + 'a, T>(buf: B, loc: usize) -> &'a [T] {
    let sz = size_of::<T>();
    debug_assert!(sz > 0);
    let len = read_scalar_at::<UOffsetT>(&buf, loc) as usize;
    let data_buf = &buf[loc + SIZE_UOFFSET..loc + SIZE_UOFFSET + len * sz];
    let ptr = data_buf.as_ptr() as *const T;
    // FIXME(colindjk) Buffer<T> This is very unsafe :(
    // Need to double check if this is _actually_ unsafe, b/c of the guarantee from Buffer.
    let s: &[T] = unsafe { from_raw_parts(ptr, len) };
    s
}

/// Implement direct slice access if the host is little-endian.
#[cfg(target_endian = "little")]
impl<'a, B: Buffer + 'a, T: EndianScalar> Follow<B> for &'a [T] {
    type Inner = &'a [T];
    fn follow(buf: B, loc: usize) -> Self::Inner {
        follow_slice_helper::<B, T>(buf, loc)
    }
}

/// Implement Follow for all possible Vectors that have Follow-able elements.
impl<B: Buffer, T: Follow<B>> Follow<B> for Vector<B, T> {
    type Inner = Vector<B, T>;
    fn follow(buf: B, loc: usize) -> Self::Inner {
        Vector::new(buf, loc)
    }
}

/// An iterator over a `Vector`.
#[derive(Debug)]
pub struct VectorIter<B, T> {
    buf: B,
    loc: usize,
    remaining: usize,
    phantom: PhantomData<T>,
}

impl<B, T> VectorIter<B, T> where B: Buffer {
    #[inline]
    pub fn new(inner: Vector<B, T>) -> Self {
        VectorIter {
            buf: inner.0.shallow_copy(),
            // inner.1 is the location of the data for the vector.
            // The first SIZE_UOFFSET bytes is the length. We skip
            // that to get to the actual vector content.
            loc: inner.1 + SIZE_UOFFSET,
            remaining: inner.len(),
            phantom: PhantomData,
        }
    }
}

impl<B: Buffer, T: Follow<B>> Clone for VectorIter<B, T> {
    #[inline]
    fn clone(&self) -> Self {
        VectorIter {
            buf: self.buf.shallow_copy(),
            loc: self.loc,
            remaining: self.remaining,
            phantom: self.phantom,
        }
    }
}

impl<B: Buffer, T: Follow<B>> Iterator for VectorIter<B, T> {
    type Item = T::Inner;

    #[inline]
    fn next(&mut self) -> Option<T::Inner> {
        let sz = size_of::<T>();
        debug_assert!(sz > 0);

        if self.remaining == 0 {
            None
        } else {
            let result = T::follow(self.buf.shallow_copy(), self.loc);
            self.loc += sz;
            self.remaining -= 1;
            Some(result)
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<T::Inner> {
        let sz = size_of::<T>();
        debug_assert!(sz > 0);

        self.remaining = self.remaining.saturating_sub(n);

        // Note that this might overflow, but that is okay because
        // in that case self.remaining will have been set to zero.
        self.loc = self.loc.wrapping_add(sz * n);

        self.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<B: Buffer, T: Follow<B>> DoubleEndedIterator for VectorIter<B, T> {
    #[inline]
    fn next_back(&mut self) -> Option<T::Inner> {
        let sz = size_of::<T>();
        debug_assert!(sz > 0);

        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            Some(T::follow(self.buf.shallow_copy(), self.loc + sz * self.remaining))
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<T::Inner> {
        self.remaining = self.remaining.saturating_sub(n);
        self.next_back()
    }
}

impl<B: Buffer, T: Follow<B>> ExactSizeIterator for VectorIter<B, T> {
    #[inline]
    fn len(&self) -> usize {
        self.remaining
    }
}

impl<B: Buffer, T: Follow<B>> FusedIterator for VectorIter<B, T> {}

impl<B: Buffer, T: Follow<B>> IntoIterator for Vector<B, T> {
    type Item = T::Inner;
    type IntoIter = VectorIter<B, T>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, B: Buffer, T: Follow<B>> IntoIterator for &'a Vector<B, T> {
    type Item = T::Inner;
    type IntoIter = VectorIter<B, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
