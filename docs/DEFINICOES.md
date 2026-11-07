# Contrato de definições (`shared/*.json`)

Todos os JSON são compartilhados entre servidor e cliente. O servidor é a fonte da verdade:
carrega, valida e serve as definições em `GET /api/content`.

## Unidades e coordenadas

- Arena em **tiles**: `x ∈ [0, 18]`, `y ∈ [0, 32]`.
- `y = 0` é o topo (lado vermelho), `y = 32` é a base (lado azul).
- Velocidades em **tiles/segundo**; alcances/raios em tiles; tempos em **milissegundos**; vida/dano inteiros.
- Torres: topo em `y≈2.5` (rei) e `y≈6.5` (princesas); base em `y≈29.5` e `y≈25.5`.
- Rio entre `y=15.5` e `y=16.5`; pontes em `x∈[2.5,5.5]` e `x∈[12.5,15.5]`.

## Catálogo

O servidor expõe `catalog` com as listas ordenadas `cards`, `troops`, `buildings`, `towers`,
`spells`, `projectiles`, `effects`, `skills`, `arenas`. Os índices numéricos usados nos
snapshots referenciam essas listas. A ordem é a ordem dos arquivos JSON.

---

## `cards.json` — array

```json
{
  "slug": "knight",
  "rarity": "common",
  "elixir_cost": 3,
  "summon_time_ms": 1000,
  "type": { "kind": "troop", "troop": "knight", "count": 1 },
  "tags": ["ground", "melee"],
  "sort": 1
}
```

- `kind`: `troop` | `spell` | `building`.
- Tropa: `troop` (slug em troops.json), `count` (unidades invocadas), formação automática.
- Feitiço: `{ "kind": "spell", "spell": "fireball" }`.
- Construção: `{ "kind": "building", "building": "cannon" }`.
- `rarity`: `common` | `rare` | `epic` | `legendary`.
- `summon_time_ms`: atraso entre o deploy e o efeito (0 para feitiços instantâneos).

## `troops.json` — objeto por slug

```json
{
  "knight": {
    "slug": "knight",
    "health": 1400,
    "speed": 1.0,
    "radius": 0.32,
    "mass": 2.0,
    "flying": false,
    "targets": ["ground", "air", "buildings", "towers"],
    "sight_range": 5.5,
    "attack": {
      "range": 0.8,
      "damage": 160,
      "cooldown_ms": 1100,
      "windup_ms": 300,
      "first_hit_ms": 300,
      "splash_radius": 0.0,
      "projectile": null,
      "projectile_speed": 0.0,
      "pierce": false,
      "chain": 0,
      "knockback": 0.0,
      "multi_shot": 0,
      "targets_air": false
    },
    "skills": [{ "skill": "knight_charge" }],
    "death": { "damage": 0, "radius": 0.0, "effects": [], "spawn": [] },
    "visual": {
      "archetype": "humanoid",
      "scale": 1.0,
      "primary": "#64748b",
      "secondary": "#e2e8f0",
      "weapon": "sword",
      "accessory": "cape"
    }
  }
}
```

- `targets`: subconjunto de `ground`, `air`, `buildings`, `towers`. Tropas sem `ground`/`air`
  ignoram unidades e atacam apenas construções/torres.
- `attack.range = 0.8` significa corpo a corpo.
- `attack.projectile`: slug de projectiles.json ou `null` (dano instantâneo).
- `attack.chain`: quantidade de saltos (0 = sem corrente).
- `attack.multi_shot`: projéteis simultâneos (0/1 = um).
- `attack.targets_air`: mantido por compatibilidade; use `targets`.
- `death.spawn`: lista de `{ "troop": "slug", "count": 2, "delay_ms": 0 }`.
- `visual.archetype`: `humanoid` | `goblin` | `skeleton` | `giant` | `dragon` | `flyer` |
  `spirit` | `beast` | `building` | `machine`.
- `visual.weapon`: `sword` | `axe` | `spear` | `bow` | `staff` | `hammer` | `bomb` | `none`.

## `buildings.json` — objeto por slug

```json
{
  "cannon": {
    "slug": "cannon",
    "health": 900,
    "lifetime_ms": 30000,
    "radius": 0.6,
    "targets": ["ground", "buildings", "towers"],
    "sight_range": 7.0,
    "attack": { "...": "mesmo formato de troops" },
    "skills": [{ "skill": "building_shield" }],
    "death": { "...": "mesmo formato de troops" },
    "visual": { "...": "mesmo formato de troops" }
  }
}
```

Construções com `spawn_periodic` em `skills[]` invocam tropas periodicamente.

## `towers.json` — objeto por slug

```json
{
  "princess_tower": {
    "slug": "princess_tower",
    "health": 2400,
    "radius": 1.2,
    "attack": { "...": "mesmo formato" },
    "visual": { "archetype": "building", "scale": 1.0, "primary": "#3b82f6", "secondary": "#94a3b8", "weapon": "none", "accessory": "princess" }
  },
  "king_tower": { "slug": "king_tower", "health": 4000, "radius": 1.5, "attack": { "...": "..." }, "visual": { "...": "..." } }
}
```

## `spells.json` — objeto por slug

