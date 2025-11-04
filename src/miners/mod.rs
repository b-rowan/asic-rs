//! Miner types and selection
//!
//! For miner type selection and scanning, see [`MinerFactory`][`factory::MinerFactory`].
//!
//! For traits implemented by each miner, see [`GetMinerData`][`backends::traits::GetMinerData`] for data gathering,
//! and [`HasMinerControl`][`backends::traits::HasMinerControl`].
//! These traits are unified by the [`Miner`][`backends::traits::Miner`] trait.
//!
//! Per-miner implementations are under [`backends`][`backends`] in their own modules.

pub mod api;
pub mod backends;
pub mod commands;
pub mod data;
pub mod factory;
pub mod listener;
pub(crate) mod util;
