//! aelm-mcp — Model Context Protocol server for the Aelm circuit-CAD toolchain.
//!
//! Speaks MCP over stdio and fulfils every request by shelling out to the
//! `aelm` CLI, so the server stays small and the Aelm core stays independent.

pub mod cli_runner;
pub mod error;
pub mod prompts;
pub mod resources;
pub mod server;
pub mod tools;
