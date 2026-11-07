# Cartas e estruturas de dados

Status: 50 cartas fechadas, implementadas apenas com dados em `shared/` e traduções no frontend.

- Sem campeões, evoluções, Mirror, Clone ou Mother Witch.
- Composição: **30 tropas + 12 feitiços + 8 construções = 50 cartas**.
- Nomes e descrições existem somente em `svelte/src/lib/i18n/cards.pt-BR.json` e
  `svelte/src/lib/i18n/cards.en.json`, indexados pelo slug.
- Torres: princesa com **2400 HP** e rei com **4000 HP**.
- Arena clássica **18x32**, velocidades em tiles/s, tempos em milissegundos.

## 1. Deck padrão

As oito cartas obrigatórias estão presentes e marcadas com a tag `deck` em `cards.json`:

`knight`, `archers`, `musketeer`, `skeletons`, `giant`, `fireball`, `cannon`, `arrows`.

## 2. As 50 cartas

| # | Slug | Nome (pt-BR) | Tipo | Custo | Raridade | Mecânica principal |
| ---: | --- | --- | --- | ---: | --- | --- |
| 1 | `knight` | Cavaleiro | tropa | 3 | comum | Corpo a corpo resistente |
| 2 | `archers` | Arqueiras | tropa | 3 | comum | 2 unidades à distância, alvo aéreo |
| 3 | `musketeer` | Mosqueteira | tropa | 4 | rara | À distância de longo alcance |
| 4 | `skeletons` | Esqueletos | tropa | 1 | comum | Enxame de 3 unidades |
| 5 | `giant` | Gigante | tropa | 5 | rara | Tanque que só ataca construções/torres |
| 6 | `mini_pekka` | Mini P.E.K.K.A | tropa | 4 | rara | Dano alto de alvo único |
| 7 | `hog_rider` | Cavaleiro de Javali | tropa | 4 | rara | Salto + só construções/torres |
| 8 | `valkyrie` | Valquíria | tropa | 4 | rara | Dano em área corpo a corpo |
| 9 | `baby_dragon` | Bebê Dragão | tropa | 4 | épica | Voador com ataque em área |
| 10 | `balloon` | Balão | tropa | 5 | épica | Voador, só construções, dano na morte |
| 11 | `witch` | Bruxa | tropa | 5 | épica | Invocação periódica de esqueletos |
| 12 | `prince` | Príncipe | tropa | 5 | épica | Carga com dano bônus |
| 13 | `bandit` | Bandida | tropa | 3 | lendária | Dash |
| 14 | `royal_ghost` | Fantasma Real | tropa | 3 | lendária | Invisibilidade periódica |
| 15 | `fisherman` | Pescador | tropa | 3 | lendária | Gancho que puxa o alvo |
| 16 | `mega_knight` | Mega Cavaleiro | tropa | 7 | lendária | Salto + dano de queda em área |
| 17 | `electro_giant` | Gigante Elétrico | tropa | 7 | épica | Reflexo de dano + só construções |
| 18 | `goblin_giant` | Gigante Goblin | tropa | 6 | épica | Só construções + invocação na morte |
| 19 | `lava_hound` | Sabujo de Lava | tropa | 7 | lendária | Voador + invocação na morte |
| 20 | `phoenix` | Fênix | tropa | 4 | lendária | Renascimento |
| 21 | `inferno_dragon` | Dragão Infernal | tropa | 4 | lendária | Ramp de dano |
| 22 | `sparky` | Faísca | tropa | 6 | lendária | Disparo carregado em área |
| 23 | `battle_ram` | Aríete de Batalha | tropa | 4 | rara | Carga + invocação na morte |
| 24 | `ice_wizard` | Mago de Gelo | tropa | 3 | lendária | Área + lentidão |
| 25 | `electro_wizard` | Mago Elétrico | tropa | 4 | lendária | Corrente elétrica + atordoamento |
| 26 | `guards` | Guardas | tropa | 3 | épica | 3 unidades com escudo |
| 27 | `royal_recruits` | Recrutas Reais | tropa | 7 | comum | 6 unidades com escudo |
| 28 | `wall_breakers` | Quebra-Muros | tropa | 2 | épica | 2 unidades autodestrutivas |
| 29 | `cannon_cart` | Carro-Canão | tropa | 5 | épica | Transformação em construção na morte |
| 30 | `battle_healer` | Curandeira de Batalha | tropa | 4 | rara | Aura de cura + cura ao acertar |
| 31 | `fireball` | Bola de Fogo | feitiço | 4 | rara | Dano em área + empurrão |
| 32 | `arrows` | Flechas | feitiço | 3 | comum | Dano em área ampla |
| 33 | `zap` | Choque | feitiço | 2 | comum | Dano + atordoamento |
| 34 | `lightning` | Raio | feitiço | 6 | épica | Dano alto + atordoamento |
| 35 | `rocket` | Foguete | feitiço | 6 | rara | Dano massivo + empurrão |
| 36 | `poison` | Veneno | feitiço | 4 | épica | Zona de dano contínuo |
| 37 | `freeze` | Congelamento | feitiço | 4 | épica | Congelamento em área |
| 38 | `tornado` | Tornado | feitiço | 3 | épica | Puxão para o centro |
| 39 | `log` | Tronco | feitiço | 2 | lendária | Rolagem + empurrão |
| 40 | `graveyard` | Cemitério | feitiço | 5 | lendária | Invocação contínua de esqueletos |
| 41 | `rage` | Fúria | feitiço | 2 | épica | Buff de velocidade e ataque |
| 42 | `earthquake` | Terremoto | feitiço | 3 | rara | Zona + lentidão + dano a construções |
| 43 | `cannon` | Canhão | construção | 3 | comum | Defesa terrestre |
| 44 | `tesla` | Tesla | construção | 4 | comum | Defesa terrestre e aérea |
| 45 | `inferno_tower` | Torre Infernal | construção | 5 | rara | Ramp de dano defensivo |
| 46 | `bomb_tower` | Torre de Bombas | construção | 4 | rara | Splash terrestre |
| 47 | `mortar` | Morteiro | construção | 4 | comum | Longo alcance com splash |
| 48 | `goblin_hut` | Cabana Goblin | construção | 5 | rara | Spawner de goblins lanceiros |
| 49 | `tombstone` | Lápide | construção | 3 | rara | Spawner + invocação na morte |
| 50 | `elixir_collector` | Coletor de Elixir | construção | 6 | rara | Geração de elixir |

