use serde::Serialize;

use crate::types::Definitions;

/// Listas ordenadas usadas pelos índices numéricos dos snapshots.
#[derive(Debug, Clone, Serialize)]
pub struct Catalog {
    pub cards: Vec<String>,
    pub troops: Vec<String>,
    pub buildings: Vec<String>,
    pub towers: Vec<String>,
    pub spells: Vec<String>,
    pub projectiles: Vec<String>,
    pub effects: Vec<String>,
    pub skills: Vec<String>,
    pub arenas: Vec<String>,
}

impl Catalog {
    pub fn build(defs: &Definitions) -> Self {
        Self {
            cards: defs.cards.iter().map(|c| c.slug.clone()).collect(),
            troops: defs.troops.keys().cloned().collect(),
            buildings: defs.buildings.keys().cloned().collect(),
            towers: defs.towers.keys().cloned().collect(),
            spells: defs.spells.keys().cloned().collect(),
            projectiles: defs.projectiles.keys().cloned().collect(),
            effects: defs.effects.keys().cloned().collect(),
            skills: defs.skills.keys().cloned().collect(),
            arenas: defs.arenas.keys().cloned().collect(),
        }
    }

    pub fn card_index(&self, slug: &str) -> Option<u16> {
        index_of(&self.cards, slug)
    }

    pub fn troop_index(&self, slug: &str) -> Option<u16> {
        index_of(&self.troops, slug)
    }

    pub fn building_index(&self, slug: &str) -> Option<u16> {
        index_of(&self.buildings, slug)
    }

    pub fn tower_index(&self, slug: &str) -> Option<u16> {
        index_of(&self.towers, slug)
    }

    pub fn spell_index(&self, slug: &str) -> Option<u16> {
        index_of(&self.spells, slug)
    }

    pub fn projectile_index(&self, slug: &str) -> Option<u16> {
        index_of(&self.projectiles, slug)
    }

    pub fn effect_index(&self, slug: &str) -> Option<u16> {
        index_of(&self.effects, slug)
    }
}

fn index_of(list: &[String], slug: &str) -> Option<u16> {
    list.iter().position(|item| item == slug).map(|i| i as u16)
}
