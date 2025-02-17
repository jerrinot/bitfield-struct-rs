//! # Bitfield Struct
//!
//! Procedural macro for bitfields that allows specifying bitfields as structs.
//! As this library provides a procedural macro, it has no runtime dependencies
//! and works for `no-std` environments.
//!
//! - Supports bool flags, raw integers, and every custom type convertible into integers (structs/enums)
//! - Ideal for driver/OS/embedded development (defining HW registers/structures)
//! - Generates minimalistic, pure, safe rust functions
//! - Compile-time checks for type and field sizes
//! - Rust-analyzer friendly (carries over documentation to accessor functions)
//! - Exports field offsets and sizes as constants (useful for const asserts)
//! - Generation of `fmt::Debug` and `Default`
//!
//! ## Basics
//!
//! Let's begin with a simple example.<br>
//! Suppose we want to store multiple data inside a single Byte, as shown below:
//!
//! <table>
//!   <tr>
//!     <td>7</td>
//!     <td>6</td>
//!     <td>5</td>
//!     <td>4</td>
//!     <td>3</td>
//!     <td>3</td>
//!     <td>1</td>
//!     <td>0</td>
//!   </tr>
//!   <tr>
//!     <td>P</td>
//!     <td colspan="2">Level</td>
//!     <td>S</td>
//!     <td colspan="4">Kind</td>
//!   </tr>
//! </table>
//!
//! This crate is able to generate a nice wrapper type that makes it easy to do this:
//!
//! ```
//! # use bitfield_struct::bitfield;
//! /// Define your type like this with the bitfield attribute
//! #[bitfield(u8)]
//! struct MyByte {
//!     /// The first field occupies the least significant bits
//!     #[bits(4)]
//!     kind: usize,
//!     /// Booleans are 1 bit large
//!     system: bool,
//!     /// The bits attribute specifies the bit size of this field
//!     #[bits(2)]
//!     level: usize,
//!     /// The last field spans over the most significant bits
//!     present: bool
//! }
//! // The macro creates three accessor functions for each field:
//! // <name>, with_<name> and set_<name>
//! let my_byte = MyByte::new()
//!     .with_kind(15)
//!     .with_system(false)
//!     .with_level(3)
//!     .with_present(true);
//!
//! assert!(my_byte.present());
//! ```
//!
//! ## Features
//!
//! Additionally, this crate has a few useful features, which are shown here in more detail.
//!
//! The example below shows how attributes are carried over and how signed integers, padding, and custom types are handled.
//!
//! ```
//! # use bitfield_struct::bitfield;
//! /// A test bitfield with documentation
//! #[bitfield(u64)]
//! #[derive(PartialEq, Eq)] // <- Attributes after `bitfield` are carried over
//! struct MyBitfield {
//!     /// defaults to 16 bits for u16
//!     int: u16,
//!     /// interpreted as 1 bit flag, with a custom default value
//!     #[bits(default = true)]
//!     flag: bool,
//!     /// custom bit size
//!     #[bits(1)]
//!     tiny: u8,
//!     /// sign extend for signed integers
//!     #[bits(13)]
//!     negative: i16,
//!     /// supports any type, with `into_bits`/`from_bits` (const) functions,
//!     /// if not configured otherwise with the `into`/`from` parameters of the bits attribute.
//!     ///
//!     /// the field is initialized with 0 (passed into `from_bits`) if not specified otherwise
//!     #[bits(16)]
//!     custom: CustomEnum,
//!     /// public field -> public accessor functions
//!     #[bits(12)]
//!     pub public: usize,
//!     /// padding
//!     #[bits(5)]
//!     __: u8,
//! }
//!
//! /// A custom enum
//! #[derive(Debug, PartialEq, Eq)]
//! #[repr(u64)]
//! enum CustomEnum {
//!     A = 0,
//!     B = 1,
//!     C = 2,
//! }
//! impl CustomEnum {
//!     // This has to be a const fn
//!     const fn into_bits(self) -> u64 {
//!         self as _
//!     }
//!     const fn from_bits(value: u64) -> Self {
//!         match value {
//!             0 => Self::A,
//!             1 => Self::B,
//!             _ => Self::C,
//!         }
//!     }
//! }
//!
//! // Usage:
//! let mut val = MyBitfield::new()
//!     .with_int(3 << 15)
//!     .with_tiny(1)
//!     .with_negative(-3)
//!     .with_custom(CustomEnum::B)
//!     .with_public(2);
//!
//! println!("{val:?}");
//! let raw: u64 = val.into();
//! println!("{raw:b}");
//!
//! assert_eq!(val.int(), 3 << 15);
//! assert_eq!(val.flag(), true);
//! assert_eq!(val.negative(), -3);
//! assert_eq!(val.tiny(), 1);
//! assert_eq!(val.custom(), CustomEnum::B);
//! assert_eq!(val.public(), 2);
//!
//! // const members
//! assert_eq!(MyBitfield::FLAG_BITS, 1);
//! assert_eq!(MyBitfield::FLAG_OFFSET, 16);
//!
//! val.set_negative(1);
//! assert_eq!(val.negative(), 1);
//! ```
//!
//! The macro generates three accessor functions for each field.
//! Each accessor also inherits the documentation of its field.
//!
//! The signatures for `int` are:
//!
//! ```ignore
//! // generated struct
//! struct MyBitfield(u64);
//! impl MyBitfield {
//!     const fn new() -> Self { Self(0) }
//!
//!     const INT_BITS: usize = 16;
//!     const INT_OFFSET: usize = 0;
//!
//!     const fn with_int(self, value: u16) -> Self { /* ... */ }
//!     const fn int(&self) -> u16 { /* ... */ }
//!     fn set_int(&mut self, value: u16) { /* ... */ }
//!
//!     // other field ...
//! }
//! // generated trait implementations
//! impl From<u64> for MyBitfield { /* ... */ }
//! impl From<MyBitfield> for u64 { /* ... */ }
//! impl Debug for MyBitfield { /* ... */ }
//! ```
//!
//! > Hint: You can use the rust-analyzer "Expand macro recursively" action to view the generated code.
//!
//! ## Bit Order
//!
//! The optional `order` macro argument determines the layout of the bits, with the default being
//! Lsb (least significant bit) first:
//!
//! ```
//! # use bitfield_struct::bitfield;
//! #[bitfield(u8, order = Lsb)]
//! struct MyLsbByte {
//!     /// The first field occupies the least significant bits
//!     #[bits(4)]
//!     kind: usize,
//!     system: bool,
//!     #[bits(2)]
//!     level: usize,
//!     present: bool
//! }
//!
//! let my_byte_lsb = MyLsbByte::new()
//!     .with_kind(10)
//!     .with_system(false)
//!     .with_level(2)
//!     .with_present(true);
//!
//! //                         .- present
//! //                         | .- level
//! //                         | |  .- system
//! //                         | |  | .- kind
//! assert!(my_byte_lsb.0 == 0b1_10_0_1010);
//! ```
//!
//! The macro generates the reverse order when Msb (most significant bit) is specified:
//!
//! ```
//! # use bitfield_struct::bitfield;
//! #[bitfield(u8, order = Msb)]
//! struct MyMsbByte {
//!     /// The first field occupies the most significant bits
//!     #[bits(4)]
//!     kind: usize,
//!     system: bool,
//!     #[bits(2)]
//!     level: usize,
//!     present: bool
//! }
//!
//! let my_byte_msb = MyMsbByte::new()
//!     .with_kind(10)
//!     .with_system(false)
//!     .with_level(2)
//!     .with_present(true);
//!
//! //                         .- kind
//! //                         |    .- system
//! //                         |    | .- level
//! //                         |    | |  .- present
//! assert!(my_byte_msb.0 == 0b1010_0_10_1);
//! ```
//!
//! ## `fmt::Debug` and `Default`
//!
//! This macro automatically creates a suitable `fmt::Debug` and `Default` implementations
//! similar to the ones created for normal structs by `#[derive(Debug, Default)]`.
//! You can disable these with the extra `debug` and `default` arguments.
//!
//! ```
//! # use std::fmt;
//! # use bitfield_struct::bitfield;
//! #[bitfield(u64, debug = false, default = false)]
//! struct CustomDebug {
//!     data: u64
//! }
//!
//! impl fmt::Debug for CustomDebug {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         write!(f, "0x{:x}", self.data())
//!     }
//! }
//!
//! impl Default for CustomDebug {
//!     fn default() -> Self {
//!         Self(123) // note: you can also use `#[bits(64, default = 123)]`
//!     }
//! }
//!
//! let val = CustomDebug::default();
//! println!("{val:?}")
//! ```
//!