### 2.1 Unidades auxiliares (não são cartas)

`troops.json` também define quatro unidades invocadas por outras cartas. Elas não aparecem em
`cards.json`, mas são referenciadas por `spawn`, `death.spawn`, `rebirth` e `spawn_periodic`:

| Slug | Invocada por | Papel |
| --- | --- | --- |
| `spear_goblin` | `goblin_giant`, `goblin_hut` | Goblin lanceiro à distância |
| `barbarian` | `battle_ram` | Bárbaro corpo a corpo |
| `lava_pup` | `lava_hound` | Filhote voador |
| `phoenix_chick` | `phoenix` | Fênix renascida |

O esqueleto (`skeleton`) é uma definição única compartilhada pela carta `skeletons`, pela
Bruxa, pelo Cemitério e pela Lápide.

## 3. Presets de `skills.json`

Todos os `kind`s usados estão na lista suportada por `docs/DEFINICOES.md`.

| Preset | `kind` | Usado por | Papel |
| --- | --- | --- | --- |
| `prince_charge` | `charge` | `prince`, `battle_ram` | Carrega após 3 tiles; ×2 velocidade e +400 de dano |
| `bandit_dash` | `dash` | `bandit` | Dash de até 4 tiles; +200 de dano |
| `hog_jump` | `jump` | `hog_rider` | Salto sobre o rio, sem dano |
| `mega_knight_jump` | `jump` | `mega_knight` | Salto com 350 de dano em raio 1,5 |
| `mega_knight_deploy` | `deploy_damage` | `mega_knight` | 400 de dano em raio 2,5 + atordoamento |
| `royal_ghost_invisibility` | `invisibility` | `royal_ghost` | Ciclo de 5000 ms invisível / 1200 ms visível |
| `fisherman_hook` | `hook` | `fisherman` | Alcance 6, puxa 3,5, 150 de dano, 6 s de recarga |
| `phoenix_rebirth` | `rebirth` | `phoenix` | Renasce como `phoenix_chick` com 900 HP após 1,2 s |
| `witch_spawn` | `spawn_periodic` | `witch` | 3 esqueletos a cada 7 s, máximo 9 vivos |
| `inferno_ramp` | `ramp` | `inferno_dragon`, `inferno_tower` | Até ×8 de dano, degrau de 400 ms, reset em 1 s |
| `electro_giant_reflect` | `reflect` | `electro_giant` | Reflete 35% do dano em raio 3,5 |
| `shield_200` | `shield` | `guards`, `royal_recruits` | Escudo de 200 HP por unidade |
| `wall_breakers_self_destruct` | `self_destruct` | `wall_breakers` | 700 de dano em raio 1,8 por unidade |
| `cannon_cart_transform` | `transform_on_death` | `cannon_cart` | Vira a construção `cannon` com 450 HP |
| `sparky_charged_shot` | `charged_shot` | `sparky` | 3,5 s de carga, 1100 de dano em raio 2,2 |
| `ice_wizard_slow` | `on_hit_effects` | `ice_wizard` | Aplica `slow_hit_30` |
| `inferno_burn` | `on_hit_effects` | `inferno_dragon` | Aplica `burn_dot` (fogo) |
| `electro_shock` | `on_hit_effects` | `electro_giant` | Aplica `shock_dot` (eletrização) |
| `electro_wizard_stun` | `on_hit_effects` | `electro_wizard` | Aplica `stun_0_5s` |
| `battle_healer_aura` | `aura` | `battle_healer` | 60 HP/s para aliados em raio 3 |
| `battle_healer_heal` | `heal_on_hit` | `battle_healer` | Cura 80 por golpe |
| `goblin_hut_spawn` | `spawn_periodic` | `goblin_hut` | 1 goblin lanceiro a cada 4,5 s, máximo 3 |
| `tombstone_spawn` | `spawn_periodic` | `tombstone` | 1 esqueleto a cada 3 s, máximo 3 |
| `elixir_collector_generate` | `elixir_generate` | `elixir_collector` | 1 elixir a cada 8,4 s durante 70 s |

