// "Tifflin" Kernel
// - By John Hodge (thePowersGang)
//
// Modules/lib_utf16/lib.rs
//! UTF-16 string support
#![feature(no_std,core)]
#![no_std]
#[macro_use] extern crate core;
#[macro_use] extern crate kernel;
use kernel::prelude::*;

use kernel::lib::byte_str::ByteStr;
use core::cmp;

pub struct Str16([u16]);

impl Str16
{
	pub fn new(v: &[u16]) -> Option<&Str16> {
		// 1. Validate that the passed array is valid UTF-16
		// 2. Create return
		Some( unsafe { Self::new_unchecked(v) } )
	}
	/// Create a new UTF-16 string without any validity checking
	pub unsafe fn new_unchecked(v: &[u16]) -> &Str16 {
		::core::mem::transmute(v)
	}
	
	/// Returns an iterator of unicode codepoints
	pub fn chars<'a>(&'a self) -> Chars<'a> {
		Chars(&self.0)
	}
	/// An iterator that returns a series of WTF-8 codepoints (same encoding as
	/// UTF-8, but invalid codepoints may be generated)
	pub fn wtf8<'a>(&'a self) -> Wtf8<'a> {
		Wtf8(self.chars(), [0; 4])
	}
}

impl_fmt! {
	Debug(self,f) for Str16 {{
		try!(write!(f, "w\""));
		for c in self.chars()
		{
			match c
			{
			'\\' => try!(write!(f, "\\\\")),
			'\n' => try!(write!(f, "\\n")),
			'\r' => try!(write!(f, "\\r")),
			'"' => try!(write!(f, "\\\"")),
			'\0' => try!(write!(f, "\\0")),
			// ASCII printable characters
			' '...'\u{127}' => try!(write!(f, "{}", c)),
			_ => try!(write!(f, "\\u{{{:x}}}", c as u32)),
			}
		}
		try!(write!(f, "\""));
		Ok( () )
	}}
	Display(self,f) for Str16 {{
		for c in self.chars()
		{
			try!(write!(f, "{}", c));
		}
		Ok( () )
	}}
}

impl cmp::PartialOrd<ByteStr> for Str16 {
	fn partial_cmp(&self, v: &ByteStr) -> Option<::core::cmp::Ordering> {
		::core::iter::order::partial_cmp( self.wtf8(), v.as_bytes().iter().cloned() )
	}
}
impl cmp::PartialEq<ByteStr> for Str16
{
	fn eq(&self, v: &ByteStr) -> bool {
		match self.partial_cmp(v)
		{
		Some(cmp::Ordering::Equal) => true,
		_ => false,
		}
	}
}
impl cmp::PartialOrd<str> for Str16 {
	fn partial_cmp(&self, v: &str) -> Option<::core::cmp::Ordering> {
		::core::iter::order::partial_cmp( self.chars(), v.chars() )
	}
}
impl cmp::PartialEq<str> for Str16
{
	fn eq(&self, v: &str) -> bool {
		match self.partial_cmp(v)
		{
		Some(cmp::Ordering::Equal) => true,
		_ => false,
		}
	}
}

pub struct Wtf8<'a>(Chars<'a>, [u8; 4]);
impl<'a> ::core::iter::Iterator for Wtf8<'a>
{
	type Item = u8;
	fn next(&mut self) -> Option<u8>
	{
		if self.1[0] == 0 {
			match self.0.next()
			{
			None => return None,
			// no real need to check length. 4 is sufficient, and NUL termination is maintained
			Some(c) => {c.encode_utf8(&mut self.1).unwrap();},
			}
		}
		let rv = self.1[0];
		for i in 0..3 {
			self.1[i] = self.1[i+1];
		}
		self.1[3] = 0;
		Some(rv)
	}
}

pub struct Chars<'a>(&'a [u16]);
impl<'a> ::core::iter::Iterator for Chars<'a>
{
	type Item = char;
	fn next(&mut self) -> Option<char>
	{
		let (cp,n) = match self.0.get(0).cloned()
			{
			None => return None,
			// High surrogate
			Some(v @ 0xD800 ... 0xDBFF) =>
				match self.0.get(1).cloned()
				{
				// - Surrogate pair
				Some(low @ 0xDC00 ... 0xDFFF) => {
					let high = v as u32 - 0xD800;
					let low = low as u32 - 0xDC00;
					let cp: u32 = 0x10000 + high << 10 + low;
					(cp, 2)
					},
				// - Lone surrogate, semi-standard response is to return it.
				_ => (v as u32, 1),
				},
			// - Lone low surrogate, use semi-standard behavior
			Some(v @ 0xDC00 ... 0xDFFF) => (v as u32, 1),
			// - Pure codepoint
			Some(v) => (v as u32, 1),
			};
		self.0 = &self.0[n..];
		Some(::core::char::from_u32(cp).expect("UTF-16 decode error"))
	}
}
