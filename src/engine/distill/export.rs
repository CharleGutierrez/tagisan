//! JSONL exporter for fine-tuning datasets compatible with Unsloth, LLaMA-Factory, and HuggingFace TRL.

use super::format::DistillationFormat;
use super::distiller::ReflexionDistiller;
use crate::error::{Result, TagisanError};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Exports distilled datasets to JSONL file on disk
pub struct DatasetExporter;

impl DatasetExporter {
    pub fn export_to_jsonl<P: AsRef<Path>>(
        distiller: &ReflexionDistiller,
        format: DistillationFormat,
        output_path: P,
    ) -> Result<usize> {
        let file = File::create(output_path).map_err(TagisanError::Io)?;
        let mut writer = std::io::BufWriter::new(file);
        let mut count = 0;

        match format {
            DistillationFormat::Alpaca => {
                for record in distiller.to_alpaca() {
                    let line = serde_json::to_string(&record).map_err(TagisanError::Serialization)?;
                    writeln!(writer, "{}", line).map_err(TagisanError::Io)?;
                    count += 1;
                }
            }
            DistillationFormat::ShareGpt => {
                for record in distiller.to_sharegpt() {
                    let line = serde_json::to_string(&record).map_err(TagisanError::Serialization)?;
                    writeln!(writer, "{}", line).map_err(TagisanError::Io)?;
                    count += 1;
                }
            }
            DistillationFormat::Dpo => {
                for record in distiller.to_dpo() {
                    let line = serde_json::to_string(&record).map_err(TagisanError::Serialization)?;
                    writeln!(writer, "{}", line).map_err(TagisanError::Io)?;
                    count += 1;
                }
            }
        }

        writer.flush().map_err(TagisanError::Io)?;
        Ok(count)
    }
}