## 4. Presets de `effects.json`

| Preset | `kind` | Usado por | Papel |
| --- | --- | --- | --- |
| `freeze_4s` | `freeze` | `freeze` | Congela por 4 s (movimento e ataque em 0) |
| `stun_0_5s` | `stun` | `zap`, `lightning`, `mega_knight_deploy`, `electro_wizard_stun` | Atordoa por 500 ms |
| `knockback_1` | `knockback` | `fireball`, `rocket` | Empurra 1 tile |
| `slow_hit_30` | `slow` | `ice_wizard_slow` | −30% de velocidade e ataque por 2 s |
| `earthquake_slow` | `slow` | `earthquake` | −30% de velocidade e −20% de ataque por 3 s |
| `poison_dot` | `poison` | `poison` | 90 de dano por segundo por 8 s |
| `rage_6s` | `rage` | `rage` | +35% de velocidade e ataque por 6 s |
| `burn_dot` | `burn` | `inferno_burn` | 30 de dano de fogo por segundo por 3 s |
| `shock_dot` | `shock` | `electro_shock` | 25 de dano elétrico por segundo por 2 s |

## 5. Mapeamento mecânica → estrutura

| Mecânica | Cartas | Como está modelada |
| --- | --- | --- |
| Corpo a corpo | `knight`, `giant`, `mini_pekka`, `valkyrie`, `prince`, `bandit`, `royal_ghost`, `fisherman`, `mega_knight`, `electro_giant`, `goblin_giant`, `battle_ram`, `guards`, `royal_recruits`, `wall_breakers`, `battle_healer`, `skeletons` | `attack.range` ≈ 0,7–1,6 e `projectile: null` |
| À distância | `archers`, `musketeer`, `witch`, `baby_dragon`, `balloon`, `phoenix`, `inferno_dragon`, `sparky`, `ice_wizard`, `electro_wizard`, `cannon_cart` | `attack.projectile` + `range` de 3,5–6 |
| Aéreo | `baby_dragon`, `balloon`, `lava_hound`, `phoenix`, `inferno_dragon` | `flying: true`; alvos e colisões seguem `targets` |
| Só construções/torres | `giant`, `hog_rider`, `balloon`, `electro_giant`, `goblin_giant`, `lava_hound`, `battle_ram`, `wall_breakers` | `targets: ["buildings", "towers"]` |
| Escudo | `guards`, `royal_recruits` | skill `shield_200` (`kind: shield`) |
| Carga | `prince`, `battle_ram` | skill `prince_charge` (`kind: charge`) |
| Dash | `bandit` | skill `bandit_dash` (`kind: dash`) |
| Salto | `hog_rider`, `mega_knight` | skills `hog_jump` e `mega_knight_jump` (`kind: jump`) |
| Invisibilidade | `royal_ghost` | skill `royal_ghost_invisibility` |
| Gancho | `fisherman` | skill `fisherman_hook` (`kind: hook`) |
| Invocação periódica | `witch`, `goblin_hut`, `tombstone` | skill `spawn_periodic` |
| Invocação na morte | `goblin_giant`, `lava_hound`, `battle_ram`, `tombstone` | `death.spawn` |
| Renascimento | `phoenix` | skill `phoenix_rebirth` (`kind: rebirth`) |
| Ramp de dano | `inferno_dragon`, `inferno_tower` | skill `inferno_ramp` (`kind: ramp`) |
| Reflexo de dano | `electro_giant` | skill `electro_giant_reflect` (`kind: reflect`) |
| Aura de cura | `battle_healer` | skill `battle_healer_aura` (`kind: aura`, `heal_per_sec`) |
| Cura ao acertar | `battle_healer` | skill `battle_healer_heal` (`kind: heal_on_hit`) |
| Raiva | `rage` | feitiço aplica `rage_6s` (`kind: rage`) |
| Autodestruição | `wall_breakers` | skill `wall_breakers_self_destruct` |
| Transformação na morte | `cannon_cart` | skill `cannon_cart_transform` |
| Disparo carregado | `sparky` | skill `sparky_charged_shot` (`kind: charged_shot`) |
| Dano de queda | `mega_knight`, `balloon`, `lava_hound` | `deploy_damage` e `death.damage` |
| Splash | `valkyrie`, `baby_dragon`, `mega_knight`, `sparky`, `ice_wizard`, `bomb_tower`, `mortar` | `attack.splash_radius` |
| Corrente elétrica | `electro_wizard` | `attack.chain: 2` + `on_hit_effects` com `stun_0_5s` |
| Lentidão | `ice_wizard`, `earthquake` | `slow_hit_30` e `earthquake_slow` |
| Atordoamento | `zap`, `lightning`, `electro_wizard`, `mega_knight` | `stun_0_5s` |
| Congelamento | `freeze` | `freeze_4s` |
| Empurrão | `fireball`, `rocket`, `log`, `tornado` | `knockback_1`, `rolling.knockback` e `pull` |
| Veneno / dano contínuo | `poison` | `zone` com `poison_dot` |
| Terremoto | `earthquake` | `zone` com `earthquake_slow` e dano a construções |
| Geração de elixir | `elixir_collector` | skill `elixir_collector_generate` |
| Enxame | `skeletons`, `guards`, `royal_recruits`, `wall_breakers`, `graveyard` | `count > 1` + `spawn`; formação automática em círculo |
| Posicionamento livre | feitiços | qualquer posição da arena é válida para cartas do tipo feitiço; tropas/construções usam a zona liberada |

