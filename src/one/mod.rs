//! OpenNebula API interaction module
//!
//! This module provides the client and authentication for OpenNebula's XML-RPC API.

pub mod auth;
pub mod client;
pub mod xmlrpc;

pub use client::OneClient;
