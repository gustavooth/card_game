//! Simulação autoritativa do jogo, independente de rede e banco.

mod combat;
mod effects;
mod match_state;
mod movement;
mod rng;
mod skills;
mod snapshot;
mod spawn;
mod spells;
mod types;
mod util;

pub use match_state::{DeployError, Match};
pub use rng::Rng;
pub use spawn::{REQUIRED_DECK, deck_indices, deck_indices as validate_deck, default_deck};
pub use types::*;

pub use royale_defs;
pub use royale_shared;