```json
{
  "fireball": {
    "slug": "fireball",
    "radius": 2.5,
    "delay_ms": 500,
    "damage": 700,
    "tower_damage_mult": 0.35,
    "targets": ["ground", "air", "buildings", "towers"],
    "effects": ["knockback_1"],
    "zone": null,
    "spawn": null,
    "rolling": null,
    "pull": null,
    "visual": { "variant": "fireball", "color": "#f97316" }
  }
}
```

- `zone`: `{ "duration_ms": 8000, "dps": 80, "effect": "poison_dot", "slow_mult": 0.0 }`
- `spawn`: `{ "troop": "skeleton", "count": 10, "interval_ms": 350, "radius": 3.0 }`
- `rolling`: `{ "distance": 8.0, "speed": 2.5, "width": 1.2, "knockback": 1.2 }`
- `pull`: `{ "strength": 2.5, "duration_ms": 1000 }`
- `effects`: presets de effects.json aplicados a todos os alvos válidos.
- `targets`: quais categorias o feitiço atinge (unidades aéreas, construções, torres).

## `projectiles.json` — objeto por slug

```json
{
  "arrow": { "slug": "arrow", "speed": 9.0, "radius": 0.1, "homing": true, "visual": "arrow" },
  "fireball_projectile": { "slug": "fireball_projectile", "speed": 7.0, "radius": 0.2, "homing": true, "visual": "ball" }
}
```

## `effects.json` — presets por slug

```json
{
  "freeze_4s": {
    "kind": "freeze",
    "duration_ms": 4000,
    "params": { "speed_mult": 0.0, "attack_mult": 0.0 },
    "visual": "ice"
  }
}
```

`kind` suportados: `freeze`, `stun`, `slow`, `snare`, `poison`, `burn`, `shock`, `rage`,
`heal`, `invisibility`, `damage_amp`, `speed_boost`, `attack_boost`, `knockback`.

`params` usados conforme o kind:
- `speed_mult`, `attack_mult` — freeze/slow/rage/speed_boost/attack_boost.
- `dps`, `tick_ms` — poison/burn/shock.
- `amount`, `tick_ms` — heal (cura por tick) e knockback (distância em tiles).
- `damage_mult` — damage_amp.
- `element` — `ice` | `poison` | `fire` | `lightning`.

## `skills.json` — presets por slug

```json
{
  "prince_charge": {
    "kind": "charge",
    "params": { "min_distance": 1.2, "speed_mult": 1.8, "bonus_damage": 220, "reset_ms": 1500 }
  }
}
```

`kind` suportados e params:

| kind | params |
| --- | --- |
| `charge` | `min_distance`, `speed_mult`, `bonus_damage`, `reset_ms`, `knockback` |
| `dash` | `trigger_range`, `speed_mult`, `duration_ms`, `bonus_damage` |
| `jump` | `trigger_range`, `windup_ms`, `damage`, `radius` |
| `on_hit_effects` | `effects` (lista de presets) |
| `deploy_damage` | `damage`, `radius`, `effects` |
| `invisibility` | `period_ms`, `visible_ms` |
| `burrow` | `travel_ms`, `speed_mult`, `deploy_anywhere` |
| `hook` | `range`, `pull_distance`, `damage`, `cooldown_ms` |
| `rebirth` | `health`, `troop`, `delay_ms` |
| `spawn_periodic` | `troop`, `count`, `interval_ms`, `first_ms`, `max_alive` |
| `aura` | `radius`, `dps`, `heal_per_sec`, `effects`, `affects_allies`, `affects_enemies` |
| `reflect` | `percent`, `max_range` |
| `ramp` | `max_mult`, `interval_ms`, `reset_ms` |
| `shield` | `health` |
| `deploy_anywhere` | `{}` |
| `elixir_generate` | `amount`, `interval_ms` |
| `elixir_on_death` | `amount` |
| `self_destruct` | `damage`, `radius`, `effects` |
| `transform_on_death` | `building`, `health` |
| `heal_on_hit` | `amount` |
| `charged_shot` | `charge_ms`, `damage`, `splash_radius`, `projectile` |
| `death_effects` | `effects`, `radius` (equivale a death.effects) |
| `tornado` | `strength`, `radius`, `duration_ms` |

## `arenas.json`

```json
{
  "classic": {
    "slug": "classic",
    "width": 18,
    "height": 32,
    "river_top": 15.5,
    "river_bottom": 16.5,
    "bridges": [[2.5, 5.5], [12.5, 15.5]],
    "tower_positions": {
      "top": { "king": [9.0, 2.5], "princess": [[3.5, 6.5], [14.5, 6.5]] },
      "bottom": { "king": [9.0, 29.5], "princess": [[3.5, 25.5], [14.5, 25.5]] }
    }
  }
}
```

## Regras de integridade

1. Todo `card` referencia `troop`/`building`/`spell` existente.
2. Toda tropa/construção referencia `projectile`, `effect` e `skill` existentes.
3. `spells[].effects` e `skills[].params.effects` referenciam presets de `effects.json`.
4. Slugs únicos e em `snake_case`.
5. `catalog` mantém a ordem dos arquivos; não reordene JSON depois de gerar snapshots.
6. Exatamente 50 cartas, sem duplicatas de slug, sem campeões/evoluções/Mirror/Clone/Mother Witch.
