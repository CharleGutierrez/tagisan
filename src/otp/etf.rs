//! Erlang External Term Format (ETF, tag 131) Binary Encoder and Decoder.
//!
//! Compliant with Erlang/OTP BEAM External Term Format specification:
//! <https://www.erlang.org/doc/apps/erts/erl_ext_dist.html>

use std::fmt;
use thiserror::Error;

/// ETF Magic Version Header (131 / 0x83)
pub const ETF_VERSION: u8 = 131;

// ETF Term Tags
pub const NEW_FLOAT_EXT: u8 = 70;
pub const NEW_PID_EXT: u8 = 88;
pub const SMALL_INTEGER_EXT: u8 = 97;
pub const INTEGER_EXT: u8 = 98;
pub const FLOAT_EXT: u8 = 99;
pub const ATOM_EXT: u8 = 100;
pub const PID_EXT: u8 = 103;
pub const SMALL_TUPLE_EXT: u8 = 104;
pub const LARGE_TUPLE_EXT: u8 = 105;
pub const NIL_EXT: u8 = 106;
pub const STRING_EXT: u8 = 107;
pub const LIST_EXT: u8 = 108;
pub const BINARY_EXT: u8 = 109;
pub const SMALL_BIG_EXT: u8 = 110;
pub const LARGE_BIG_EXT: u8 = 111;
pub const SMALL_ATOM_EXT: u8 = 115;
pub const MAP_EXT: u8 = 116;
pub const ATOM_UTF8_EXT: u8 = 118;
pub const SMALL_ATOM_UTF8_EXT: u8 = 119;

/// Errors that can occur during ETF serialization or deserialization
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EtfError {
    #[error("Invalid ETF magic version byte: expected {expected}, got {got}")]
    InvalidVersion { expected: u8, got: u8 },
    #[error("Unexpected end of buffer at offset {offset}, needed {needed} bytes")]
    UnexpectedEof { offset: usize, needed: usize },
    #[error("Unknown or unsupported ETF tag: {0}")]
    UnknownTag(u8),
    #[error("Invalid UTF-8 string in term: {0}")]
    InvalidUtf8(String),
    #[error("Integer overflow or unsupported big integer size")]
    IntegerOverflow,
    #[error("Malformed float string: {0}")]
    MalformedFloat(String),
    #[error("Malformed ETF structure: {0}")]
    Malformed(String),
}

/// Representation of an Erlang/Elixir BEAM Term
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    /// Erlang Atom (e.g. :ok, :error, :tagisan)
    Atom(String),
    /// Signed integer (covers SmallInt, Int, and 64-bit BigInt)
    Integer(i64),
    /// 64-bit IEEE-754 float
    Float(f64),
    /// Raw binary data (e.g. <<"hello">>)
    Binary(Vec<u8>),
    /// String (list of bytes 0..255)
    String(String),
    /// Erlang List of terms
    List(Vec<Term>),
    /// Erlang Tuple of terms (e.g. {:ok, result})
    Tuple(Vec<Term>),
    /// Erlang / Elixir Map of Key-Value pairs (%{key => value})
    Map(Vec<(Term, Term)>),
    /// Empty list []
    Nil,
    /// BEAM Process Identifier (node, id, serial, creation)
    Pid {
        node: String,
        id: u32,
        serial: u32,
        creation: u32,
    },
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Atom(s) => write!(f, ":{}", s),
            Term::Integer(i) => write!(f, "{}", i),
            Term::Float(v) => write!(f, "{}", v),
            Term::Binary(b) => {
                if let Ok(s) = std::str::from_utf8(b) {
                    write!(f, "<<{:?}>>", s)
                } else {
                    write!(f, "<<{:?}>>", b)
                }
            }
            Term::String(s) => write!(f, "\"{}\"", s),
            Term::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Term::Tuple(elements) => {
                write!(f, "{{")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "}}")
            }
            Term::Map(pairs) => {
                write!(f, "%{{")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} => {}", k, v)?;
                }
                write!(f, "}}")
            }
            Term::Nil => write!(f, "[]"),
            Term::Pid { node, id, serial, creation: _ } => {
                write!(f, "<{}.{}.{}>", node, id, serial)
            }
        }
    }
}