## 6. Feitiços: campos específicos

| Feitiço | Raio | Atraso | Dano | Multiplicador em torres | Campo especial |
| --- | ---: | ---: | ---: | ---: | --- |
| `fireball` | 2,5 | 500 ms | 700 | 0,35 | `effects: ["knockback_1"]` |
| `arrows` | 4,0 | 800 ms | 220 | 0,35 | — |
| `zap` | 2,5 | 0 | 160 | 0,35 | `effects: ["stun_0_5s"]` |
| `lightning` | 3,0 | 500 ms | 900 | 0,35 | `effects: ["stun_0_5s"]` |
| `rocket` | 2,0 | 700 ms | 1400 | 0,35 | `effects: ["knockback_1"]` |
| `poison` | 3,0 | 500 ms | 0 | 0,35 | `zone`: 90 dps por 8 s |
| `freeze` | 3,0 | 0 | 0 | 0,35 | `effects: ["freeze_4s"]` |
| `tornado` | 3,5 | 0 | 120 | 0,35 | `pull`: força 2,5 por 1 s |
| `log` | 1,2 | 0 | 240 | 0,35 | `rolling`: 9 tiles, largura 1,2, empurrão 1,2 |
| `graveyard` | 3,0 | 0 | 0 | 0,35 | `spawn`: 10 esqueletos, 1 a cada 400 ms |
| `rage` | 3,0 | 0 | 0 | 0,35 | `effects: ["rage_6s"]` |
| `earthquake` | 3,5 | 0 | 250 | 2,5 | `zone`: 120 dps por 3 s + `earthquake_slow` |

