// Automatically generated file. Do not edit.
// Generated by build-resource-byte-arrays crate.

#[cfg(feature = "build_resource_byte_arrays")]
#[doc = "This file path is automatically generated during compilation and points to a build-time resource (generated pre-compilation by `build.rs`)."]
#[doc = "It reflects the state of the resource at the time of compilation, which may not match the source code or file system in subsequent builds."]
pub static COMPRESSED_COMPANY_SYMBOL_LIST_BYTE_ARRAY: &[u8] = include_bytes!("../target/debug/build/ticker-sniffer-82f0d34d30344691/out/bin/COMPRESSED_COMPANY_SYMBOL_LIST_BYTE_ARRAY.bin");

#[cfg(not(feature = "build_resource_byte_arrays"))]
pub static COMPRESSED_COMPANY_SYMBOL_LIST_BYTE_ARRAY: &[u8] = &[];

#[cfg(not(feature = "build_resource_byte_arrays"))]
#[ctor::ctor]
fn warn_COMPRESSED_COMPANY_SYMBOL_LIST_BYTE_ARRAY_empty() {
    eprintln!("Warning: `COMPRESSED_COMPANY_SYMBOL_LIST_BYTE_ARRAY` is empty because the `build_resource_byte_arrays ` feature is not enabled.");
}

