use crate::error::{Result, TagisanError};
use memmap2::Mmap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub const GGUF_MAGIC: [u8; 4] = [b'G', b'G', b'U', b'F'];
pub const GGUF_VERSION_2: u32 = 2;
pub const GGUF_VERSION_3: u32 = 3;
pub const DEFAULT_ALIGNMENT: u64 = 32;

/// GGUF Value Types according to specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u32)]
pub enum GgufValueType {
    Uint8 = 0,
    Int8 = 1,
    Uint16 = 2,
    Int16 = 3,
    Uint32 = 4,
    Int32 = 5,
    Float32 = 6,
    Bool = 7,
    String = 8,
    Array = 9,
    Uint64 = 10,
    Int64 = 11,
    Float64 = 12,
}

impl GgufValueType {
    pub fn from_u32(val: u32) -> Result<Self> {
        match val {
            0 => Ok(Self::Uint8),
            1 => Ok(Self::Int8),
            2 => Ok(Self::Uint16),
            3 => Ok(Self::Int16),
            4 => Ok(Self::Uint32),
            5 => Ok(Self::Int32),
            6 => Ok(Self::Float32),
            7 => Ok(Self::Bool),
            8 => Ok(Self::String),
            9 => Ok(Self::Array),
            10 => Ok(Self::Uint64),
            11 => Ok(Self::Int64),
            12 => Ok(Self::Float64),
            other => Err(TagisanError::Execution(format!(
                "Unknown GGUF value type ID: {other}"
            ))),
        }
    }
}

/// Strongly typed GGUF Value representation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GgufValue {
    Uint8(u8),
    Int8(i8),
    Uint16(u16),
    Int16(i16),
    Uint32(u32),
    Int32(i32),
    Float32(f32),
    Bool(bool),
    String(String),
    Array(Vec<GgufValue>),
    Uint64(u64),
    Int64(i64),
    Float64(f64),
}

impl GgufValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Uint8(v) => Some(*v as u64),
            Self::Uint16(v) => Some(*v as u64),
            Self::Uint32(v) => Some(*v as u64),
            Self::Uint64(v) => Some(*v),
            Self::Int8(v) if *v >= 0 => Some(*v as u64),
            Self::Int16(v) if *v >= 0 => Some(*v as u64),
            Self::Int32(v) if *v >= 0 => Some(*v as u64),
            Self::Int64(v) if *v >= 0 => Some(*v as u64),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int8(v) => Some(*v as i64),
            Self::Int16(v) => Some(*v as i64),
            Self::Int32(v) => Some(*v as i64),
            Self::Int64(v) => Some(*v),
            Self::Uint8(v) => Some(*v as i64),
            Self::Uint16(v) => Some(*v as i64),
            Self::Uint32(v) => Some(*v as i64),
            Self::Uint64(v) => i64::try_from(*v).ok(),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float32(v) => Some(*v as f64),
            Self::Float64(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[GgufValue]> {
        match self {
            Self::Array(a) => Some(a.as_slice()),
            _ => None,
        }
    }

    pub fn to_display_string(&self) -> String {
        match self {
            Self::Uint8(v) => v.to_string(),
            Self::Int8(v) => v.to_string(),
            Self::Uint16(v) => v.to_string(),
            Self::Int16(v) => v.to_string(),
            Self::Uint32(v) => v.to_string(),
            Self::Int32(v) => v.to_string(),
            Self::Float32(v) => v.to_string(),
            Self::Bool(v) => v.to_string(),
            Self::String(v) => v.clone(),
            Self::Uint64(v) => v.to_string(),
            Self::Int64(v) => v.to_string(),
            Self::Float64(v) => v.to_string(),
            Self::Array(a) => {
                if a.len() <= 6 {
                    format!("[{}]", a.iter().map(|e| e.to_display_string()).collect::<Vec<_>>().join(", "))
                } else {
                    format!("[Array with {} elements]", a.len())
                }
            }
        }
    }
}

/// Metadata and layout info for an individual tensor in the GGUF container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GgufTensorInfo {
    pub name: String,
    pub n_dims: u32,
    pub dims: Vec<u64>,
    pub tensor_type: u32,
    pub offset: u64,
    pub size_bytes: u64,
}