#![warn(clippy::unwrap_used)]

use proc_macro as pc;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use std::stringify;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::Token;

/// Creates a bitfield for this struct.
///
/// The arguments first, have to begin with the underlying type of the bitfield:
/// For example: `#[bitfield(u64)]`.
///
/// It can contain an extra `debug` argument for disabling the `Debug` trait
/// generation (`#[bitfield(u64, debug = false)]`).
#[proc_macro_attribute]
pub fn bitfield(args: pc::TokenStream, input: pc::TokenStream) -> pc::TokenStream {
    match bitfield_inner(args.into(), input.into()) {
        Ok(result) => result.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn bitfield_inner(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<syn::ItemStruct>(input)?;
    let Params {
        ty,
        bits,
        debug,
        default,
        order,
    } = syn::parse2::<Params>(args)?;

    let span = input.fields.span();
    let name = input.ident;
    let name_str = name.to_string();
    let vis = input.vis;
    let attrs: TokenStream = input.attrs.iter().map(ToTokens::to_token_stream).collect();

    let syn::Fields::Named(fields) = input.fields else {
        return Err(syn::Error::new(span, "only named fields are supported"));
    };

    let mut offset = 0;
    let mut members = Vec::with_capacity(fields.named.len());
    for field in fields.named {
        let f = Member::new(ty.clone(), bits, field, offset, order)?;
        offset += f.bits;
        members.push(f);
    }

    if offset < bits {
        return Err(syn::Error::new(
            span,
            format!(
                "The bitfiled size ({bits} bits) has to be equal to the sum of its members ({offset} bits)!. \
                You might have to add padding (a {} bits large member prefixed with \"_\").",
                bits - offset
            ),
        ));
    }
    if offset > bits {
        return Err(syn::Error::new(
            span,
            format!(
                "The size of the members ({offset} bits) is larger than the type ({bits} bits)!."
            ),
        ));
    }

    let debug_impl = if debug {
        let debug_fields = members.iter().map(Member::debug);
        quote! {
            impl core::fmt::Debug for #name {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_struct(#name_str)
                        #( #debug_fields )*
                        .finish()
                }
            }
        }
    } else {
        TokenStream::default()
    };

    let defaults = members.iter().map(Member::default);

    let default_impl = if default {
        quote! {
            impl Default for #name {
                fn default() -> Self {
                    Self::new()
                }
            }
        }
    } else {
        TokenStream::new()
    };

    Ok(quote! {
        #attrs
        #[derive(Copy, Clone)]
        #[repr(transparent)]
        #vis struct #name(#ty);

        impl #name {
            /// Creates a new default initialized bitfield.
            #vis const fn new() -> Self {
                let mut this = Self(0);
                #( #defaults )*
                this
            }

            #( #members )*
        }

        #default_impl

        impl From<#ty> for #name {
            fn from(v: #ty) -> Self {
                Self(v)
            }
        }
        impl From<#name> for #ty {
            fn from(v: #name) -> #ty {
                v.0
            }
        }

        #debug_impl
    })
}

