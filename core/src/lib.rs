#![warn(
    clippy::std_instead_of_core,
    clippy::derive_partial_eq_without_eq,
    clippy::match_same_arms,
    clippy::same_name_method,
    clippy::unwrap_used,
    clippy::redundant_clone,
    clippy::manual_async_fn,
    clippy::missing_panics_doc,
    clippy::use_self,
    clippy::single_char_lifetime_names,
    clippy::missing_const_for_fn,
    clippy::impl_trait_in_params
)]

pub mod common;
pub mod core;
