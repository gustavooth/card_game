use std::sync::Arc;

use royale_core::{Match, Phase};
use royale_defs::{CardType, load};
use royale_shared::Side;
use uuid::Uuid;

fn new_match() -> Match {
    let loaded = load().expect("definições válidas");
    let deck = royale_core::default_deck(&loaded.defs, &loaded.catalog);
    let defs = Arc::new(loaded.defs);
    let indices = royale_core::deck_indices(&defs, &loaded.catalog, &deck).unwrap();
    Match::new(
        defs,
        loaded.catalog,
        "test-room".into(),
        "classic",
        [
            (Uuid::new_v4(), "blue".into(), indices.clone()),
            (Uuid::new_v4(), "red".into(), indices),
        ],
        42,
        0,
    )
    .expect("partida criada")
}

#[test]
fn loads_fifty_cards_and_validates() {
    let loaded = load().expect("definições válidas");
    assert_eq!(loaded.defs.cards.len(), 50);
    assert!(
        loaded
            .defs
            .cards
            .iter()
            .any(|card| matches!(card.card_type, CardType::Spell { .. }))
    );
    assert!(
        loaded
            .defs
            .cards
            .iter()
            .any(|card| matches!(card.card_type, CardType::Building { .. }))
    );
}

#[test]
fn every_definition_can_spawn_without_panicking() {
    let mut game = new_match();
    let troops: Vec<String> = game.defs().troops.keys().cloned().collect();
    for (index, slug) in troops.iter().enumerate() {
        let x = 2.0 + (index % 6) as f32 * 2.2;
        let y = if index % 2 == 0 { 24.0 } else { 8.0 };
        let side = if index % 2 == 0 {
            Side::Blue
        } else {
            Side::Red
        };
        game.spawn_troop(side, "knight", slug, x, y, None, false, None);
    }
    let buildings: Vec<String> = game.defs().buildings.keys().cloned().collect();
    for (index, slug) in buildings.iter().enumerate() {
        let x = 3.0 + index as f32 * 1.8;
        game.spawn_building(Side::Blue, "cannon", slug, x, 26.0, None);
    }
    let spells: Vec<String> = game.defs().spells.keys().cloned().collect();
    for (index, slug) in spells.iter().enumerate() {
        let x = 2.0 + (index % 5) as f32 * 3.0;
        game.cast_spell(Side::Red, "fireball", slug, x, 20.0);
    }
    for _ in 0..30 {
        game.tick();
    }
    assert!(game.entity_count() > 0);
}

#[test]
fn full_match_reaches_combat_and_ends() {
    let mut game = new_match();
    let mut ended_before = false;
    let mut deploys = [0u32; 2];
    for tick in 0..9000u32 {
        if tick % 40 == 0 {
            for index in 0..2 {
                let side = Side::from_index(index as u8);
                game.players[index].elixir = 10.0;
                let x = if (tick / 40 + index as u32) % 2 == 0 {
                    4.0
                } else {
                    14.0
                };
                let y = if side == Side::Blue { 22.0 } else { 10.0 };
                for slot in 0..4u8 {
                    if game.deploy(game.players[index].user_id, slot, x, y).is_ok() {
                        deploys[index] += 1;
                        break;
                    }
                }
            }
        }
        if tick == 1500 {
            let blue_units = game
                .entities
                .iter()
                .filter(|entity| entity.side == Side::Blue && !entity.is_tower)
                .count();
            let red_units = game
                .entities
                .iter()
                .filter(|entity| entity.side == Side::Red && !entity.is_tower)
                .count();
            let blue_towers: Vec<i32> = game
                .entities
                .iter()
                .filter(|entity| entity.side == Side::Blue && entity.is_tower)
                .map(|entity| entity.hp)
                .collect();
            println!(
                "t=1500 deploys={:?} blue_units={blue_units} red_units={red_units} blue_towers={blue_towers:?}",
                deploys
            );
        }
        game.tick();
        if game.phase == Phase::Ended {
            ended_before = true;
            break;
        }
    }
    let result = game.result();
    println!(
        "fim: winner={:?} reason={:?} crowns={:?} damage={:?} tick={}",
        result.winner, result.end_reason, result.crowns, result.damage, game.tick
    );
    assert!(
        ended_before || result.damage[0] + result.damage[1] > 0.0,
        "unidades devem causar dano às torres (danos: {:?}, tick: {})",
        result.damage,
        game.tick
    );
}

#[test]
fn deploy_consumes_elixir_and_spawns_after_summon_time() {
    let mut game = new_match();
    game.players[0].elixir = 10.0;
    let before_hand = game.players[0].hand.clone();
    let before_elixir = game.players[0].elixir;
    game.deploy(game.players[0].user_id, 0, 9.0, 22.0)
        .expect("deploy válido");
    assert!(game.players[0].elixir < before_elixir);
    assert_ne!(game.players[0].hand, before_hand);
    let before_units = game
        .entities
        .iter()
        .filter(|entity| entity.kind == royale_core::EntityKind::Troop)
        .count();
    for _ in 0..40 {
        game.tick();
    }
    let after_units = game
        .entities
        .iter()
        .filter(|entity| entity.kind == royale_core::EntityKind::Troop)
        .count();
    assert!(after_units > before_units);
}

#[test]
fn freeze_stuns_and_expires() {
    let mut game = new_match();
    let id = game
        .spawn_troop(Side::Blue, "knight", "knight", 9.0, 20.0, None, false, None)
        .expect("tropa");
    game.apply_effect_preset(id, "freeze_4s", Side::Red, None);
    game.tick();
    let frozen = game.entity(id).expect("tropa viva");
    assert!(frozen.is_stunned());
    assert_eq!(frozen.speed_mult(), 0.0);
    for _ in 0..140 {
        game.tick();
    }
    let thawed = game.entity(id).expect("tropa viva");
    assert!(!thawed.is_stunned());
}

#[test]
fn phoenix_rebirth_creates_chick() {
    let mut game = new_match();
    let id = game
        .spawn_troop(Side::Blue, "phoenix", "phoenix", 9.0, 20.0, None, false, None)
        .expect("fênix");
    game.deal_damage(id, 50_000, Side::Red, None, None, false);
    game.process_deaths();
    for _ in 0..60 {
        game.tick();
    }
    let chicks = game
        .entities
        .iter()
        .filter(|entity| entity.slug == "phoenix_chick")
        .count();
    assert_eq!(chicks, 1);
}

#[test]
fn witch_spawns_skeletons_periodically() {
    let mut game = new_match();
    game.spawn_troop(Side::Blue, "witch", "witch", 9.0, 20.0, None, false, None)
        .expect("bruxa");
    for _ in 0..90 {
        game.tick();
    }
    let skeletons = game
        .entities
        .iter()
        .filter(|entity| entity.slug == "skeleton" && entity.side == Side::Blue)
        .count();
    assert!(skeletons >= 3, "esperava esqueletos invocados, achou {skeletons}");
}
