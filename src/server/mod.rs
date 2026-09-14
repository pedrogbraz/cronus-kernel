#![allow(dead_code, unused_imports, unused_variables)]
//! CRONUS HTTP server submodules.
//!
//! The live dispatcher is `main.rs::handle_request` → `handle_request_inner`.

pub(crate) mod auth_pages;
pub(crate) mod docs;
pub(crate) mod docs_index;
pub(crate) mod response;
pub(crate) mod state;
