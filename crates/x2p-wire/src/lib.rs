//! Canonical JSON parser and pretty-printer for the x2p wire format.
//!
//! There is exactly one source of truth for the canonical wire form so that
//! `x2p-core`, `x2p-web-adapter-host`, and the browser extension all agree
//! byte-for-byte (Req. 23.1, 23.2). This crate will host that implementation;
//! it is currently a skeleton populated by later tasks.