## 7. Adaptações de mecânica

1. **Lightning em 3 alvos.** O contrato de `spells.json` tem apenas um raio, sem contagem de alvos.
   O Raio aplica 900 de dano e `stun_0_5s` a todos os alvos válidos dentro de raio 3,0.
2. **Salto do Cavaleiro de Javali.** O salto usa `kind: jump` com `damage: 0` e `radius: 0`,
   disparado quando o alvo está a até 3 tiles. Ele representa o salto curto/travessia do rio;
   caso o pathfinding da engine não modele o rio, o salto continua válido como avanço curto.
3. **Mega Cavaleiro.** Usa dois skills: `mega_knight_jump` (salto com dano) e
   `mega_knight_deploy` (dano de queda + atordoamento).
4. **Gigante Elétrico.** O zapping de atacantes próximos é aproximado por
   `electro_giant_reflect` (35% do dano refletido em raio 3,5). Não há skill de aura elétrica.
5. **Curandeira de Batalha.** A aura usa `heal_per_sec: 60` e o ataque usa `heal_on_hit: 80`;
   não existe aura de raiva em unidade. Raiva é representada pelo feitiço `rage` (`rage_6s`).
6. **Fênix.** O renascimento cria a unidade auxiliar `phoenix_chick` (900 HP) após 1200 ms.
   A unidade renascida não possui o skill de renascimento, evitando recursão infinita.
7. **Carro-Canão.** Ao morrer, `transform_on_death` cria a construção `cannon` com 450 HP,
   reutilizando a definição existente.
8. **Quebra-Muros.** Cada unidade carrega `self_destruct` com 700 de dano em raio 1,8.
9. **Recrutas Reais.** `deploy_anywhere` permite o posicionamento fora da zona normal; o
   servidor continua validando elixir, tempo de invocação e limites da arena.
10. **Bruxa.** Invoca 3 esqueletos a cada 7 s, com no máximo 9 vivos, usando `spawn_periodic`.
11. **Cemitério.** Invoca 10 esqueletos em 4 s (intervalo de 400 ms) dentro de raio 3,0.
12. **Lápide.** Invoca 1 esqueleto a cada 3 s (máximo 3 vivos) e libera 4 esqueletos ao morrer.
13. **Cabana Goblin.** Invoca 1 goblin lanceiro a cada 4,5 s, com no máximo 3 vivos.
14. **Coletor de Elixir.** Gera 1 elixir a cada 8,4 s durante 70 s, totalizando cerca de 8 elixirs.
15. **Inferno.** Dragão e Torre Infernal compartilham `inferno_ramp`: até ×8 de dano, degrau de
    400 ms e reset de 1 s sem alvo.
