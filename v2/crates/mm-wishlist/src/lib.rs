//! Wishlist worker: drives the state machine
//! `PENDING → SEARCHING → DOWNLOADING → CHECKING → IMPORTING → COMPLETED/FAILED`,
//! coordinating with `song-rs` for soulseek and `mm-import` for the final import.
