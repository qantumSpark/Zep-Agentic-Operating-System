//! ZAOS Event System.
//!
//! Each runtime provider has its own mapper module (`claude_mapper`, etc.)
//! that converts provider-native events into the unified `ZaosEvent` model.
//! The `types` module contains Claude-specific wire types (`CliEvent`).
//! The `zaos_events` module defines the provider-neutral event enum.

pub mod claude_mapper;
pub mod parser;
pub mod types;
pub mod zaos_events;