impl Term {
    // -------------------------------------------------------------------------
    // Constructors
    // -------------------------------------------------------------------------

    pub fn atom(s: impl Into<String>) -> Self {
        Term::Atom(s.into())
    }

    pub fn int(i: impl Into<i64>) -> Self {
        Term::Integer(i.into())
    }

    pub fn float(f: f64) -> Self {
        Term::Float(f)
    }

    pub fn binary(b: impl Into<Vec<u8>>) -> Self {
        Term::Binary(b.into())
    }

    pub fn string(s: impl Into<String>) -> Self {
        Term::String(s.into())
    }

    pub fn list(items: Vec<Term>) -> Self {
        if items.is_empty() {
            Term::Nil
        } else {
            Term::List(items)
        }
    }

    pub fn tuple(elements: Vec<Term>) -> Self {
        Term::Tuple(elements)
    }

    pub fn map(pairs: Vec<(Term, Term)>) -> Self {
        Term::Map(pairs)
    }

    pub fn nil() -> Self {
        Term::Nil
    }

    pub fn boolean(b: bool) -> Self {
        Term::Atom(if b { "true".to_string() } else { "false".to_string() })
    }

    pub fn ok() -> Self {
        Term::Atom("ok".to_string())
    }

    pub fn ok_val(val: Term) -> Self {
        Term::Tuple(vec![Term::Atom("ok".to_string()), val])
    }

    pub fn error(reason: Term) -> Self {
        Term::Tuple(vec![Term::Atom("error".to_string()), reason])
    }

    // -------------------------------------------------------------------------
    // Inspectors / Accessors
    // -------------------------------------------------------------------------

