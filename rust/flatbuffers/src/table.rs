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

use crate::buffer::Buffer;
use crate::follow::Follow;
use crate::primitives::*;
use crate::vtable::VTable;

#[derive(Clone, Debug, PartialEq)]
pub struct Table<B> {
    pub buf: B,
    pub loc: usize,
}

impl<B> Table<B> where B: Buffer {
    #[inline]
    pub fn new(buf: B, loc: usize) -> Self {
        Table { buf, loc }
    }
    #[inline]
    pub fn vtable(&self) -> VTable<B> {
        <BackwardsSOffset<VTable<B>>>::follow(self.buf.shallow_copy(), self.loc)
    }
    #[inline]
    pub fn get<T: Follow<B>>(
        &self,
        slot_byte_loc: VOffsetT,
        default: Option<T::Inner>,
    ) -> Option<T::Inner> {
        let o = self.vtable().get(slot_byte_loc) as usize;
        if o == 0 {
            return default;
        }
        Some(<T>::follow(self.buf.shallow_copy(), self.loc + o))
    }
}

impl<B> Follow<B> for Table<B> where B: Buffer {
    type Inner = Table<B>;
    #[inline]
    fn follow(buf: B, loc: usize) -> Self::Inner {
        Table { buf, loc }
    }
}

#[inline]
pub fn buffer_has_identifier(data: &[u8], ident: &str, size_prefixed: bool) -> bool {
    assert_eq!(ident.len(), FILE_IDENTIFIER_LENGTH);

    let got = if size_prefixed {
        <SkipSizePrefix<SkipRootOffset<FileIdentifier>>>::follow(data, 0)
    } else {
        <SkipRootOffset<FileIdentifier>>::follow(data, 0)
    };

    ident.as_bytes() == got
}