impl GgufTensorInfo {
    pub fn type_name(&self) -> &'static str {
        match self.tensor_type {
            0 => "F32",
            1 => "F16",
            2 => "Q4_0",
            3 => "Q4_1",
            6 => "Q5_0",
            7 => "Q5_1",
            8 => "Q8_0",
            9 => "Q8_1",
            10 => "Q2_K",
            11 => "Q3_K",
            12 => "Q4_K",
            13 => "Q5_K",
            14 => "Q6_K",
            15 => "Q8_K",
            16 => "IQ2_XXS",
            17 => "IQ2_XS",
            18 => "IQ3_XXS",
            19 => "IQ1_S",
            20 => "IQ4_NL",
            21 => "IQ3_S",
            22 => "IQ2_S",
            23 => "IQ4_XS",
            24 => "I8",
            25 => "I16",
            26 => "I32",
            27 => "I64",
            28 => "F64",
            29 => "IQ1_M",
            30 => "BF16",
            _ => "UNKNOWN",
        }
    }

    pub fn element_count(&self) -> u64 {
        if self.dims.is_empty() {
            0
        } else {
            self.dims.iter().product()
        }
    }
}

/// Parsed high-level model metadata and raw key-value store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GgufMetadata {
    pub architecture: String,
    pub context_length: Option<u64>,
    pub embedding_length: Option<u64>,
    pub block_count: Option<u64>,
    pub head_count: Option<u64>,
    pub head_count_kv: Option<u64>,
    pub chat_template: Option<String>,
    pub vocab_size: Option<u64>,
    pub raw_kv: HashMap<String, GgufValue>,
}

impl GgufMetadata {
    pub fn get(&self, key: &str) -> Option<&GgufValue> {
        self.raw_kv.get(key)
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.raw_kv.get(key).and_then(|v| v.as_str())
    }

    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.raw_kv.get(key).and_then(|v| v.as_u64())
    }
}

/// Zero-copy memory mapped GGUF file
pub struct GgufFile {
    pub path: PathBuf,
    pub version: u32,
    pub tensor_count: u64,
    pub metadata_kv_count: u64,
    pub metadata: GgufMetadata,
    pub tensors: Vec<GgufTensorInfo>,
    pub tensor_data_offset: u64,
    mmap: Arc<Mmap>,
}

