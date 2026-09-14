//// Strongly Typed Erlang External Term Format (ETF) Wire Protocol Representation.

pub type EtfPacket {
  AtomPacket(name: String)
  IntPacket(value: Int)
  FloatPacket(value: Float)
  StringPacket(value: String)
  TuplePacket(items: List(EtfPacket))
  ListPacket(items: List(EtfPacket))
  NilPacket
}

pub type PacketHeader {
  PacketHeader(magic_version: Int, payload_length: Int)
}

pub const etf_version = 131

/// Validate standard 131 magic version header of incoming ETF term buffer
pub fn validate_version(header_byte: Int) -> Result(Int, String) {
  case header_byte {
    131 -> Ok(131)
    other -> Error("Invalid ETF version header: expected 131, got " <> "unknown")
  }
}

/// Create an OTP-compliant Call request envelope
pub fn call_envelope(id: Int, payload: EtfPacket) -> EtfPacket {
  TuplePacket([
    AtomPacket("$gen_call"),
    TuplePacket([AtomPacket("client_ref"), IntPacket(id)]),
    payload,
  ])
}

/// Create an OTP-compliant Cast message envelope
pub fn cast_envelope(payload: EtfPacket) -> EtfPacket {
  TuplePacket([
    AtomPacket("$gen_cast"),
    payload,
  ])
}