    pub fn as_atom(&self) -> Option<&str> {
        match self {
            Term::Atom(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Term::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Term::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            Term::Binary(b) => Some(b.as_slice()),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Term::String(s) => Some(s.as_str()),
            Term::Binary(b) => std::str::from_utf8(b).ok(),
            Term::Atom(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_tuple(&self) -> Option<&[Term]> {
        match self {
            Term::Tuple(elems) => Some(elems.as_slice()),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Term]> {
        match self {
            Term::List(items) => Some(items.as_slice()),
            Term::Nil => Some(&[]),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self.as_atom() {
            Some("true") => Some(true),
            Some("false") => Some(false),
            _ => None,
        }
    }

    pub fn is_nil(&self) -> bool {
        matches!(self, Term::Nil)
    }

    pub fn get_map_value(&self, key: &Term) -> Option<&Term> {
        match self {
            Term::Map(pairs) => pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    // -------------------------------------------------------------------------
    // Encoding
    // -------------------------------------------------------------------------

    /// Encode this term to an Erlang External Term Format byte buffer (prefixed with tag 131)
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(64);
        buf.push(ETF_VERSION);
        self.encode_inner(&mut buf);
        buf
    }

    fn encode_inner(&self, buf: &mut Vec<u8>) {
        match self {
            Term::Atom(s) => {
                let bytes = s.as_bytes();
                if bytes.len() <= 255 {
                    buf.push(SMALL_ATOM_UTF8_EXT);
                    buf.push(bytes.len() as u8);
                    buf.extend_from_slice(bytes);
                } else {
                    buf.push(ATOM_UTF8_EXT);
                    buf.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
                    buf.extend_from_slice(bytes);
                }
            }
            Term::Integer(val) => {
                let v = *val;
                if (0..=255).contains(&v) {
                    buf.push(SMALL_INTEGER_EXT);
                    buf.push(v as u8);
                } else if (i32::MIN as i64..=i32::MAX as i64).contains(&v) {
                    buf.push(INTEGER_EXT);
                    buf.extend_from_slice(&(v as i32).to_be_bytes());
                } else {
                    // Small Big Integer (Tag 110)
                    let sign: u8 = if v < 0 { 1 } else { 0 };
                    let abs_val = v.unsigned_abs();
                    let mut digits = Vec::new();
                    let mut temp = abs_val;
                    while temp > 0 {
                        digits.push((temp & 0xFF) as u8);
                        temp >>= 8;
                    }
                    if digits.is_empty() {
                        digits.push(0);
                    }
                    buf.push(SMALL_BIG_EXT);
                    buf.push(digits.len() as u8);
                    buf.push(sign);
                    buf.extend_from_slice(&digits);
                }
            }
            Term::Float(f) => {
                buf.push(NEW_FLOAT_EXT);
                buf.extend_from_slice(&f.to_be_bytes());
            }
            Term::Binary(bytes) => {
                buf.push(BINARY_EXT);
                buf.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
                buf.extend_from_slice(bytes);
            }
            Term::String(s) => {
                let bytes = s.as_bytes();
                if bytes.len() <= u16::MAX as usize {
                    buf.push(STRING_EXT);
                    buf.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
                    buf.extend_from_slice(bytes);
                } else {
                    buf.push(BINARY_EXT);
                    buf.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
                    buf.extend_from_slice(bytes);
                }
            }
            Term::Nil => {
                buf.push(NIL_EXT);
            }
            Term::List(items) => {
                if items.is_empty() {
                    buf.push(NIL_EXT);
                } else {
                    buf.push(LIST_EXT);
                    buf.extend_from_slice(&(items.len() as u32).to_be_bytes());
                    for item in items {
                        item.encode_inner(buf);
                    }
                    buf.push(NIL_EXT); // Tail of proper list
                }
            }
            Term::Tuple(elements) => {
                if elements.len() <= 255 {
                    buf.push(SMALL_TUPLE_EXT);
                    buf.push(elements.len() as u8);
                } else {
                    buf.push(LARGE_TUPLE_EXT);
                    buf.extend_from_slice(&(elements.len() as u32).to_be_bytes());
                }
                for elem in elements {
                    elem.encode_inner(buf);
                }
            }
            Term::Map(pairs) => {
                buf.push(MAP_EXT);
                buf.extend_from_slice(&(pairs.len() as u32).to_be_bytes());
                for (k, v) in pairs {
                    k.encode_inner(buf);
                    v.encode_inner(buf);
                }
            }
            Term::Pid { node, id, serial, creation } => {
                buf.push(NEW_PID_EXT);
                let node_bytes = node.as_bytes();
                buf.push(SMALL_ATOM_UTF8_EXT);
                buf.push(node_bytes.len() as u8);
                buf.extend_from_slice(node_bytes);
                buf.extend_from_slice(&id.to_be_bytes());
                buf.extend_from_slice(&serial.to_be_bytes());
                buf.extend_from_slice(&creation.to_be_bytes());
            }
        }
    }

    // -------------------------------------------------------------------------
    // Decoding
    // -------------------------------------------------------------------------

    /// Decode a complete ETF buffer (must begin with version tag 131)
    pub fn decode(bytes: &[u8]) -> Result<Term, EtfError> {
        let (term, _consumed) = Self::decode_full(bytes)?;
        Ok(term)
    }

    /// Decode an ETF term and return the decoded Term and total bytes consumed
    pub fn decode_full(bytes: &[u8]) -> Result<(Term, usize), EtfError> {
        if bytes.is_empty() {
            return Err(EtfError::UnexpectedEof { offset: 0, needed: 1 });
        }
        if bytes[0] != ETF_VERSION {
            return Err(EtfError::InvalidVersion {
                expected: ETF_VERSION,
                got: bytes[0],
            });
        }
        let (term, consumed) = Self::decode_inner(&bytes[1..], 1)?;
        Ok((term, consumed + 1))
    }

    fn decode_inner(bytes: &[u8], offset: usize) -> Result<(Term, usize), EtfError> {
        if bytes.is_empty() {
            return Err(EtfError::UnexpectedEof { offset, needed: 1 });
        }

        let tag = bytes[0];
        let rest = &bytes[1..];
        let cur_offset = offset + 1;

        match tag {
            SMALL_INTEGER_EXT => {
                if rest.is_empty() {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 1 });
                }
                let val = rest[0] as i64;
                Ok((Term::Integer(val), 2))
            }
            INTEGER_EXT => {
                if rest.len() < 4 {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 4 });
                }
                let val = i32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]) as i64;
                Ok((Term::Integer(val), 5))
            }
            SMALL_BIG_EXT => {
                if rest.len() < 2 {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 2 });
                }
                let n = rest[0] as usize;
                let sign = rest[1];
                if rest.len() < 2 + n {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset + 2, needed: n });
                }
                let digits = &rest[2..2 + n];
                if n > 8 {
                    return Err(EtfError::IntegerOverflow);
                }
                let mut val: u64 = 0;
                for (i, &d) in digits.iter().enumerate() {
                    val |= (d as u64) << (i * 8);
                }
                let signed_val = if sign == 1 {
                    -(val as i64)
                } else {
                    val as i64
                };
                Ok((Term::Integer(signed_val), 1 + 2 + n))
            }
            LARGE_BIG_EXT => {
                if rest.len() < 5 {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 5 });
                }
                let n = u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
                let sign = rest[4];
                if rest.len() < 5 + n {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset + 5, needed: n });
                }
                let digits = &rest[5..5 + n];
                if n > 8 {
                    return Err(EtfError::IntegerOverflow);
                }
                let mut val: u64 = 0;
                for (i, &d) in digits.iter().enumerate() {
                    val |= (d as u64) << (i * 8);
                }
                let signed_val = if sign == 1 {
                    -(val as i64)
                } else {
                    val as i64
                };
                Ok((Term::Integer(signed_val), 1 + 5 + n))
            }
            NEW_FLOAT_EXT => {
                if rest.len() < 8 {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 8 });
                }
                let val = f64::from_be_bytes([
                    rest[0], rest[1], rest[2], rest[3], rest[4], rest[5], rest[6], rest[7],
                ]);
                Ok((Term::Float(val), 9))
            }
            FLOAT_EXT => {
                if rest.len() < 31 {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 31 });
                }
                let s = std::str::from_utf8(&rest[..31])
                    .map_err(|e| EtfError::InvalidUtf8(e.to_string()))?;
                let clean = s.trim_end_matches('\0').trim();
                let f = clean.parse::<f64>()
                    .map_err(|e| EtfError::MalformedFloat(e.to_string()))?;
                Ok((Term::Float(f), 32))
            }
            SMALL_ATOM_UTF8_EXT | SMALL_ATOM_EXT => {
                if rest.is_empty() {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 1 });
                }
                let len = rest[0] as usize;
                if rest.len() < 1 + len {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset + 1, needed: len });
                }
                let s = std::str::from_utf8(&rest[1..1 + len])
                    .map_err(|e| EtfError::InvalidUtf8(e.to_string()))?;
                Ok((Term::Atom(s.to_string()), 1 + 1 + len))
            }
            ATOM_UTF8_EXT | ATOM_EXT => {
                if rest.len() < 2 {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 2 });
                }
                let len = u16::from_be_bytes([rest[0], rest[1]]) as usize;
                if rest.len() < 2 + len {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset + 2, needed: len });
                }
                let s = std::str::from_utf8(&rest[2..2 + len])
                    .map_err(|e| EtfError::InvalidUtf8(e.to_string()))?;
                Ok((Term::Atom(s.to_string()), 1 + 2 + len))
            }
            BINARY_EXT => {
                if rest.len() < 4 {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 4 });
                }
                let len = u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
                if rest.len() < 4 + len {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset + 4, needed: len });
                }
                let bytes = rest[4..4 + len].to_vec();
                Ok((Term::Binary(bytes), 1 + 4 + len))
            }
            STRING_EXT => {
                if rest.len() < 2 {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 2 });
                }
                let len = u16::from_be_bytes([rest[0], rest[1]]) as usize;
                if rest.len() < 2 + len {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset + 2, needed: len });
                }
                let s = String::from_utf8_lossy(&rest[2..2 + len]).into_owned();
                Ok((Term::String(s), 1 + 2 + len))
            }
            NIL_EXT => {
                Ok((Term::Nil, 1))
            }
            LIST_EXT => {
                if rest.len() < 4 {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 4 });
                }
                let len = u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
                let mut consumed = 5; // tag (1) + length (4)
                let mut items = Vec::with_capacity(len);

                for _ in 0..len {
                    let (item, c) = Self::decode_inner(&bytes[consumed..], offset + consumed)?;
                    items.push(item);
                    consumed += c;
                }

                // Decode tail
                if bytes.len() <= consumed {
                    return Err(EtfError::UnexpectedEof { offset: offset + consumed, needed: 1 });
                }
                let (tail, c) = Self::decode_inner(&bytes[consumed..], offset + consumed)?;
                consumed += c;

                if !tail.is_nil() {
                    // Improper list: append tail as last item
                    items.push(tail);
                }

                Ok((Term::List(items), consumed))
            }
            SMALL_TUPLE_EXT => {
                if rest.is_empty() {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 1 });
                }
                let arity = rest[0] as usize;
                let mut consumed = 2; // tag (1) + arity (1)
                let mut elements = Vec::with_capacity(arity);

                for _ in 0..arity {
                    let (elem, c) = Self::decode_inner(&bytes[consumed..], offset + consumed)?;
                    elements.push(elem);
                    consumed += c;
                }

                Ok((Term::Tuple(elements), consumed))
            }
            LARGE_TUPLE_EXT => {
                if rest.len() < 4 {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 4 });
                }
                let arity = u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
                let mut consumed = 5; // tag (1) + arity (4)
                let mut elements = Vec::with_capacity(arity);

                for _ in 0..arity {
                    let (elem, c) = Self::decode_inner(&bytes[consumed..], offset + consumed)?;
                    elements.push(elem);
                    consumed += c;
                }

                Ok((Term::Tuple(elements), consumed))
            }
            MAP_EXT => {
                if rest.len() < 4 {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset, needed: 4 });
                }
                let arity = u32::from_be_bytes([rest[0], rest[1], rest[2], rest[3]]) as usize;
                let mut consumed = 5; // tag (1) + arity (4)
                let mut pairs = Vec::with_capacity(arity);

                for _ in 0..arity {
                    let (k, c1) = Self::decode_inner(&bytes[consumed..], offset + consumed)?;
                    consumed += c1;
                    let (v, c2) = Self::decode_inner(&bytes[consumed..], offset + consumed)?;
                    consumed += c2;
                    pairs.push((k, v));
                }

                Ok((Term::Map(pairs), consumed))
            }
            PID_EXT => {
                let (node_term, c1) = Self::decode_inner(rest, cur_offset)?;
                let node = node_term.as_atom().unwrap_or("nonode@nohost").to_string();
                let sub = &rest[c1..];
                if sub.len() < 9 {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset + c1, needed: 9 });
                }
                let id = u32::from_be_bytes([sub[0], sub[1], sub[2], sub[3]]);
                let serial = u32::from_be_bytes([sub[4], sub[5], sub[6], sub[7]]);
                let creation = sub[8] as u32;
                Ok((Term::Pid { node, id, serial, creation }, 1 + c1 + 9))
            }
            NEW_PID_EXT => {
                let (node_term, c1) = Self::decode_inner(rest, cur_offset)?;
                let node = node_term.as_atom().unwrap_or("nonode@nohost").to_string();
                let sub = &rest[c1..];
                if sub.len() < 12 {
                    return Err(EtfError::UnexpectedEof { offset: cur_offset + c1, needed: 12 });
                }
                let id = u32::from_be_bytes([sub[0], sub[1], sub[2], sub[3]]);
                let serial = u32::from_be_bytes([sub[4], sub[5], sub[6], sub[7]]);
                let creation = u32::from_be_bytes([sub[8], sub[9], sub[10], sub[11]]);
                Ok((Term::Pid { node, id, serial, creation }, 1 + c1 + 12))
            }
            _ => Err(EtfError::UnknownTag(tag)),
        }
    }
}

/// Helper struct for streaming ETF encoding
pub struct EtfEncoder;

impl EtfEncoder {
    pub fn encode(term: &Term) -> Vec<u8> {
        term.encode()
    }
}

/// Helper struct for streaming ETF decoding
pub struct EtfDecoder;

impl EtfDecoder {
    pub fn decode(bytes: &[u8]) -> Result<Term, EtfError> {
        Term::decode(bytes)
    }

    pub fn decode_full(bytes: &[u8]) -> Result<(Term, usize), EtfError> {
        Term::decode_full(bytes)
    }
}
