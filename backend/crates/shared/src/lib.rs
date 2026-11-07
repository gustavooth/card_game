//! Tipos compartilhados entre servidor e cliente (protocolo e constantes).

pub mod constants;
pub mod protocol;
pub mod side;

pub use constants::*;
pub use protocol::*;
pub use side::Side;
