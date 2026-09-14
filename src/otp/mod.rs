//! Native BEAM & OTP Engine for Tagisan.
//!
//! Provides production-grade Erlang External Term Format (ETF) serialization,
//! isolated actor processes with mailboxes and panic boundaries, true OTP
//! supervision trees with Let-It-Crash semantics, and Erlang Port protocols.

pub mod actor;
pub mod etf;
pub mod port;
pub mod supervisor;

pub use actor::{
    ActorError, ActorPid, ActorProcess, ActorRef, Envelope, GenServer, ProcessExit, ProcessRegistry,
};
pub use etf::{
    EtfDecoder, EtfEncoder, EtfError, Term, ATOM_EXT, ATOM_UTF8_EXT, BINARY_EXT, ETF_VERSION,
    FLOAT_EXT, INTEGER_EXT, LARGE_BIG_EXT, LARGE_TUPLE_EXT, LIST_EXT, MAP_EXT, NEW_FLOAT_EXT,
    NEW_PID_EXT, NIL_EXT, PID_EXT, SMALL_ATOM_EXT, SMALL_ATOM_UTF8_EXT, SMALL_BIG_EXT,
    SMALL_INTEGER_EXT, SMALL_TUPLE_EXT, STRING_EXT,
};
pub use port::{run_port_loop, PacketFramer, PortDispatcher};
pub use supervisor::{
    ChildFactory, ChildInfo, ChildSpec, RestartStrategy, RestartType, Supervisor, SupervisorError,
    SupervisorHandle, SupervisorSpec,
};
