use royale_shared::protocol::entity_index;
use royale_shared::{
    ANIM_ATTACK, ANIM_BURROW, ANIM_IDLE, ANIM_WALK, ENTITY_BUILDING, ENTITY_PROJECTILE,
    ENTITY_TOWER, ENTITY_TROOP, ENTITY_ZONE, PlayerSnapshot, Snapshot,
};

use crate::Match;
use crate::types::*;

impl Match {
    pub fn snapshot(&self) -> Snapshot {
        let mut entities = Vec::with_capacity(
            self.entities.len() + self.projectiles.len() + self.zones.len() + self.rollers.len(),
        );
        for entity in &self.entities {
            entities.push(self.entity_snapshot(entity));
        }
        for projectile in &self.projectiles {
            entities.push(vec![
                projectile.id as f32,
                ENTITY_PROJECTILE as f32,
                projectile.side as u8 as f32,
                projectile.visual_idx as f32,
                0.0,
                0.0,
                projectile.x,
                projectile.y,
                projectile.radius,
                0.0,
                0.0,
                0.0,
                if projectile.dir_x >= 0.0 { 1.0 } else { -1.0 },
                0.0,
                0.0,
                0.0,
            ]);
        }
        for zone in &self.zones {
            entities.push(vec![
                zone.id as f32,
                ENTITY_ZONE as f32,
                zone.side as u8 as f32,
                zone.spell_idx as f32,
                0.0,
                0.0,
                zone.x,
                zone.y,
                zone.radius,
                0.0,
                0.0,
                0.0,
                0.0,
                zone.remaining_ms as f32,
                0.0,
                0.0,
            ]);
        }
        for roller in &self.rollers {
            entities.push(vec![
                roller.id as f32,
                ENTITY_ZONE as f32,
                roller.side as u8 as f32,
                roller.spell_idx as f32,
                0.0,
                0.0,
                roller.x,
                roller.y,
                roller.width * 0.5,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
                0.0,
            ]);
        }

        let players = self
            .players
            .iter()
            .map(|player| {
                let hand = player
                    .hand
                    .iter()
                    .map(|index| {
                        let cost = self
                            .defs
                            .cards
                            .get(*index as usize)
                            .map(|card| card.elixir_cost as f32)
                            .unwrap_or(0.0);
                        [*index as f32, cost]
                    })
                    .collect::<Vec<_>>();
                let next_cost = self
                    .defs
                    .cards
                    .get(player.next as usize)
                    .map(|card| card.elixir_cost as f32)
                    .unwrap_or(0.0);
                PlayerSnapshot {
                    side: player.side as u8,
                    elixir: player.elixir,
                    crowns: player.crowns,
                    tower_damage: player.tower_damage,
                    hand,
                    next_card: [player.next as f32, next_cost],
                }
            })
            .collect::<Vec<_>>();

        let events = self
            .events
            .iter()
            .map(|event| {
                vec![
                    event.kind as f32,
                    event.x,
                    event.y,
                    event.side as f32,
                    event.index as f32,
                ]
            })
            .collect::<Vec<_>>();

        Snapshot {
            tag: "s",
            tick: self.tick,
            time_ms: self.elapsed_ms,
            phase: self.phase.code(),
            overtime: self.phase == Phase::Overtime,
            players,
            entities,
            events,
        }
    }

    fn entity_snapshot(&self, entity: &Entity) -> Vec<f32> {
        let (kind, type_index) = match entity.kind {
            EntityKind::Troop => (
                ENTITY_TROOP as u8,
                self.catalog.troop_index(&entity.slug).unwrap_or(0),
            ),
            EntityKind::Building => (
                ENTITY_BUILDING as u8,
                self.catalog.building_index(&entity.slug).unwrap_or(0),
            ),
            EntityKind::Tower => (
                ENTITY_TOWER as u8,
                self.catalog.tower_index(&entity.slug).unwrap_or(0),
            ),
        };
        let mut anim = entity.anim;
        if entity.burrowed {
            anim = ANIM_BURROW;
        } else if entity.windup > 0 {
            anim = ANIM_ATTACK;
        } else if entity.moving {
            anim = ANIM_WALK;
        } else if anim == ANIM_ATTACK {
            anim = ANIM_IDLE;
        }
        let life = if entity.is_tower {
            -1.0
        } else {
            entity.life_ms as f32
        };
        let mut tuple = Vec::with_capacity(entity_index::BASE_LEN + entity.effects.len() * 3);
        tuple.push(entity.id as f32);
        tuple.push(kind as f32);
        tuple.push(entity.side as u8 as f32);
        tuple.push(type_index as f32);
        tuple.push(entity.hp.max(0) as f32);
        tuple.push(entity.max_hp as f32);
        tuple.push(entity.x);
        tuple.push(entity.y);
        tuple.push(entity.radius);
        tuple.push(if entity.flying { 1.0 } else { 0.0 });
        tuple.push(entity.shield.max(0) as f32);
        tuple.push(anim as f32);
        tuple.push(entity.facing as f32);
        tuple.push(life);
        tuple.push(if entity.active { 1.0 } else { 0.0 });
        tuple.push(entity.effects.len() as f32);
        for effect in &entity.effects {
            let effect_index = self.catalog.effect_index(&effect.preset).unwrap_or(0);
            let magnitude = if effect.dps > 0.0 {
                effect.dps
            } else if effect.heal_per_sec > 0.0 {
                effect.heal_per_sec
            } else if effect.damage_mult > 1.0 {
                effect.damage_mult
            } else {
                1.0
            };
            tuple.push(effect_index as f32);
            tuple.push(effect.remaining_ms.max(0) as f32);
            tuple.push(magnitude);
        }
        tuple
    }
}
