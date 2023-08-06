#![feature(result_option_inspect)]
#![feature(doc_cfg)]

pub use anyhow::{anyhow, Ok, Result};

#[cfg(feature = "dioxus")]
#[doc(cfg(dioxus))]
pub mod dioxus;
pub mod enums;

#[cfg(feature = "google")]
#[doc(cfg(google))]
pub mod google;

#[cfg(feature = "gpt")]
#[doc(cfg(gpt))]
pub mod gpt;

pub mod json;

#[cfg(feature = "process")]
#[doc(cfg(process))]
pub mod process;

#[cfg(feature = "streams")]
#[doc(cfg(streams))]
pub mod streams;

#[cfg(feature = "teloxide")]
#[doc(cfg(teloxide))]
pub mod teloxide;

#[cfg(feature = "vector_embeddings")]
#[doc(cfg(vector_embeddings))]
pub mod vector_embeddings;