/// Represents a member where accessor functions should be generated for.
struct Member {
    offset: usize,
    bits: usize,
    base_ty: syn::Type,
    default: TokenStream,
    inner: Option<MemberInner>,
}

struct MemberInner {
    ident: syn::Ident,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
    vis: syn::Visibility,
    into: TokenStream,
    from: TokenStream,
}

impl Member {
    fn new(
        base_ty: syn::Type,
        base_bits: usize,
        f: syn::Field,
        offset: usize,
        order: Order,
    ) -> syn::Result<Self> {
        let span = f.span();

        let syn::Field {
            mut attrs,
            vis,
            ident,
            ty,
            ..
        } = f;

        let ident = ident.ok_or_else(|| syn::Error::new(span, "Not supported"))?;
        let ignore = ident.to_string().starts_with('_');

        let Field {
            bits,
            ty,
            mut default,
            into,
            from,
        } = parse_field(&attrs, &ty, ignore)?;

        if bits > 0 && !ignore {
            if offset + bits > base_bits {
                return Err(syn::Error::new(
                    ty.span(),
                    "The total size of the members is too large!",
                ));
            };

            // compute the offset
            let offset = if order == Order::Lsb {
                offset
            } else {
                base_bits - offset - bits
            };

            if into.is_empty() || from.is_empty() {
                return Err(syn::Error::new(
                    ty.span(),
                    "Custom types require 'into', and 'from' in the #[bits] attribute",
                ));
            }

            if default.is_empty() {
                default = quote!(#ty::from_bits(0));
            }

            // remove our attribute
            attrs.retain(|a| !a.path().is_ident("bits"));

            Ok(Self {
                offset,
                bits,
                base_ty,
                default,
                inner: Some(MemberInner {
                    ident,
                    ty,
                    attrs,
                    vis,
                    into,
                    from,
                }),
            })
        } else {
            if default.is_empty() {
                default = quote!(0);
            }

            Ok(Self {
                offset,
                bits,
                base_ty,
                default,
                inner: None,
            })
        }
    }

    fn debug(&self) -> TokenStream {
        if let Some(inner) = &self.inner {
            let ident_str = inner.ident.to_string();
            let ident = &inner.ident;
            quote!(.field(#ident_str, &self.#ident()))
        } else {
            quote!()
        }
    }

    fn default(&self) -> TokenStream {
        let default = &self.default;
        if let Some(inner) = &self.inner {
            let ident = &inner.ident;
            let with_ident = format_ident!("with_{ident}");
            quote!(this = this.#with_ident(#default);)
        } else {
            let offset = self.offset;
            let base_ty = &self.base_ty;
            quote!(this.0 |= (#default as #base_ty) << #offset;)
        }
    }
}

impl ToTokens for Member {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            offset,
            bits,
            base_ty,
            default: _,
            inner: Some(MemberInner { ident, ty, attrs, vis, into, from }),
        } = self else {
            return Default::default();
        };

        let ident_str = ident.to_string();

        let with_ident = format_ident!("with_{ident}");
        let set_ident = format_ident!("set_{ident}");
        let bits_ident = format_ident!("{}_BITS", ident_str.to_uppercase());
        let offset_ident = format_ident!("{}_OFFSET", ident_str.to_uppercase());

        let location = format!("\n\nBits: {offset}..{}", offset + bits);

        let doc: TokenStream = attrs
            .iter()
            .filter(|a| !a.path().is_ident("bits"))
            .map(ToTokens::to_token_stream)
            .collect();

        let mask = u128::MAX >> (u128::BITS - *bits as u32);
        let mask = syn::LitInt::new(&format!("0x{mask:x}"), Span::mixed_site());

        let code = quote! {
            const #bits_ident: usize = #bits;
            const #offset_ident: usize = #offset;

            #doc
            #[doc = #location]
            #[cfg_attr(debug_assertions, track_caller)]
            #vis const fn #with_ident(self, value: #ty) -> Self {
                let value: #base_ty = {
                    let this = value;
                    #into
                };
                #[allow(unused_comparisons)]
                debug_assert!(value <= #mask, "value out of bounds");
                Self(self.0 & !(#mask << #offset) | (value & #mask) << #offset)
            }
            #doc
            #[doc = #location]
            #vis const fn #ident(&self) -> #ty {
                let this = (self.0 >> #offset) & #mask;
                #from
            }
            #doc
            #[doc = #location]
            #[cfg_attr(debug_assertions, track_caller)]
            #vis fn #set_ident(&mut self, value: #ty) {
                *self = self.#with_ident(value);
            }

        };
        tokens.extend(code);
    }
}

/// Distinguish between different types for code generation.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum TypeClass {
    /// Booleans with 1 bit size
    Bool,
    /// Unsigned ints with fixes sizes: u8, u64, ...
    UInt,
    /// Signed ints with fixes sizes: i8, i64, ...
    SInt,
    /// Custom types
    Other,
}

/// Field information, including the `bits` attribute
struct Field {
    bits: usize,
    ty: syn::Type,

    default: TokenStream,
    into: TokenStream,
    from: TokenStream,
}

/// Parses the `bits` attribute that allows specifying a custom number of bits.
fn parse_field(attrs: &[syn::Attribute], ty: &syn::Type, ignore: bool) -> syn::Result<Field> {
    fn malformed(mut e: syn::Error, attr: &syn::Attribute) -> syn::Error {
        e.combine(syn::Error::new(attr.span(), "malformed #[bits] attribute"));
        e
    }

    // Defaults for the different types
    let (class, ty_bits) = type_bits(ty);
    let mut ret = match class {
        TypeClass::Bool => Field {
            bits: ty_bits,
            ty: ty.clone(),
            default: quote!(false),
            into: quote!(this as _),
            from: quote!(this != 0),
        },
        TypeClass::SInt => Field {
            bits: ty_bits,
            ty: ty.clone(),
            default: quote!(0),
            into: TokenStream::new(),
            from: TokenStream::new(),
        },
        TypeClass::UInt => Field {
            bits: ty_bits,
            ty: ty.clone(),
            default: quote!(0),
            into: quote!(this as _),
            from: quote!(this as _),
        },
        TypeClass::Other => Field {
            bits: ty_bits,
            ty: ty.clone(),
            default: TokenStream::new(),
            into: quote!(#ty::into_bits(this)),
            from: quote!(#ty::from_bits(this)),
        },
    };

    // Find and parse the bits attribute
    for attr in attrs {
        let syn::Attribute {
                style: syn::AttrStyle::Outer,
                meta: syn::Meta::List(syn::MetaList { path, tokens, .. }),
                ..
        } = attr else {
            continue
        };
        if path.is_ident("bits") {
            let span = tokens.span();
            let BitsAttr {
                bits,
                default,
                into,
                from,
            } = syn::parse2(tokens.clone()).map_err(|e| malformed(e, attr))?;

            if let Some(bits) = bits {
                if bits == 0 {
                    return Err(syn::Error::new(span, "bits cannot bit 0"));
                }
                if ty_bits != 0 && bits > ty_bits {
                    return Err(syn::Error::new(span, "overflowing field type"));
                }
                ret.bits = bits;
            }
            if ignore && (into.is_some() || from.is_some()) {
                return Err(syn::Error::new(
                    default.span(),
                    "'into' and 'from' are not supported on padding",
                ));
            }

            if let Some(into) = into {
                ret.into = quote!(#into(this));
            }
            if let Some(from) = from {
                // Auto-conversion from zero
                if default.is_none() {
                    ret.default = quote!(#from(0));
                }

                ret.from = quote!(#from(this));
            }
            if let Some(default) = default {
                ret.default = default.into_token_stream();
            }
        }
    }

    if ret.bits == 0 {
        return Err(syn::Error::new(
            ty.span(),
            "Custom types and isize/usize require the size in the #[bits] attribute",
        ));
    }

    // Signed integers need some special handling...
    if !ignore && class == TypeClass::SInt {
        let bits = ret.bits as u32;
        let mask = u128::MAX >> (u128::BITS - ret.bits as u32);
        let mask = syn::LitInt::new(&format!("0x{mask:x}"), Span::mixed_site());
        if ret.into.is_empty() {
            // Bounds check and remove leading ones from negative values
            ret.into = quote! {{
                #[allow(unused_comparisons)]
                debug_assert!(if this >= 0 { this & !#mask == 0 } else { !this & !#mask == 0 }, "value out of bounds");
                (this & #mask) as _
            }};
        }
        if ret.from.is_empty() {
            // Sign extend negative values
            ret.from = quote! {{
                let shift = #ty::BITS - #bits;
                ((this as #ty) << shift) >> shift
            }};
        }
    }

    Ok(ret)
}

/// The bits attribute of the fields of a bitfield struct
struct BitsAttr {
    bits: Option<usize>,
    default: Option<syn::Expr>,
    into: Option<syn::Path>,
    from: Option<syn::Path>,
}

impl Parse for BitsAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attr = Self {
            bits: None,
            default: None,
            into: None,
            from: None,
        };
        if let Ok(bits) = syn::LitInt::parse(input) {
            attr.bits = Some(bits.base10_parse()?);
            if !input.is_empty() {
                <Token![,]>::parse(input)?;
            }
        }
        // parse remainder
        if !input.is_empty() {
            loop {
                let ident = syn::Ident::parse(input)?;

                <Token![=]>::parse(input)?;

                if ident == "default" {
                    attr.default = Some(input.parse()?);
                } else if ident == "into" {
                    attr.into = Some(input.parse()?);
                } else if ident == "from" {
                    attr.from = Some(input.parse()?);
                }

                if input.is_empty() {
                    break;
                }

                <Token![,]>::parse(input)?;
            }
        }
        Ok(attr)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Order {
    Lsb,
    Msb,
}

/// The bitfield macro parameters
struct Params {
    ty: syn::Type,
    bits: usize,
    debug: bool,
    default: bool,
    order: Order,
}

impl Parse for Params {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let Ok(ty) = syn::Type::parse(input) else {
            return Err(syn::Error::new(input.span(), "unknown type"));
        };
        let (class, bits) = type_bits(&ty);
        if class != TypeClass::UInt {
            return Err(syn::Error::new(input.span(), "unsupported type"));
        }

        let mut debug = true;
        let mut default = true;
        let mut order = Order::Lsb;

        // try parse additional args
        while <Token![,]>::parse(input).is_ok() {
            let ident = Ident::parse(input)?;
            <Token![=]>::parse(input)?;
            match ident.to_string().as_str() {
                "debug" => {
                    let value = syn::LitBool::parse(input)?.value;
                    debug = value;
                }
                "default" => {
                    let value = syn::LitBool::parse(input)?.value;
                    default = value;
                }
                "order" => {
                    let value = match syn::Ident::parse(input)?.to_string().as_str() {
                        "Msb" | "msb" => Order::Msb,
                        "Lsb" | "lsb" => Order::Lsb,
                        _ => return Err(syn::Error::new(ident.span(), "unknown value for order")),
                    };
                    order = value;
                }
                _ => return Err(syn::Error::new(ident.span(), "unknown argument")),
            };
        }

        Ok(Params {
            ty,
            bits,
            debug,
            default,
            order,
        })
    }
}

/// Returns the number of bits for a given type
fn type_bits(ty: &syn::Type) -> (TypeClass, usize) {
    let syn::Type::Path(syn::TypePath{ path, .. }) = ty else {
        return (TypeClass::Other, 0);
    };
    let Some(ident) = path.get_ident() else {
        return (TypeClass::Other, 0);
    };
    if ident == "bool" {
        return (TypeClass::Bool, 1);
    }
    if ident == "isize" || ident == "usize" {
        return (TypeClass::UInt, 0); // they have architecture dependend sizes
    }
    macro_rules! integer {
        ($ident:ident => $($uint:ident),* ; $($sint:ident),*) => {
            match ident {
                $(_ if ident == stringify!($uint) => (TypeClass::UInt, $uint::BITS as _),)*
                $(_ if ident == stringify!($sint) => (TypeClass::SInt, $sint::BITS as _),)*
                _ => (TypeClass::Other, 0)
            }
        };
    }
    integer!(ident => u8, u16, u32, u64, u128 ; i8, i16, i32, i64, i128)
}

#[cfg(test)]
mod test {
    use quote::quote;

    use crate::{BitsAttr, Order, Params};

    #[test]
    fn parse_args() {
        let args = quote!(u64);
        let params = syn::parse2::<Params>(args).unwrap();
        assert!(params.bits == u64::BITS as usize && params.debug == true);

        let args = quote!(u32, debug = false);
        let params = syn::parse2::<Params>(args).unwrap();
        assert!(params.bits == u32::BITS as usize && params.debug == false);

        let args = quote!(u32, order = Msb);
        let params = syn::parse2::<Params>(args).unwrap();
        assert!(params.bits == u32::BITS as usize && params.order == Order::Msb);
    }

    #[test]
    fn parse_bits() {
        let args = quote!(8);
        let attr = syn::parse2::<BitsAttr>(args).unwrap();
        assert_eq!(attr.bits, Some(8));
        assert!(attr.default.is_none());
        assert!(attr.into.is_none());
        assert!(attr.from.is_none());

        let args = quote!(8, default = 8);
        let attr = syn::parse2::<BitsAttr>(args).unwrap();
        assert_eq!(attr.bits, Some(8));
        assert!(attr.default.is_some());
        assert!(attr.into.is_none());
        assert!(attr.from.is_none());

        let args = quote!(default = 8);
        let attr = syn::parse2::<BitsAttr>(args).unwrap();
        assert_eq!(attr.bits, None);
        assert!(attr.default.is_some());
        assert!(attr.into.is_none());
        assert!(attr.from.is_none());

        let args = quote!(3, into = into_something, default = 1, from = from_something);
        let attr = syn::parse2::<BitsAttr>(args).unwrap();
        assert_eq!(attr.bits, Some(3));
        assert!(attr.default.is_some());
        assert!(attr.into.is_some());
        assert!(attr.from.is_some());
    }
}
