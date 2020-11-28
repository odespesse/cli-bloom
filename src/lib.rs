//! # cli-bloom
//!
//! The `cli-bloom` crate provide a convenient way to ingest files in an `index-bloom`.
//!
//! Ingest text from a file or from all files in a directory (not recursively). All files must contains only valid UTF-8 characters.
//! When ingestion is done, it is possible to dump the index content in JSON format and to restore it later.
//!
//! # Quick start
//!
//! ```rust
//! use cli_bloom::FsIndex;
//!
//! # fn search_index() {
//! let mut fs_index = FsIndex::new(0.00001);
//! fs_index.ingest("/foo/bar");
//! let hits = fs_index.search("content");
//! println!("{:?}", hits.unwrap());
//! # }
//! ```
//!
//! # Ingest and save for later
//!
//! ```rust
//! use cli_bloom::FsIndex;
//!
//! # fn search_index() {
//! let mut fs_index = FsIndex::new(0.00001);
//! fs_index.ingest("/foo/bar");
//! fs_index.dump("/foo/dump.json");
//! # }
//! ```
//!
//! # Ingest more files
//!
//! ```rust
//! use cli_bloom::FsIndex;
//!
//! # fn search_index() {
//! let mut fs_index = FsIndex::restore("/foo/dump.json");
//! fs_index.ingest("/more/files");
//! fs_index.dump("/foo/dump.json");
//! # }
//! ```

mod fs_loader;
pub use fs_loader::FsIndex;

mod errors;
