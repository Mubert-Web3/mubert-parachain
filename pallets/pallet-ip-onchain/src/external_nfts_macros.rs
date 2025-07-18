// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Implements encoding and decoding traits for a wrapper type that represents
/// bitflags. The wrapper type should contain a field of type `$size`, where
/// `$size` is an integer type (e.g., u8, u16, u32) that can represent the bitflags.
/// The `$bitflag_enum` type is the enumeration type that defines the individual bitflags.
///
/// This macro provides implementations for the following traits:
/// - `MaxEncodedLen`: Calculates the maximum encoded length for the wrapper type.
/// - `Encode`: Encodes the wrapper type using the provided encoding function.
/// - `EncodeLike`: Trait indicating the type can be encoded as is.
/// - `Decode`: Decodes the wrapper type from the input.
/// - `TypeInfo`: Provides type information for the wrapper type.
macro_rules! impl_codec_bitflags {
    ($wrapper:ty, $size:ty, $bitflag_enum:ty) => {
        impl MaxEncodedLen for $wrapper {
            fn max_encoded_len() -> usize {
                <$size>::max_encoded_len()
            }
        }
        impl Encode for $wrapper {
            fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
                self.0.bits().using_encoded(f)
            }
        }
        impl EncodeLike for $wrapper {}
        impl Decode for $wrapper {
            fn decode<I: scale_codec::Input>(
                input: &mut I,
            ) -> ::core::result::Result<Self, scale_codec::Error> {
                let field = <$size>::decode(input)?;
                Ok(Self(
                    BitFlags::from_bits(field as $size).map_err(|_| "invalid value")?,
                ))
            }
        }

        impl TypeInfo for $wrapper {
            type Identity = Self;

            fn type_info() -> Type {
                Type::builder()
                    .path(Path::new("BitFlags", module_path!()))
                    .type_params(vec![TypeParameter::new(
                        "T",
                        Some(meta_type::<$bitflag_enum>()),
                    )])
                    .composite(
                        Fields::unnamed()
                            .field(|f| f.ty::<$size>().type_name(stringify!($bitflag_enum))),
                    )
            }
        }
    };
}
pub(crate) use impl_codec_bitflags;