impl GgufFile {
    /// Open and parse a GGUF file using zero-copy memory mapping
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path_buf = path.as_ref().to_path_buf();
        let file = File::open(&path_buf).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to open GGUF file at '{}': {e}",
                path_buf.display()
            ))
        })?;

        let mmap = unsafe {
            Mmap::map(&file).map_err(|e| {
                TagisanError::Execution(format!(
                    "Failed to memory map GGUF file at '{}': {e}",
                    path_buf.display()
                ))
            })?
        };

        let data = mmap.as_ref();
        if data.len() < 24 {
            return Err(TagisanError::Execution(
                "File too small to be a valid GGUF container (< 24 bytes)".to_string(),
            ));
        }

        // 1. Magic
        if &data[0..4] != GGUF_MAGIC {
            return Err(TagisanError::Execution(format!(
                "Invalid GGUF magic: expected {:?}, got {:?}",
                GGUF_MAGIC,
                &data[0..4]
            )));
        }

        // 2. Version
        let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
        if version != GGUF_VERSION_2 && version != GGUF_VERSION_3 {
            return Err(TagisanError::Execution(format!(
                "Unsupported GGUF version: {version}. Only v2 and v3 are supported."
            )));
        }

        // 3. Tensor count & KV count (both uint64 in v2 & v3)
        let tensor_count = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let metadata_kv_count = u64::from_le_bytes(data[16..24].try_into().unwrap());

        let mut cursor = 24usize;
        let mut raw_kv = HashMap::new();

        // 4. Parse Metadata KV pairs
        for _ in 0..metadata_kv_count {
            let (key, new_cursor) = Self::read_string(data, cursor)?;
            cursor = new_cursor;

            if cursor + 4 > data.len() {
                return Err(TagisanError::Execution(
                    "Unexpected EOF reading metadata value type".to_string(),
                ));
            }
            let val_type_id = u32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap());
            cursor += 4;
            let val_type = GgufValueType::from_u32(val_type_id)?;

            let (val, new_cursor) = Self::read_value(data, cursor, val_type)?;
            cursor = new_cursor;
            raw_kv.insert(key, val);
        }

        // Alignment lookup
        let alignment = raw_kv
            .get("general.alignment")
            .and_then(|v| v.as_u64())
            .unwrap_or(DEFAULT_ALIGNMENT);

        // 5. Parse Tensor Info
        let mut tensors = Vec::with_capacity(tensor_count as usize);
        for _ in 0..tensor_count {
            let (name, new_cursor) = Self::read_string(data, cursor)?;
            cursor = new_cursor;

            if cursor + 4 > data.len() {
                return Err(TagisanError::Execution(
                    "Unexpected EOF reading tensor dimensions count".to_string(),
                ));
            }
            let n_dims = u32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap());
            cursor += 4;

            let dims_bytes_len = (n_dims as usize) * 8;
            if cursor + dims_bytes_len + 12 > data.len() {
                return Err(TagisanError::Execution(
                    "Unexpected EOF reading tensor dimensions and offset".to_string(),
                ));
            }

            let mut dims = Vec::with_capacity(n_dims as usize);
            for _ in 0..n_dims {
                let d = u64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap());
                dims.push(d);
                cursor += 8;
            }

            let tensor_type = u32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap());
            cursor += 4;

            let offset = u64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap());
            cursor += 8;

            tensors.push(GgufTensorInfo {
                name,
                n_dims,
                dims,
                tensor_type,
                offset,
                size_bytes: 0,
            });
        }

        // Tensor data start offset is aligned to `alignment` bytes
        let tensor_data_offset = ((cursor as u64) + (alignment - 1)) & !(alignment - 1);

        // Calculate tensor sizes based on element counts, types, or relative offset gaps
        let total_file_len = data.len() as u64;
        let tensor_count_usize = tensors.len();
        for i in 0..tensor_count_usize {
            let tensor_type = tensors[i].tensor_type;
            let elems = tensors[i].element_count();
            let estimated_size = Self::estimate_tensor_bytes(tensor_type, elems);

            // Bounded check against next tensor offset or file length
            let max_possible = if i + 1 < tensor_count_usize {
                tensors[i + 1].offset.saturating_sub(tensors[i].offset)
            } else {
                total_file_len.saturating_sub(tensor_data_offset + tensors[i].offset)
            };

            tensors[i].size_bytes = if estimated_size > 0 && estimated_size <= max_possible {
                estimated_size
            } else {
                max_possible
            };
        }

        // 6. Assemble GgufMetadata
        let architecture = raw_kv
            .get("general.architecture")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let context_length = raw_kv
            .get(&format!("{architecture}.context_length"))
            .or_else(|| raw_kv.get("general.context_length"))
            .and_then(|v| v.as_u64());

        let embedding_length = raw_kv
            .get(&format!("{architecture}.embedding_length"))
            .or_else(|| raw_kv.get("general.embedding_length"))
            .and_then(|v| v.as_u64());

        let block_count = raw_kv
            .get(&format!("{architecture}.block_count"))
            .or_else(|| raw_kv.get("general.block_count"))
            .and_then(|v| v.as_u64());

        let head_count = raw_kv
            .get(&format!("{architecture}.attention.head_count"))
            .or_else(|| raw_kv.get("general.head_count"))
            .and_then(|v| v.as_u64());

        let head_count_kv = raw_kv
            .get(&format!("{architecture}.attention.head_count_kv"))
            .and_then(|v| v.as_u64());

        let chat_template = raw_kv
            .get("tokenizer.chat_template")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let vocab_size = raw_kv
            .get("tokenizer.ggml.tokens")
            .and_then(|v| v.as_array())
            .map(|a| a.len() as u64)
            .or_else(|| {
                raw_kv
                    .get(&format!("{architecture}.vocab_size"))
                    .and_then(|v| v.as_u64())
            });

        let metadata = GgufMetadata {
            architecture,
            context_length,
            embedding_length,
            block_count,
            head_count,
            head_count_kv,
            chat_template,
            vocab_size,
            raw_kv,
        };

        Ok(Self {
            path: path_buf,
            version,
            tensor_count,
            metadata_kv_count,
            metadata,
            tensors,
            tensor_data_offset,
            mmap: Arc::new(mmap),
        })
    }

    /// Read a string (uint64 length + UTF-8 bytes)
    fn read_string(data: &[u8], cursor: usize) -> Result<(String, usize)> {
        if cursor + 8 > data.len() {
            return Err(TagisanError::Execution(
                "Unexpected EOF reading string length in GGUF".to_string(),
            ));
        }
        let len = u64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap()) as usize;
        let start = cursor + 8;
        let end = start + len;
        if end > data.len() {
            return Err(TagisanError::Execution(
                "Unexpected EOF reading string bytes in GGUF".to_string(),
            ));
        }
        let s = String::from_utf8_lossy(&data[start..end]).to_string();
        Ok((s, end))
    }

    /// Read a typed GGUF value from the binary buffer
    fn read_value(data: &[u8], cursor: usize, val_type: GgufValueType) -> Result<(GgufValue, usize)> {
        match val_type {
            GgufValueType::Uint8 => {
                if cursor + 1 > data.len() {
                    return Err(TagisanError::Execution("EOF reading Uint8".to_string()));
                }
                Ok((GgufValue::Uint8(data[cursor]), cursor + 1))
            }
            GgufValueType::Int8 => {
                if cursor + 1 > data.len() {
                    return Err(TagisanError::Execution("EOF reading Int8".to_string()));
                }
                Ok((GgufValue::Int8(data[cursor] as i8), cursor + 1))
            }
            GgufValueType::Uint16 => {
                if cursor + 2 > data.len() {
                    return Err(TagisanError::Execution("EOF reading Uint16".to_string()));
                }
                let val = u16::from_le_bytes(data[cursor..cursor + 2].try_into().unwrap());
                Ok((GgufValue::Uint16(val), cursor + 2))
            }
            GgufValueType::Int16 => {
                if cursor + 2 > data.len() {
                    return Err(TagisanError::Execution("EOF reading Int16".to_string()));
                }
                let val = i16::from_le_bytes(data[cursor..cursor + 2].try_into().unwrap());
                Ok((GgufValue::Int16(val), cursor + 2))
            }
            GgufValueType::Uint32 => {
                if cursor + 4 > data.len() {
                    return Err(TagisanError::Execution("EOF reading Uint32".to_string()));
                }
                let val = u32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap());
                Ok((GgufValue::Uint32(val), cursor + 4))
            }
            GgufValueType::Int32 => {
                if cursor + 4 > data.len() {
                    return Err(TagisanError::Execution("EOF reading Int32".to_string()));
                }
                let val = i32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap());
                Ok((GgufValue::Int32(val), cursor + 4))
            }
            GgufValueType::Float32 => {
                if cursor + 4 > data.len() {
                    return Err(TagisanError::Execution("EOF reading Float32".to_string()));
                }
                let val = f32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap());
                Ok((GgufValue::Float32(val), cursor + 4))
            }
            GgufValueType::Bool => {
                if cursor + 1 > data.len() {
                    return Err(TagisanError::Execution("EOF reading Bool".to_string()));
                }
                Ok((GgufValue::Bool(data[cursor] != 0), cursor + 1))
            }
            GgufValueType::String => {
                let (s, new_cursor) = Self::read_string(data, cursor)?;
                Ok((GgufValue::String(s), new_cursor))
            }
            GgufValueType::Array => {
                if cursor + 12 > data.len() {
                    return Err(TagisanError::Execution("EOF reading Array header".to_string()));
                }
                let elem_type_id = u32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap());
                let elem_count = u64::from_le_bytes(data[cursor + 4..cursor + 12].try_into().unwrap()) as usize;
                let mut current_cursor = cursor + 12;
                let elem_type = GgufValueType::from_u32(elem_type_id)?;

                let mut arr = Vec::with_capacity(elem_count.min(100_000));
                for _ in 0..elem_count {
                    let (val, next_cursor) = Self::read_value(data, current_cursor, elem_type)?;
                    arr.push(val);
                    current_cursor = next_cursor;
                }
                Ok((GgufValue::Array(arr), current_cursor))
            }
            GgufValueType::Uint64 => {
                if cursor + 8 > data.len() {
                    return Err(TagisanError::Execution("EOF reading Uint64".to_string()));
                }
                let val = u64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap());
                Ok((GgufValue::Uint64(val), cursor + 8))
            }
            GgufValueType::Int64 => {
                if cursor + 8 > data.len() {
                    return Err(TagisanError::Execution("EOF reading Int64".to_string()));
                }
                let val = i64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap());
                Ok((GgufValue::Int64(val), cursor + 8))
            }
            GgufValueType::Float64 => {
                if cursor + 8 > data.len() {
                    return Err(TagisanError::Execution("EOF reading Float64".to_string()));
                }
                let val = f64::from_le_bytes(data[cursor..cursor + 8].try_into().unwrap());
                Ok((GgufValue::Float64(val), cursor + 8))
            }
        }
    }

    /// Calculate expected byte size for standard GGML quant types
    fn estimate_tensor_bytes(tensor_type: u32, element_count: u64) -> u64 {
        match tensor_type {
            0 => element_count * 4,             // F32
            1 | 30 => element_count * 2,        // F16, BF16
            2 => (element_count / 32) * 18,     // Q4_0
            3 => (element_count / 32) * 20,     // Q4_1
            6 => (element_count / 32) * 22,     // Q5_0
            7 => (element_count / 32) * 24,     // Q5_1
            8 => (element_count / 32) * 34,     // Q8_0
            9 => (element_count / 32) * 40,     // Q8_1
            10 => (element_count / 256) * 84,   // Q2_K
            11 => (element_count / 256) * 110,  // Q3_K
            12 => (element_count / 256) * 144,  // Q4_K
            13 => (element_count / 256) * 176,  // Q5_K
            14 => (element_count / 256) * 210,  // Q6_K
            15 => (element_count / 256) * 292,  // Q8_K
            24 => element_count,                // I8
            25 => element_count * 2,            // I16
            26 => element_count * 4,            // I32
            27 | 28 => element_count * 8,       // I64, F64
            _ => 0,
        }
    }

    /// Retrieve zero-copy slice of a named tensor from the memory map
    pub fn tensor_slice(&self, name: &str) -> Result<&[u8]> {
        let t = self
            .tensors
            .iter()
            .find(|t| t.name == name)
            .ok_or_else(|| TagisanError::Execution(format!("Tensor not found: '{name}'")))?;

        let start = (self.tensor_data_offset + t.offset) as usize;
        let end = start + (t.size_bytes as usize);
        let mmap_len = self.mmap.len();

        if end <= mmap_len {
            Ok(&self.mmap[start..end])
        } else if start < mmap_len {
            Ok(&self.mmap[start..mmap_len])
        } else {
            Err(TagisanError::Execution(format!(
                "Tensor '{name}' offset {start} exceeds file boundary {mmap_len}"
            )))
        }
    }

    /// Return a human-readable inspection report
    pub fn inspect(&self) -> String {
        let mut out = String::new();
        out.push_str("=======================================================\n");
        out.push_str(&format!(" Tagisan GGUF Inspector: {}\n", self.path.display()));
        out.push_str("=======================================================\n");
        out.push_str(&format!("Format Version:      GGUF v{}\n", self.version));
        out.push_str(&format!("Total Tensors:       {}\n", self.tensor_count));
        out.push_str(&format!("Metadata KV Pairs:   {}\n", self.metadata_kv_count));
        out.push_str(&format!("Architecture:        {}\n", self.metadata.architecture));
        out.push_str(&format!("Context Length:      {}\n", self.metadata.context_length.unwrap_or(0)));
        out.push_str(&format!("Embedding Length:    {}\n", self.metadata.embedding_length.unwrap_or(0)));
        out.push_str(&format!("Block / Layer Count: {}\n", self.metadata.block_count.unwrap_or(0)));
        out.push_str(&format!("Attention Heads:     {}\n", self.metadata.head_count.unwrap_or(0)));
        out.push_str(&format!("KV Heads:            {}\n", self.metadata.head_count_kv.unwrap_or(0)));
        out.push_str(&format!("Vocab Size:          {}\n", self.metadata.vocab_size.unwrap_or(0)));
        out.push_str(&format!("Tensor Data Offset:  0x{:X} ({} bytes)\n", self.tensor_data_offset, self.tensor_data_offset));
        out.push_str("-------------------------------------------------------\n");
        out.push_str("Sample Tensors (First 15):\n");
        for (i, t) in self.tensors.iter().take(15).enumerate() {
            let dims_str = t
                .dims
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(" x ");
            out.push_str(&format!(
                "  [{:03}] {:<40} {:<8} [{}] ({:.2} MB)\n",
                i,
                t.name,
                t.type_name(),
                dims_str,
                (t.size_bytes as f64) / 1024.0 / 1024.0
            ));
        }
        if self.tensors.len() > 15 {
            out.push_str(&format!("  ... and {} more tensors\n", self.tensors.len() - 15));
        }
        out.push_str("=======================================================\n");
        out
    }
}

