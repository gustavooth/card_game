use std::collections::HashSet;

use crate::DefsError;
use crate::types::*;

pub const REQUIRED_DECK: [&str; 8] = [
    "knight",
    "archers",
    "fireball",
    "cannon",
    "skeletons",
    "musketeer",
    "giant",
    "arrows",
];

pub fn validate(defs: &Definitions) -> Result<(), DefsError> {
    if defs.cards.len() != 50 {
        return Err(invalid(format!(
            "esperadas 50 cartas, encontradas {}",
            defs.cards.len()
        )));
    }

    let mut seen = HashSet::new();
    for card in &defs.cards {
        if !seen.insert(card.slug.as_str()) {
            return Err(invalid(format!("carta duplicada: {}", card.slug)));
        }
        match &card.card_type {
            CardType::Troop { troop, count } => {
                if !defs.troops.contains_key(troop) {
                    return Err(invalid(format!(
                        "carta {} referencia tropa inexistente {}",
                        card.slug, troop
                    )));
                }
                if *count == 0 {
                    return Err(invalid(format!("carta {} tem count 0", card.slug)));
                }
            }
            CardType::Spell { spell } => {
                if !defs.spells.contains_key(spell) {
                    return Err(invalid(format!(
                        "carta {} referencia feitiço inexistente {}",
                        card.slug, spell
                    )));
                }
            }
            CardType::Building { building } => {
                if !defs.buildings.contains_key(building) {
                    return Err(invalid(format!(
                        "carta {} referencia construção inexistente {}",
                        card.slug, building
                    )));
                }
            }
        }
        if card.elixir_cost == 0 || card.elixir_cost > 10 {
            return Err(invalid(format!(
                "carta {} tem custo de elixir inválido ({})",
                card.slug, card.elixir_cost
            )));
        }
    }

    for slug in REQUIRED_DECK {
        if !seen.contains(slug) {
            return Err(invalid(format!("carta obrigatória ausente: {slug}")));
        }
    }

    for (slug, troop) in &defs.troops {
        check_attack(defs, slug, Some(&troop.attack))?;
        check_targets(slug, &troop.targets)?;
        check_death(defs, slug, &troop.death)?;
        check_skills(defs, slug, &troop.skills)?;
    }

    for (slug, building) in &defs.buildings {
        check_attack(defs, slug, building.attack.as_ref())?;
        if building
            .attack
            .as_ref()
            .is_some_and(|attack| attack.damage > 0)
        {
            check_targets(slug, &building.targets)?;
        }
        check_death(defs, slug, &building.death)?;
        check_skills(defs, slug, &building.skills)?;
        if building.health <= 0 {
            return Err(invalid(format!("construção {slug} sem vida")));
        }
    }

    for (slug, tower) in &defs.towers {
        check_attack(defs, slug, Some(&tower.attack))?;
        if tower.health <= 0 {
            return Err(invalid(format!("torre {slug} sem vida")));
        }
    }

    if !defs.towers.contains_key("princess_tower") || !defs.towers.contains_key("king_tower") {
        return Err(invalid(
            "towers.json deve conter princess_tower e king_tower".into(),
        ));
    }

    for (slug, spell) in &defs.spells {
        if spell.radius <= 0.0 {
            return Err(invalid(format!("feitiço {slug} sem raio")));
        }
        check_targets(slug, &spell.targets)?;
        for effect in &spell.effects {
            if !defs.effects.contains_key(effect) {
                return Err(invalid(format!(
                    "feitiço {slug} referencia efeito inexistente {effect}"
                )));
            }
        }
        if let Some(zone) = &spell.zone {
            for effect in &zone.effects {
                if !defs.effects.contains_key(effect) {
                    return Err(invalid(format!(
                        "zona de {slug} referencia efeito inexistente {effect}"
                    )));
                }
            }
            if let Some(effect) = &zone.effect {
                if !defs.effects.contains_key(effect) {
                    return Err(invalid(format!(
                        "zona de {slug} referencia efeito inexistente {effect}"
                    )));
                }
            }
        }
        if let Some(spawn) = &spell.spawn {
            if !defs.troops.contains_key(&spawn.troop) {
                return Err(invalid(format!(
                    "feitiço {slug} invoca tropa inexistente {}",
                    spawn.troop
                )));
            }
        }
    }

    for (slug, effect) in &defs.effects {
        if effect.duration_ms == 0
            && !matches!(
                effect.kind,
                EffectKind::DamageAmp | EffectKind::SpeedBoost | EffectKind::AttackBoost
            )
        {
            return Err(invalid(format!("efeito {slug} com duração 0")));
        }
    }

    for (slug, skill) in &defs.skills {
        for effect in &skill.params.effects {
            if !defs.effects.contains_key(effect) {
                return Err(invalid(format!(
                    "skill {slug} referencia efeito inexistente {effect}"
                )));
            }
        }
        if let Some(troop) = &skill.params.troop {
            if !defs.troops.contains_key(troop) {
                return Err(invalid(format!(
                    "skill {slug} referencia tropa inexistente {troop}"
                )));
            }
        }
        if let Some(building) = &skill.params.building {
            if !defs.buildings.contains_key(building) {
                return Err(invalid(format!(
                    "skill {slug} referencia construção inexistente {building}"
                )));
            }
        }
        if let Some(projectile) = &skill.params.projectile {
            if !defs.projectiles.contains_key(projectile) {
                return Err(invalid(format!(
                    "skill {slug} referencia projétil inexistente {projectile}"
                )));
            }
        }
    }

    let arena = defs
        .arenas
        .get("classic")
        .ok_or_else(|| invalid("arenas.json deve conter a arena classic".into()))?;
    if arena.bridges.len() != 2 {
        return Err(invalid("arena classic deve ter duas pontes".into()));
    }
    if arena.tower_positions.top.princess.len() != 2
        || arena.tower_positions.bottom.princess.len() != 2
    {
        return Err(invalid(
            "arena classic deve ter duas torres princesa por lado".into(),
        ));
    }

    Ok(())
}