16. **Faísca.** O disparo carregado leva 3,5 s, causa 1100 de dano em raio 2,2 e usa o projétil
    `sparky_blast`. O ataque básico de 500 de dano existe como fallback caso a engine não
    dispare o skill; a prioridade esperada é o disparo carregado.
17. **Pescador.** O gancho tem alcance 6,0, puxa 3,5 tiles, causa 150 de dano e recarrega em 6 s.
18. **Fantasma Real.** O ciclo de invisibilidade é `period_ms: 5000` e `visible_ms: 1200`.
19. **Congelamento.** Dura 4 s (`freeze_4s`), zerando velocidade e ataque.
20. **Veneno.** Zona de 8 s com 90 de dano por segundo; `slow_mult: 1.0` (não reduz velocidade).
21. **Terremoto.** 250 de dano imediato, zona de 120 dps por 3 s, lentidão de 30% e
    `tower_damage_mult: 2.5` para reproduzir a força contra construções.
22. **Bola de Fogo e Foguete.** Aplicam `knockback_1` (1 tile) em todos os alvos válidos.
23. **Tronco.** Rolagem de 9 tiles com largura 1,2, 240 de dano e `rolling.knockback: 1.2`.
24. **Tornado.** Puxa com força 2,5 por 1 s, causa 120 de dano e não afeta torres com puxão.
25. **Arqueiras, Esqueletos, Guardas, Recrutas e Quebra-Muros.** Usam `count` por carta e
    estatísticas por unidade; a engine distribui a formação automaticamente em círculo.
26. **Sem Snowball / Giant Snowball.** O efeito reutilizável `knockback_1` já está disponível
    para cartas futuras que precisem de empurrão radial.
27. **Alvos de buff em feitiços.** O contrato de `spells.json` não possui um campo de time.
    Feitiços de dano aplicam efeitos a inimigos; `rage` deve aplicar `rage_6s` somente a
    aliados, pois `rage`, `heal`, `speed_boost` e `attack_boost` são efeitos de buff. A
    validação garante que o preset existe; a seleção de aliados é responsabilidade da engine.
28. **Construções sem ataque.** `goblin_hut`, `tombstone` e `elixir_collector` usam
    `targets: []`, `sight_range: 0` e ataque zerado; toda a função vem de `spawn_periodic`
    ou `elixir_generate`. Isso evita que ataquem sem solicitar uma mecânica extra.
29. **Atualização de contrato: `knockback`.** O kind `knockback` e o preset `knockback_1`
    (`params.amount = 1.0`) foram adicionados após a atualização de `docs/DEFINICOES.md`;
    `fireball` e `rocket` os utilizam. O tronco continua usando `rolling.knockback` e o
    tornado usa `pull`, que são campos próprios do feitiço.

## 8. Validação

```bash
python3 shared/validate.py
```

O script verifica:

- leitura de todos os JSON;
- exatamente 50 cartas e slugs únicos em `snake_case`;
- referências `cards → troops/buildings/spells`;
- referências `troops/buildings → projectiles/effects/skills`, incluindo `death.spawn`,
  `death.effects` e parâmetros de skill (`troop`, `building`, `projectile`, `effects`);
- referências `spells → effects`, `zone.effect` e `spawn.troop`;
- existência de todos os presets de skill e efeito usados;
- `kind`s suportados em skills e effects e presença dos parâmetros obrigatórios;
- torres obrigatórias, arena 18x32, rio e duas pontes;
- presença das 8 cartas do deck padrão;
- traduções completas e não vazias para as 50 cartas nos dois idiomas.

Saída esperada: `OK: validação concluída sem erros.`
