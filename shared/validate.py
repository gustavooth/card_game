#!/usr/bin/env python3
"""Validate the shared definition files and card translations.

Run from anywhere:  python3 shared/validate.py
Exit code 0 means every invariant below holds.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

SHARED = Path(__file__).resolve().parent
ROOT = SHARED.parent
I18N = ROOT / "svelte" / "src" / "lib" / "i18n"

REQUIRED_FILES = [
    "cards.json",
    "troops.json",
    "buildings.json",
    "towers.json",
    "spells.json",
    "projectiles.json",
    "effects.json",
    "skills.json",
    "arenas.json",
]

DECK = {"knight", "archers", "fireball", "cannon", "skeletons", "musketeer", "giant", "arrows"}
FORBIDDEN = {"mirror", "clone", "mother_witch"}
SLUG_RE = re.compile(r"^[a-z][a-z0-9_]*$")
TARGETS = {"ground", "air", "buildings", "towers"}
CARD_KINDS = {"troop", "spell", "building"}
RARITIES = {"common", "rare", "epic", "legendary"}

EFFECT_KINDS = {
    "freeze", "stun", "slow", "snare", "poison", "burn", "shock", "rage",
    "heal", "invisibility", "damage_amp", "speed_boost", "attack_boost",
    "knockback",
}

SKILL_PARAMS = {
    "charge": {"min_distance", "speed_mult", "bonus_damage", "reset_ms", "knockback"},
    "dash": {"trigger_range", "speed_mult", "duration_ms", "bonus_damage"},
    "jump": {"trigger_range", "windup_ms", "damage", "radius"},
    "on_hit_effects": {"effects"},
    "deploy_damage": {"damage", "radius", "effects"},
    "invisibility": {"period_ms", "visible_ms"},
    "burrow": {"travel_ms", "speed_mult", "deploy_anywhere"},
    "hook": {"range", "pull_distance", "damage", "cooldown_ms"},
    "rebirth": {"health", "troop", "delay_ms"},
    "spawn_periodic": {"troop", "count", "interval_ms", "first_ms", "max_alive"},
    "aura": {"radius", "dps", "heal_per_sec", "effects", "affects_allies", "affects_enemies"},
    "reflect": {"percent", "max_range"},
    "ramp": {"max_mult", "interval_ms", "reset_ms"},
    "shield": {"health"},
    "deploy_anywhere": set(),
    "elixir_generate": {"amount", "interval_ms"},
    "elixir_on_death": {"amount"},
    "self_destruct": {"damage", "radius", "effects"},
    "transform_on_death": {"building", "health"},
    "heal_on_hit": {"amount"},
    "charged_shot": {"charge_ms", "damage", "splash_radius", "projectile"},
    "death_effects": {"effects", "radius"},
    "tornado": {"strength", "radius", "duration_ms"},
}

EFFECT_PARAMS = {
    "freeze": {"speed_mult", "attack_mult"},
    "slow": {"speed_mult", "attack_mult"},
    "rage": {"speed_mult", "attack_mult"},
    "speed_boost": {"speed_mult"},
    "attack_boost": {"attack_mult"},
    "poison": {"dps", "tick_ms"},
    "burn": {"dps", "tick_ms"},
    "shock": {"dps", "tick_ms"},
    "heal": {"amount"},
    "knockback": {"amount"},
    "damage_amp": {"damage_mult"},
}

errors: list[str] = []
warnings: list[str] = []


def err(message: str) -> None:
    errors.append(message)


def warn(message: str) -> None:
    warnings.append(message)


def load(name: str):
    path = SHARED / name
    if not path.is_file():
        err(f"arquivo ausente: {path}")
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        err(f"{name}: JSON inválido ({exc.msg}, linha {exc.lineno})")
        return None


def require_keys(obj: dict, keys: set[str], where: str) -> None:
    missing = keys - set(obj)
    if missing:
        err(f"{where}: faltam campos {sorted(missing)}")


def check_slug(slug: str, where: str) -> bool:
    if not isinstance(slug, str) or not SLUG_RE.match(slug):
        err(f"{where}: slug inválido {slug!r} (esperado snake_case)")
        return False
    return True


def expect_type(value, typ, where: str) -> bool:
    if not isinstance(value, typ):
        err(f"{where}: tipo inesperado {type(value).__name__} (esperado {typ.__name__})")
        return False
    return True


# ---------------------------------------------------------------------------
# Load
# ---------------------------------------------------------------------------
data = {}
for filename in REQUIRED_FILES:
    data[filename] = load(filename)

rarities = load("rarities.json")
if any(value is None for value in data.values()):
    for message in errors:
        print(f"ERRO: {message}")
    print("\nValidação interrompida: há arquivos inválidos ou ausentes.")
    sys.exit(1)

cards = data["cards.json"]
troops = data["troops.json"]
buildings = data["buildings.json"]
towers = data["towers.json"]
spells = data["spells.json"]
projectiles = data["projectiles.json"]
effects = data["effects.json"]
skills = data["skills.json"]
arenas = data["arenas.json"]

for name, obj in [
    ("cards.json", cards),
    ("troops.json", troops),
    ("buildings.json", buildings),
    ("towers.json", towers),
    ("spells.json", spells),
    ("projectiles.json", projectiles),
    ("effects.json", effects),
    ("skills.json", skills),
    ("arenas.json", arenas),
]:
    expect_type(obj, list if name == "cards.json" else dict, name)

if errors:
    for message in errors:
        print(f"ERRO: {message}")
    sys.exit(1)

# ---------------------------------------------------------------------------
# Cards
# ---------------------------------------------------------------------------
if len(cards) != 50:
    err(f"cards.json: esperadas exatamente 50 cartas, encontradas {len(cards)}")

seen_cards: set[str] = set()
seen_sort: set[int] = set()
card_slugs: list[str] = []
for index, card in enumerate(cards):
    where = f"cards[{index}]"
    if not isinstance(card, dict):
        err(f"{where}: deve ser objeto")
        continue

    require_keys(
        card,
        {"slug", "rarity", "elixir_cost", "summon_time_ms", "type", "tags", "sort"},
        where,
    )
    slug = card.get("slug", "")
    if not check_slug(slug, where):
        continue
    if slug in seen_cards:
        err(f"cards.json: slug duplicado {slug!r}")
    seen_cards.add(slug)
    card_slugs.append(slug)

    if slug in FORBIDDEN:
        err(f"cards.json: carta proibida pelo escopo: {slug!r}")

    rarity = card.get("rarity")
    if rarity not in RARITIES:
        err(f"{where} ({slug}): raridade inválida {rarity!r}")
    if rarities and isinstance(rarities, dict) and rarity not in rarities:
        err(f"{where} ({slug}): raridade {rarity!r} não existe em rarities.json")

    elixir = card.get("elixir_cost")
    if not isinstance(elixir, int) or not 1 <= elixir <= 10:
        err(f"{where} ({slug}): elixir_cost deve ser inteiro entre 1 e 10")

    summon = card.get("summon_time_ms")
    if not isinstance(summon, int) or summon < 0:
        err(f"{where} ({slug}): summon_time_ms deve ser inteiro >= 0")

    sort = card.get("sort")
    if not isinstance(sort, int):
        err(f"{where} ({slug}): sort deve ser inteiro")
    elif sort in seen_sort:
        err(f"cards.json: sort duplicado {sort}")
    else:
        seen_sort.add(sort)

    if not isinstance(card.get("tags"), list) or not all(isinstance(t, str) for t in card["tags"]):
        err(f"{where} ({slug}): tags deve ser lista de strings")

    ctype = card.get("type")
    if not isinstance(ctype, dict):
        err(f"{where} ({slug}): type deve ser objeto")
        continue
    kind = ctype.get("kind")
    if kind not in CARD_KINDS:
        err(f"{where} ({slug}): type.kind inválido {kind!r}")
        continue

    if kind == "troop":
        require_keys(ctype, {"kind", "troop", "count"}, f"{where}.type")
        troop_slug = ctype.get("troop")
        if troop_slug not in troops:
            err(f"{where} ({slug}): tropa {troop_slug!r} não existe em troops.json")
        count = ctype.get("count")
        if not isinstance(count, int) or count < 1:
            err(f"{where} ({slug}): count deve ser inteiro >= 1")
    elif kind == "spell":
        require_keys(ctype, {"kind", "spell"}, f"{where}.type")
        spell_slug = ctype.get("spell")
        if spell_slug not in spells:
            err(f"{where} ({slug}): feitiço {spell_slug!r} não existe em spells.json")
    else:
        require_keys(ctype, {"kind", "building"}, f"{where}.type")
        building_slug = ctype.get("building")
        if building_slug not in buildings:
            err(f"{where} ({slug}): construção {building_slug!r} não existe em buildings.json")

missing_deck = DECK - seen_cards
if missing_deck:
    err(f"cards.json: deck padrão incompleto, faltam {sorted(missing_deck)}")

# ---------------------------------------------------------------------------
# Definition maps (slug uniqueness / object keys)
# ---------------------------------------------------------------------------
for filename, obj in [
    ("troops.json", troops),
    ("buildings.json", buildings),
    ("towers.json", towers),
    ("spells.json", spells),
    ("projectiles.json", projectiles),
    ("effects.json", effects),
    ("skills.json", skills),
    ("arenas.json", arenas),
]:
    for key, value in obj.items():
        check_slug(key, f"{filename}[{key!r}]")
        if isinstance(value, dict):
            if "slug" in value and value["slug"] != key:
                err(f"{filename}: chave {key!r} difere de slug {value['slug']!r}")
        else:
            err(f"{filename}[{key!r}]: valor deve ser objeto")

# ---------------------------------------------------------------------------
# Reference helpers
# ---------------------------------------------------------------------------
def check_effect_ref(ref: str, where: str) -> None:
    if ref not in effects:
        err(f"{where}: efeito {ref!r} não existe em effects.json")


def check_skill_ref(ref: str, where: str) -> None:
    if ref not in skills:
        err(f"{where}: skill {ref!r} não existe em skills.json")


def check_projectile_ref(ref, where: str) -> None:
    if ref is not None and ref not in projectiles:
        err(f"{where}: projétil {ref!r} não existe em projectiles.json")


def check_troop_ref(ref, where: str) -> None:
    if ref not in troops:
        err(f"{where}: tropa {ref!r} não existe em troops.json")


def check_skill_payload(skill_slug: str, where: str) -> None:
    preset = skills[skill_slug]
    if not isinstance(preset, dict):
        return
    kind = preset.get("kind")
    if kind not in SKILL_PARAMS:
        err(f"{where}: kind de skill não suportado {kind!r}")
        return
    params = preset.get("params", {})
    if not isinstance(params, dict):
        err(f"{where}: params deve ser objeto")
        return
    missing = SKILL_PARAMS[kind] - set(params)
    if missing:
        err(f"{where}: skill {skill_slug!r} ({kind}) sem params {sorted(missing)}")

    for effect_ref in params.get("effects", []) or []:
        check_effect_ref(effect_ref, where)
    if "troop" in params:
        check_troop_ref(params["troop"], where)
    if "building" in params:
        if params["building"] not in buildings:
            err(f"{where}: construção {params['building']!r} não existe em buildings.json")
    if "projectile" in params:
        check_projectile_ref(params["projectile"], where)


def check_attack(stats: dict, where: str) -> None:
    atk = stats.get("attack")
    if not isinstance(atk, dict):
        err(f"{where}: attack deve ser objeto")
        return
    require_keys(
        atk,
        {
            "range", "damage", "cooldown_ms", "windup_ms", "first_hit_ms",
            "splash_radius", "projectile", "projectile_speed", "pierce",
            "chain", "knockback", "multi_shot", "targets_air",
        },
        f"{where}.attack",
    )
    check_projectile_ref(atk.get("projectile"), f"{where}.attack")
    if atk.get("damage", 0) < 0:
        err(f"{where}.attack: damage não pode ser negativo")
    if atk.get("cooldown_ms", 0) < 0:
        err(f"{where}.attack: cooldown_ms não pode ser negativo")


def check_common_unit(stats: dict, where: str, with_speed: bool) -> None:
    require_keys(
        stats,
        {"slug", "health", "radius", "targets", "sight_range", "attack", "skills", "death", "visual"},
        where,
    )
    if with_speed:
        require_keys(stats, {"speed", "mass", "flying"}, where)

    targets = stats.get("targets")
    if not isinstance(targets, list) or not set(targets).issubset(TARGETS):
        err(f"{where}: targets inválidos {targets!r}")

    for target in targets or []:
        if target not in TARGETS:
            err(f"{where}: alvo desconhecido {target!r}")

    check_attack(stats, where)

    for index, skill_ref in enumerate(stats.get("skills", [])):
        if not isinstance(skill_ref, dict) or "skill" not in skill_ref:
            err(f"{where}.skills[{index}]: deve ser objeto com campo 'skill'")
            continue
        skill_slug = skill_ref["skill"]
        check_skill_ref(skill_slug, f"{where}.skills[{index}]")
        if skill_slug in skills:
            check_skill_payload(skill_slug, f"{where}.skills[{index}]")

    dth = stats.get("death")
    if not isinstance(dth, dict):
        err(f"{where}: death deve ser objeto")
    else:
        require_keys(dth, {"damage", "radius", "effects", "spawn"}, f"{where}.death")
        for effect_ref in dth.get("effects", []) or []:
            check_effect_ref(effect_ref, f"{where}.death")
        for index, spawn_spec in enumerate(dth.get("spawn", []) or []):
            if not isinstance(spawn_spec, dict):
                err(f"{where}.death.spawn[{index}]: deve ser objeto")
                continue
            require_keys(spawn_spec, {"troop", "count", "delay_ms"}, f"{where}.death.spawn[{index}]")
            check_troop_ref(spawn_spec.get("troop"), f"{where}.death.spawn[{index}]")

    vis = stats.get("visual")
    if not isinstance(vis, dict):
        err(f"{where}: visual deve ser objeto")
    else:
        require_keys(
            vis,
            {"archetype", "scale", "primary", "secondary", "weapon", "accessory"},
            f"{where}.visual",
        )
        if vis.get("archetype") not in {
            "humanoid", "goblin", "skeleton", "giant", "dragon", "flyer",
            "spirit", "beast", "building", "machine",
        }:
            err(f"{where}.visual: archetype inválido {vis.get('archetype')!r}")
        if vis.get("weapon") not in {"sword", "axe", "spear", "bow", "staff", "hammer", "bomb", "none"}:
            err(f"{where}.visual: weapon inválido {vis.get('weapon')!r}")


# ---------------------------------------------------------------------------
# Troops / buildings / towers
# ---------------------------------------------------------------------------
for slug, stats in troops.items():
    where = f"troops[{slug}]"
    if not isinstance(stats, dict):
        err(f"{where}: deve ser objeto")
        continue
    if stats.get("slug") != slug:
        err(f"{where}: slug divergente")
    if not isinstance(stats.get("health"), int) or stats["health"] < 1:
        err(f"{where}: health deve ser inteiro >= 1")
    if not isinstance(stats.get("speed"), (int, float)) or stats["speed"] < 0:
        err(f"{where}: speed inválida")
    if not isinstance(stats.get("radius"), (int, float)) or stats["radius"] <= 0:
        err(f"{where}: radius inválido")
    check_common_unit(stats, where, with_speed=True)

for slug, stats in buildings.items():
    where = f"buildings[{slug}]"
    if not isinstance(stats, dict):
        err(f"{where}: deve ser objeto")
        continue
    if stats.get("slug") != slug:
        err(f"{where}: slug divergente")
    if not isinstance(stats.get("health"), int) or stats["health"] < 1:
        err(f"{where}: health deve ser inteiro >= 1")
    if not isinstance(stats.get("lifetime_ms"), int) or stats["lifetime_ms"] <= 0:
        err(f"{where}: lifetime_ms deve ser inteiro > 0")
    check_common_unit(stats, where, with_speed=False)

for slug, stats in towers.items():
    where = f"towers[{slug}]"
    if not isinstance(stats, dict):
        err(f"{where}: deve ser objeto")
        continue
    require_keys(stats, {"slug", "health", "radius", "attack", "visual"}, where)
    check_attack(stats, where)
    if not isinstance(stats.get("health"), int) or stats["health"] < 1:
        err(f"{where}: health deve ser inteiro >= 1")

for required_tower in ("princess_tower", "king_tower"):
    if required_tower not in towers:
        err(f"towers.json: torre obrigatória {required_tower!r} ausente")

# ---------------------------------------------------------------------------
# Skills / effects
# ---------------------------------------------------------------------------
for slug, preset in skills.items():
    if not isinstance(preset, dict):
        continue
    require_keys(preset, {"kind", "params"}, f"skills[{slug}]")
    kind = preset.get("kind")
    if kind not in SKILL_PARAMS:
        err(f"skills[{slug}]: kind não suportado {kind!r}")
    else:
        params = preset.get("params", {})
        if not isinstance(params, dict):
            err(f"skills[{slug}]: params deve ser objeto")
        else:
            missing = SKILL_PARAMS[kind] - set(params)
            if missing:
                err(f"skills[{slug}]: faltam params {sorted(missing)}")
            for effect_ref in params.get("effects", []) or []:
                check_effect_ref(effect_ref, f"skills[{slug}]")
            if "troop" in params:
                check_troop_ref(params["troop"], f"skills[{slug}]")
            if "building" in params and params["building"] not in buildings:
                err(f"skills[{slug}]: construção {params['building']!r} inexistente")
            if "projectile" in params:
                check_projectile_ref(params["projectile"], f"skills[{slug}]")

for slug, preset in effects.items():
    if not isinstance(preset, dict):
        continue
    require_keys(preset, {"kind", "duration_ms", "params", "visual"}, f"effects[{slug}]")
    kind = preset.get("kind")
    if kind not in EFFECT_KINDS:
        err(f"effects[{slug}]: kind não suportado {kind!r}")
        continue
    params = preset.get("params", {})
    if not isinstance(params, dict):
        err(f"effects[{slug}]: params deve ser objeto")
        continue
    missing = EFFECT_PARAMS.get(kind, set()) - set(params)
    if missing:
        err(f"effects[{slug}]: faltam params {sorted(missing)}")
    if not isinstance(preset.get("duration_ms"), int) or preset["duration_ms"] < 0:
        err(f"effects[{slug}]: duration_ms deve ser inteiro >= 0")

# ---------------------------------------------------------------------------
# Spells
# ---------------------------------------------------------------------------
for slug, spell in spells.items():
    where = f"spells[{slug}]"
    if not isinstance(spell, dict):
        err(f"{where}: deve ser objeto")
        continue
    require_keys(
        spell,
        {
            "slug", "radius", "delay_ms", "damage", "tower_damage_mult",
            "targets", "effects", "zone", "spawn", "rolling", "pull", "visual",
        },
        where,
    )
    if spell.get("slug") != slug:
        err(f"{where}: slug divergente")
    if not isinstance(spell.get("radius"), (int, float)) or spell["radius"] <= 0:
        err(f"{where}: radius inválido")
    if not isinstance(spell.get("delay_ms"), int) or spell["delay_ms"] < 0:
        err(f"{where}: delay_ms inválido")
    if not isinstance(spell.get("damage"), int) or spell["damage"] < 0:
        err(f"{where}: damage inválido")

    targets = spell.get("targets")
    if not isinstance(targets, list) or not set(targets).issubset(TARGETS):
        err(f"{where}: targets inválidos {targets!r}")

    for effect_ref in spell.get("effects", []) or []:
        check_effect_ref(effect_ref, f"{where}.effects")

    zone = spell.get("zone")
    if zone is not None:
        if not isinstance(zone, dict):
            err(f"{where}.zone: deve ser objeto ou null")
        else:
            require_keys(zone, {"duration_ms", "dps", "effect", "slow_mult"}, f"{where}.zone")
            check_effect_ref(zone.get("effect"), f"{where}.zone")

    spawn_spec = spell.get("spawn")
    if spawn_spec is not None:
        if not isinstance(spawn_spec, dict):
            err(f"{where}.spawn: deve ser objeto ou null")
        else:
            require_keys(
                spawn_spec, {"troop", "count", "interval_ms", "radius"}, f"{where}.spawn"
            )
            check_troop_ref(spawn_spec.get("troop"), f"{where}.spawn")
            if not isinstance(spawn_spec.get("count"), int) or spawn_spec["count"] < 1:
                err(f"{where}.spawn: count deve ser inteiro >= 1")

    rolling = spell.get("rolling")
    if rolling is not None:
        if not isinstance(rolling, dict):
            err(f"{where}.rolling: deve ser objeto ou null")
        else:
            require_keys(
                rolling, {"distance", "speed", "width", "knockback"}, f"{where}.rolling"
            )

    pull = spell.get("pull")
    if pull is not None:
        if not isinstance(pull, dict):
            err(f"{where}.pull: deve ser objeto ou null")
        else:
            require_keys(pull, {"strength", "duration_ms"}, f"{where}.pull")

    vis = spell.get("visual")
    if not isinstance(vis, dict):
        err(f"{where}.visual: deve ser objeto")
    else:
        require_keys(vis, {"variant", "color"}, f"{where}.visual")

# ---------------------------------------------------------------------------
# Projectiles / arenas
# ---------------------------------------------------------------------------
for slug, projectile in projectiles.items():
    where = f"projectiles[{slug}]"
    require_keys(projectile, {"slug", "speed", "radius", "homing", "visual"}, where)
    if projectile.get("slug") != slug:
        err(f"{where}: slug divergente")
    if not isinstance(projectile.get("speed"), (int, float)) or projectile["speed"] <= 0:
        err(f"{where}: speed inválida")

classic = arenas.get("classic")
if not isinstance(classic, dict):
    err("arenas.json: arena 'classic' ausente")
else:
    require_keys(
        classic,
        {"slug", "width", "height", "river_top", "river_bottom", "bridges", "tower_positions"},
        "arenas[classic]",
    )
    if classic.get("width") != 18 or classic.get("height") != 32:
        err("arenas[classic]: dimensões devem ser 18x32")
    if not (classic.get("river_top") == 15.5 and classic.get("river_bottom") == 16.5):
        err("arenas[classic]: rio deve ficar entre 15.5 e 16.5")
    bridges = classic.get("bridges")
    if not isinstance(bridges, list) or len(bridges) != 2:
        err("arenas[classic]: deve haver exatamente 2 pontes")
    positions = classic.get("tower_positions")
    if not isinstance(positions, dict) or set(positions) != {"top", "bottom"}:
        err("arenas[classic]: tower_positions deve ter top e bottom")
    else:
        for side, layout in positions.items():
            if not isinstance(layout, dict) or set(layout) != {"king", "princess"}:
                err(f"arenas[classic].tower_positions.{side}: layout inválido")
                continue
            if not isinstance(layout["king"], list) or len(layout["king"]) != 2:
                err(f"arenas[classic].tower_positions.{side}.king: coordenada inválida")
            princess = layout["princess"]
            if not isinstance(princess, list) or len(princess) != 2:
                err(f"arenas[classic].tower_positions.{side}.princess: devem ser 2 torres")

# ---------------------------------------------------------------------------
# Translations
# ---------------------------------------------------------------------------
translations = {}
for lang, filename in (("pt-BR", "cards.pt-BR.json"), ("en", "cards.en.json")):
    path = I18N / filename
    if not path.is_file():
        err(f"tradução ausente: {path}")
        translations[lang] = None
        continue
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        err(f"{filename}: JSON inválido ({exc.msg}, linha {exc.lineno})")
        translations[lang] = None
        continue
    if not isinstance(payload, dict):
        err(f"{filename}: deve ser um objeto indexado por slug")
        translations[lang] = None
        continue
    translations[lang] = payload

    missing = seen_cards - set(payload)
    if missing:
        err(f"{filename}: faltam traduções para {sorted(missing)}")
    extra = set(payload) - seen_cards
    if extra:
        warn(f"{filename}: traduções sem carta correspondente {sorted(extra)}")

    for slug, entry in payload.items():
        if not isinstance(entry, dict):
            err(f"{filename}[{slug}]: deve ser objeto com name/description")
            continue
        name = entry.get("name")
        description = entry.get("description")
        if not isinstance(name, str) or not name.strip():
            err(f"{filename}[{slug}]: name vazio")
        if not isinstance(description, str) or not description.strip():
            err(f"{filename}[{slug}]: description vazia")

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
counts = {"troop": 0, "spell": 0, "building": 0}
for card in cards:
    kind = card.get("type", {}).get("kind")
    if kind in counts:
        counts[kind] += 1

print("Resumo das definições")
print(f"  cartas:        {len(cards)}")
print(f"  tropas:        {len(troops)} ({counts['troop']} cartas de tropa)")
print(f"  feitiços:      {len(spells)} ({counts['spell']} cartas de feitiço)")
print(f"  construções:   {len(buildings)} ({counts['building']} cartas de construção)")
print(f"  torres:        {len(towers)}")
print(f"  projéteis:     {len(projectiles)}")
print(f"  efeitos:       {len(effects)}")
print(f"  skills:        {len(skills)}")
print(f"  arenas:        {len(arenas)}")
print(f"  traduções:     {len(card_slugs)} slugs x 2 idiomas")

if warnings:
    print("\nAvisos")
    for message in warnings:
        print(f"  AVISO: {message}")

if errors:
    print(f"\n{len(errors)} erro(s) encontrados")
    for message in errors:
        print(f"  ERRO: {message}")
    sys.exit(1)

print("\nOK: validação concluída sem erros.")