/// Discovered Ollama model details matching Ollama /api/tags JSON schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelDetails {
    pub parent_model: String,
    pub format: String,
    pub family: String,
    pub families: Vec<String>,
    pub parameter_size: String,
    pub quantization_level: String,
}

/// Discovered Ollama Model Summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelSummary {
    pub name: String,
    pub full_name: String,
    pub model: String,
    pub tag: String,
    pub size: u64,
    pub digest: String,
    pub modified_at: String,
    pub model_path: PathBuf,
    pub template_path: Option<PathBuf>,
    pub params_path: Option<PathBuf>,
    pub system_path: Option<PathBuf>,
    pub details: OllamaModelDetails,
}

/// Resolves Ollama models and blobs directly from the local filesystem
#[derive(Debug, Clone)]
pub struct OllamaBlobResolver {
    pub base_dir: PathBuf,
}

impl OllamaBlobResolver {
    /// Initialize with auto-detected or custom Ollama directory
    pub fn new(custom_path: Option<PathBuf>) -> Self {
        let base_dir = custom_path
            .or_else(|| std::env::var("OLLAMA_MODELS").ok().map(PathBuf::from))
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".ollama/models"))
            })
            .unwrap_or_else(|| PathBuf::from("/home/dyna/.ollama/models"));

        Self { base_dir }
    }

    /// List all installed Ollama models discovered in manifests
    pub fn list_installed_models(&self) -> Result<Vec<OllamaModelSummary>> {
        let manifests_dir = self.base_dir.join("manifests");
        let mut models = Vec::new();

        if !manifests_dir.exists() {
            return Ok(models);
        }

        let mut manifest_files = Vec::new();
        Self::collect_files_recursive(&manifests_dir, &mut manifest_files);

        for path in manifest_files {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(layers) = val.get("layers").and_then(|l| l.as_array()) {
                        // Extract relative path to manifests
                        let rel = path
                            .strip_prefix(&manifests_dir)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .to_string();

                        let parts: Vec<&str> = rel.split(std::path::MAIN_SEPARATOR).collect();
                        let (model_name, tag) = if parts.len() >= 2 {
                            let tag = parts[parts.len() - 1].to_string();
                            let model_name = parts[parts.len() - 2].to_string();
                            (model_name, tag)
                        } else {
                            ("unknown".to_string(), "latest".to_string())
                        };

                        let name = format!("{}:{}", model_name, tag);
                        let full_name = rel.clone();

                        let mut model_blob = None;
                        let mut model_size = 0u64;
                        let mut model_digest = String::new();
                        let mut template_path = None;
                        let mut params_path = None;
                        let mut system_path = None;

                        for layer in layers {
                            let media_type = layer.get("mediaType").and_then(|m| m.as_str()).unwrap_or("");
                            let digest = layer.get("digest").and_then(|d| d.as_str()).unwrap_or("");
                            let size = layer.get("size").and_then(|s| s.as_u64()).unwrap_or(0);

                            let blob_filename = digest.replace(':', "-");
                            let blob_path = self.base_dir.join("blobs").join(&blob_filename);

                            match media_type {
                                "application/vnd.ollama.image.model" => {
                                    model_blob = Some(blob_path);
                                    model_size = size;
                                    model_digest = digest.to_string();
                                }
                                "application/vnd.ollama.image.template" => {
                                    template_path = Some(blob_path);
                                }
                                "application/vnd.ollama.image.params" => {
                                    params_path = Some(blob_path);
                                }
                                "application/vnd.ollama.image.system" => {
                                    system_path = Some(blob_path);
                                }
                                _ => {}
                            }
                        }

                        if let Some(model_path) = model_blob {
                            // Extract family and parameter size from model name or GGUF
                            let (family, param_size, quant) = Self::infer_model_specs(&model_name, model_size);

                            let modified_at = std::fs::metadata(&path)
                                .and_then(|m| m.modified())
                                .map(|t| {
                                    let dt: chrono::DateTime<chrono::Utc> = t.into();
                                    dt.to_rfc3339()
                                })
                                .unwrap_or_else(|_| "2026-09-13T00:00:00Z".to_string());

                            models.push(OllamaModelSummary {
                                name,
                                full_name,
                                model: model_name,
                                tag,
                                size: model_size,
                                digest: model_digest,
                                modified_at,
                                model_path,
                                template_path,
                                params_path,
                                system_path,
                                details: OllamaModelDetails {
                                    parent_model: String::new(),
                                    format: "gguf".to_string(),
                                    family: family.clone(),
                                    families: vec![family],
                                    parameter_size: param_size,
                                    quantization_level: quant,
                                },
                            });
                        }
                    }
                }
            }
        }

        models.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(models)
    }

    /// Resolve a model query like "abliterated", "llama3.2-abliterate:3b-instruct", or partial name
    pub fn resolve(&self, query: &str) -> Result<OllamaModelSummary> {
        let models = self.list_installed_models()?;
        if models.is_empty() {
            return Err(TagisanError::Execution(format!(
                "No Ollama models found in '{}'",
                self.base_dir.display()
            )));
        }

        let q = query.trim().to_lowercase();

        // 1. Exact matches
        for m in &models {
            if m.name.to_lowercase() == q
                || m.full_name.to_lowercase() == q
                || m.model.to_lowercase() == q
                || format!("{}:{}", m.model.to_lowercase(), m.tag.to_lowercase()) == q
            {
                return Ok(m.clone());
            }
        }

        // 2. Substring matches
        for m in &models {
            if m.name.to_lowercase().contains(&q)
                || m.full_name.to_lowercase().contains(&q)
                || m.model.to_lowercase().contains(&q)
            {
                return Ok(m.clone());
            }
        }

        // 3. Stemmed / fuzzy match
        let stem = q
            .trim_end_matches('d')
            .trim_end_matches('s')
            .trim_end_matches("ed");
        for m in &models {
            let m_name = m.name.to_lowercase();
            let m_model = m.model.to_lowercase();
            if m_name.contains(stem) || m_model.contains(stem) || stem.contains(&m_model) {
                return Ok(m.clone());
            }
        }

        let available = models.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(", ");
        Err(TagisanError::Execution(format!(
            "Model '{query}' not found in Ollama store. Available models: [{available}]"
        )))
    }

    fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    Self::collect_files_recursive(&p, files);
                } else if p.is_file() {
                    files.push(p);
                }
            }
        }
    }

    fn infer_model_specs(model_name: &str, size_bytes: u64) -> (String, String, String) {
        let name_lower = model_name.to_lowercase();
        let family = if name_lower.contains("llama") {
            "llama".to_string()
        } else if name_lower.contains("qwen") {
            "qwen2".to_string()
        } else if name_lower.contains("mistral") {
            "mistral".to_string()
        } else if name_lower.contains("phi") {
            "phi3".to_string()
        } else if name_lower.contains("gemma") {
            "gemma".to_string()
        } else if name_lower.contains("deepseek") {
            "deepseek".to_string()
        } else {
            "llama".to_string()
        };

        let param_size = if name_lower.contains("3b") || name_lower.contains("3.2b") {
            "3.2B".to_string()
        } else if name_lower.contains("1b") {
            "1B".to_string()
        } else if name_lower.contains("7b") || name_lower.contains("8b") {
            "8B".to_string()
        } else if name_lower.contains("14b") {
            "14B".to_string()
        } else if name_lower.contains("70b") {
            "70B".to_string()
        } else {
            let gb = (size_bytes as f64) / (1024.0 * 1024.0 * 1024.0);
            if gb < 2.0 {
                "1.5B".to_string()
            } else if gb < 4.0 {
                "3.2B".to_string()
            } else if gb < 7.0 {
                "8B".to_string()
            } else {
                "14B".to_string()
            }
        };

        let quant = if name_lower.contains("q4") {
            "Q4_K_M".to_string()
        } else if name_lower.contains("q8") {
            "Q8_0".to_string()
        } else if name_lower.contains("fp16") || name_lower.contains("f16") {
            "F16".to_string()
        } else {
            "Q4_K_M".to_string()
        };

        (family, param_size, quant)
    }
}
