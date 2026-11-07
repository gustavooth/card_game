use serde::{Deserialize, Serialize};

/// Lado absoluto da arena. Azul começa embaixo (y alto), vermelho em cima.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Side {
    Blue = 0,
    Red = 1,
}

impl Side {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn enemy(self) -> Side {
        match self {
            Side::Blue => Side::Red,
            Side::Red => Side::Blue,
        }
    }

    pub fn from_index(value: u8) -> Side {
        if value == 0 { Side::Blue } else { Side::Red }
    }

    pub fn name(self) -> &'static str {
        match self {
            Side::Blue => "blue",
            Side::Red => "red",
        }
    }

    /// O lado azul ocupa a metade inferior da arena.
    pub fn is_bottom(self) -> bool {
        self == Side::Blue
    }
}
