//! Where files go on disk: path templates, sanitization, atomic move with
//! cross-filesystem fallback. No DB access; this crate only knows the
//! filesystem.
//!
//! See TDD 03 §Stage 5 and TDD 11 §On-disk layout for the contracts.
