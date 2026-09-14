//! Erlang Port & Port Driver Protocol.
//!
//! Provides a 4-byte network-endian length-prefixed packet protocol compatible with:
//! `Port.open({:spawn_executable, "tgs"}, [:binary, {:packet, 4}])` in Elixir/Erlang.

use std::io;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::otp::actor::ProcessRegistry;
use crate::otp::etf::{EtfDecoder, EtfEncoder, Term};

/// Wire-format packet framer for 4-byte network byte order (big-endian) packets
pub struct PacketFramer;

impl PacketFramer {
    /// Prepend a 4-byte big-endian length prefix to payload
    pub fn encode_packet(payload: &[u8]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + payload.len());
        let len = payload.len() as u32;
        buf.extend_from_slice(&len.to_be_bytes());
        buf.extend_from_slice(payload);
        buf
    }

    /// Extract a complete packet from an accumulating buffer if available
    pub fn decode_packet(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
        if buffer.len() < 4 {
            return None;
        }
        let len = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]) as usize;
        if buffer.len() < 4 + len {
            return None;
        }
        buffer.drain(0..4);
        let packet = buffer.drain(0..len).collect();
        Some(packet)
    }

    /// Asynchronously read a 4-byte length prefixed packet from a stream
    pub async fn read_packet<R: AsyncRead + Unpin>(reader: &mut R) -> io::Result<Vec<u8>> {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;
        let mut payload = vec![0u8; len];
        reader.read_exact(&mut payload).await?;
        Ok(payload)
    }

    /// Asynchronously write a 4-byte length prefixed packet to a stream
    pub async fn write_packet<W: AsyncWrite + Unpin>(
        writer: &mut W,
        payload: &[u8],
    ) -> io::Result<()> {
        let len = payload.len() as u32;
        writer.write_all(&len.to_be_bytes()).await?;
        writer.write_all(payload).await?;
        writer.flush().await?;
        Ok(())
    }
}

/// Dispatches Erlang/Elixir commands received over the Port protocol
pub struct PortDispatcher;

impl PortDispatcher {
    /// Dispatch an incoming packet and return the response packet
    pub async fn dispatch(packet: &[u8]) -> Vec<u8> {
        let response_term = match EtfDecoder::decode(packet) {
            Ok(term) => Self::handle_term(term).await,
            Err(e) => Term::error(Term::string(format!("invalid_etf: {}", e))),
        };

        EtfEncoder::encode(&response_term)
    }

    async fn handle_term(term: Term) -> Term {
        match term {
            // :ping or {:ping}
            Term::Atom(ref s) if s == "ping" => Term::Atom("pong".to_string()),
            Term::Tuple(ref elems) if elems.len() == 1 && elems[0].as_atom() == Some("ping") => {
                Term::Atom("pong".to_string())
            }

            // :status or {:status}
            Term::Atom(ref s) if s == "status" || s == "info" => Self::status_term(),
            Term::Tuple(ref elems) if elems.len() == 1 && elems[0].as_atom() == Some("status") => {
                Self::status_term()
            }

            // {:echo, term}
            Term::Tuple(ref elems) if elems.len() == 2 && elems[0].as_atom() == Some("echo") => {
                Term::ok_val(elems[1].clone())
            }

            // {:whereis, actor_name}
            Term::Tuple(ref elems)
                if elems.len() == 2 && elems[0].as_atom() == Some("whereis") =>
            {
                if let Some(name) = elems[1].as_str() {
                    if let Some(actor) = ProcessRegistry::whereis(name) {
                        let pid = actor.pid();
                        let pid_term = Term::Pid {
                            node: pid.node.clone(),
                            id: pid.id as u32,
                            serial: pid.serial,
                            creation: 0,
                        };
                        Term::ok_val(pid_term)
                    } else {
                        Term::Atom("undefined".to_string())
                    }
                } else {
                    Term::error(Term::atom("invalid_name"))
                }
            }

            // {:call, actor_name, request}
            Term::Tuple(ref elems) if elems.len() == 3 && elems[0].as_atom() == Some("call") => {
                if let Some(name) = elems[1].as_str() {
                    if let Some(actor) = ProcessRegistry::whereis(name) {
                        let timeout = Duration::from_secs(5);
                        match actor.call(elems[2].clone(), timeout).await {
                            Ok(reply) => Term::ok_val(reply),
                            Err(e) => Term::error(Term::string(e.to_string())),
                        }
                    } else {
                        Term::error(Term::atom("actor_not_found"))
                    }
                } else {
                    Term::error(Term::atom("invalid_actor_name"))
                }
            }

            // {:cast, actor_name, message}
            Term::Tuple(ref elems) if elems.len() == 3 && elems[0].as_atom() == Some("cast") => {
                if let Some(name) = elems[1].as_str() {
                    if let Some(actor) = ProcessRegistry::whereis(name) {
                        match actor.cast(elems[2].clone()).await {
                            Ok(()) => Term::ok(),
                            Err(e) => Term::error(Term::string(e.to_string())),
                        }
                    } else {
                        Term::error(Term::atom("actor_not_found"))
                    }
                } else {
                    Term::error(Term::atom("invalid_actor_name"))
                }
            }

            // {:ask, prompt}
            Term::Tuple(ref elems) if elems.len() == 2 && elems[0].as_atom() == Some("ask") => {
                if let Some(prompt) = elems[1].as_str() {
                    let response = format!("Tagisan Sovereignty acknowledged prompt: {}", prompt);
                    Term::ok_val(Term::string(response))
                } else {
                    Term::error(Term::atom("invalid_prompt"))
                }
            }

            // Unknown command
            other => Term::error(Term::tuple(vec![
                Term::atom("unknown_command"),
                other,
            ])),
        }
    }

    fn status_term() -> Term {
        let pairs = vec![
            (Term::atom("engine"), Term::atom("tagisan")),
            (Term::atom("version"), Term::string("0.2.0")),
            (Term::atom("status"), Term::atom("ready")),
            (
                Term::atom("registered_actors"),
                Term::int(ProcessRegistry::all_registered().len() as i64),
            ),
        ];
        Term::ok_val(Term::map(pairs))
    }
}

/// Run the stdio Erlang Port 4-byte packet protocol event loop
pub async fn run_port_loop() -> io::Result<()> {
    let mut stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();

    loop {
        match PacketFramer::read_packet(&mut stdin).await {
            Ok(packet) => {
                let response = PortDispatcher::dispatch(&packet).await;
                PacketFramer::write_packet(&mut stdout, &response).await?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                // Port closed by parent BEAM / Elixir process
                break;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(())
}
