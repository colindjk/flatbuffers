/*
 * Copyright 2020 Google Inc. All rights reserved.
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

use crate::{
    Buffer, Follow, ForwardsUOffset, InvalidFlatbuffer, SkipSizePrefix, Verifiable, Verifier,
    VerifierOptions,
};

/// Gets the root of the Flatbuffer, verifying it first with default options.
/// Note that verification is an experimental feature and may not be maximally performant or
/// catch every error (though that is the goal). See the `_unchecked` variants for previous
/// behavior.
pub fn root<T, B>(data: B) -> Result<T::Inner, InvalidFlatbuffer>
where
    B: Buffer,
    T: Follow<B> + Verifiable,
{
    let opts = VerifierOptions::default();
    root_with_opts::<T, B>(&opts, data)
}

#[inline]
/// Gets the root of the Flatbuffer, verifying it first with given options.
/// Note that verification is an experimental feature and may not be maximally performant or
/// catch every error (though that is the goal). See the `_unchecked` variants for previous
/// behavior.
pub fn root_with_opts<'opts, T, B>(
    opts: &'opts VerifierOptions,
    data: B,
) -> Result<T::Inner, InvalidFlatbuffer>
where
    T: Follow<B> + Verifiable,
    B: Buffer,
{
    let mut v = Verifier::new(&opts, &data);
    <ForwardsUOffset<T>>::run_verifier(&mut v, 0)?;
    Ok(unsafe { root_unchecked::<T, B>(data) })
}

#[inline]
/// Gets the root of a size prefixed Flatbuffer, verifying it first with default options.
/// Note that verification is an experimental feature and may not be maximally performant or
/// catch every error (though that is the goal). See the `_unchecked` variants for previous
/// behavior.
pub fn size_prefixed_root<T, B>(data: B) -> Result<T::Inner, InvalidFlatbuffer>
where
    T: Follow<B> + Verifiable,
    B: Buffer,
{
    let opts = VerifierOptions::default();
    size_prefixed_root_with_opts::<T, B>(&opts, data)
}

#[inline]
/// Gets the root of a size prefixed Flatbuffer, verifying it first with given options.
/// Note that verification is an experimental feature and may not be maximally performant or
/// catch every error (though that is the goal). See the `_unchecked` variants for previous
/// behavior.
pub fn size_prefixed_root_with_opts<'opts, T, B>(
    opts: &'opts VerifierOptions,
    data: B,
) -> Result<T::Inner, InvalidFlatbuffer>
where
    T: Follow<B> + Verifiable,
    B: Buffer,
{
    let mut v = Verifier::new(&opts, &data);
    <SkipSizePrefix<ForwardsUOffset<T>>>::run_verifier(&mut v, 0)?;
    Ok(unsafe { size_prefixed_root_unchecked::<T, B>(data) })
}

#[inline]
/// Gets root for a trusted Flatbuffer.
/// # Safety
/// Flatbuffers accessors do not perform validation checks before accessing. Unlike the other
/// `root` functions, this does not validate the flatbuffer before returning the accessor. Users
/// must trust `data` contains a valid flatbuffer (e.g. b/c it was built by your software). Reading
/// unchecked buffers may cause panics or even UB.
pub unsafe fn root_unchecked<T, B>(data: B) -> T::Inner
where
    T: Follow<B>,
    B: Buffer,
{
    <ForwardsUOffset<T>>::follow(data, 0)
}

#[inline]
/// Gets root for a trusted, size prefixed, Flatbuffer.
/// # Safety
/// Flatbuffers accessors do not perform validation checks before accessing. Unlike the other
/// `root` functions, this does not validate the flatbuffer before returning the accessor. Users
/// must trust `data` contains a valid flatbuffer (e.g. b/c it was built by your software). Reading
/// unchecked buffers may cause panics or even UB.
pub unsafe fn size_prefixed_root_unchecked<T, B>(data: B) -> T::Inner
where
    T: Follow<B>,
    B: Buffer,
{
    <SkipSizePrefix<ForwardsUOffset<T>>>::follow(data, 0)
}