fn check_attack(
    defs: &Definitions,
    slug: &str,
    attack: Option<&AttackDef>,
) -> Result<(), DefsError> {
    if let Some(attack) = attack {
        if attack.damage < 0 || (attack.damage > 0 && attack.cooldown_ms == 0) {
            return Err(invalid(format!("{slug} tem ataque inválido")));
        }
        if let Some(projectile) = &attack.projectile {
            if !defs.projectiles.contains_key(projectile) {
                return Err(invalid(format!(
                    "{slug} referencia projétil inexistente {projectile}"
                )));
            }
        }
    }
    Ok(())
}

fn check_targets(slug: &str, targets: &[TargetKind]) -> Result<(), DefsError> {
    if targets.is_empty() {
        return Err(invalid(format!("{slug} sem alvos definidos")));
    }
    Ok(())
}

fn check_death(defs: &Definitions, slug: &str, death: &DeathDef) -> Result<(), DefsError> {
    for effect in &death.effects {
        if !defs.effects.contains_key(effect) {
            return Err(invalid(format!(
                "{slug} referencia efeito de morte inexistente {effect}"
            )));
        }
    }
    for spawn in &death.spawn {
        if !defs.troops.contains_key(&spawn.troop) {
            return Err(invalid(format!(
                "{slug} invoca tropa inexistente na morte: {}",
                spawn.troop
            )));
        }
    }
    Ok(())
}

fn check_skills(defs: &Definitions, slug: &str, skills: &[SkillRef]) -> Result<(), DefsError> {
    for skill in skills {
        if !defs.skills.contains_key(&skill.skill) {
            return Err(invalid(format!(
                "{slug} referencia skill inexistente {}",
                skill.skill
            )));
        }
    }
    Ok(())
}

fn invalid(message: String) -> DefsError {
    DefsError::Invalid(message)
}
