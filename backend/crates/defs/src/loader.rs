use crate::DefsError;
use crate::catalog::Catalog;
use crate::types::*;
use crate::validate::validate;

#[derive(Debug, Clone)]
pub struct Loaded {
    pub defs: Definitions,
    pub catalog: Catalog,
}

impl Loaded {
    pub fn default_arena(&self) -> Option<&ArenaDef> {
        self.defs
            .arenas
            .get("classic")
            .or_else(|| self.defs.arenas.values().next())
    }
}

macro_rules! json_file {
    ($file:literal) => {
        include_str!(concat!("../../../../shared/", $file))
    };
}

pub fn load() -> Result<Loaded, DefsError> {
    let cards: Vec<CardDef> = parse(json_file!("cards.json"), "cards.json")?;
    let troops: std::collections::BTreeMap<String, TroopDef> =
        parse(json_file!("troops.json"), "troops.json")?;
    let buildings: std::collections::BTreeMap<String, BuildingDef> =
        parse(json_file!("buildings.json"), "buildings.json")?;
    let towers: std::collections::BTreeMap<String, TowerDef> =
        parse(json_file!("towers.json"), "towers.json")?;
    let spells: std::collections::BTreeMap<String, SpellDef> =
        parse(json_file!("spells.json"), "spells.json")?;
    let projectiles: std::collections::BTreeMap<String, ProjectileDef> =
        parse(json_file!("projectiles.json"), "projectiles.json")?;
    let effects: std::collections::BTreeMap<String, EffectDef> =
        parse(json_file!("effects.json"), "effects.json")?;
    let skills: std::collections::BTreeMap<String, SkillDef> =
        parse(json_file!("skills.json"), "skills.json")?;
    let arenas: std::collections::BTreeMap<String, ArenaDef> =
        parse(json_file!("arenas.json"), "arenas.json")?;

    let defs = Definitions {
        cards,
        troops,
        buildings,
        towers,
        spells,
        projectiles,
        effects,
        skills,
        arenas,
    };

    validate(&defs)?;
    let catalog = Catalog::build(&defs);
    Ok(Loaded { defs, catalog })
}

fn parse<T: serde::de::DeserializeOwned>(
    text: &'static str,
    file: &'static str,
) -> Result<T, DefsError> {
    serde_json::from_str(text).map_err(|source| DefsError::Json { file, source })
}
