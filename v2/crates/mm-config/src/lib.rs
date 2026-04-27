//! Env-var loader for music-manager.
//!
//! Single entry point: [`Config::from_env`]. No other crate is allowed to read
//! `std::env::var` directly — this is enforced by `scripts/check-env-source.sh`
//! in CI (per conventions §3).
