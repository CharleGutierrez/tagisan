//! Automated Prompt Distillation & LoRA Dataset Generator for Tagisan (TGS).
//!
//! Converts episodic case law (.tagisan/reflexions.json) and agent traces
//! into Alpaca, ShareGPT, and DPO fine-tuning datasets for local QLoRA fine-tuning.

pub mod format;
pub mod distiller;
pub mod export;

pub use format::{
    AlpacaRecord, DistillationFormat, DpoRecord, ShareGptMessage, ShareGptRecord,
};
pub use distiller::ReflexionDistiller;
pub use export::DatasetExporter;
