//! Cloud transcription (cloudimplementation §7): the route, its provider,
//! its audio budget, and what it reads of an Ogg file. Nothing here writes
//! to the database (P-16).

pub mod budget;
pub mod ogg;
pub mod provider;
pub mod route;
