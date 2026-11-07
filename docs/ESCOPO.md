# Escopo — Royale Game

Jogo inspirado em Clash Royale, com arquitetura cliente-servidor e backend autoritativo.

Status: escopo fechado em 2026-11-07.

---

## 1. Stack e ambiente

| Camada | Tecnologia |
| --- | --- |
| Backend | Rust (edition 2024), Cargo workspace com múltiplos crates |
| Servidor HTTP/WS | axum + tokio |
| Banco | PostgreSQL 16 em container Docker (`compose.yaml` na raiz) |
| Acesso ao banco | `deadpool-postgres` + `tokio-postgres`; todo SQL isolado no crate `db` |
| Migrações | Arquivos numerados em `crates/db/migrations/`, executados na inicialização em ordem |
| Frontend | SvelteKit 3 + Svelte 5 + TypeScript + Vite 8 |
| Render | Canvas2D puro, engine própria (sem PixiJS/Three.js) |
| Áudio | WebAudio 100% procedural (sem arquivos de áudio) |
| Desenvolvimento | `docker compose up` (Postgres) + `cargo run` + `npm run dev` |
| Versionamento | Git inicializado na raiz, commits por área concluída |

Sem etapa de deploy/produção neste escopo.

---

## 2. Fluxo do jogador

1. **Carregamento** — busca definições no servidor, verifica sessão, pré-aquece áudio (após gesto), faz bake dos sprites e mostra progresso.
2. **Login/Registro** — nome, e-mail, senha e confirmação. Sem verificação de e-mail.
3. **Menu principal** — abas Cartas, Batalha (inicial) e Chat global; perfil/estatísticas e replays acessíveis no menu.
4. **Batalha** — botão Batalhar → fila de matchmaking real (somente PvP humano) → partida.
5. **Fim de partida** — tela de resultado com vitória/derrota/empate e opção de revanche (volta à fila).

Sessão persistente por cookie `HttpOnly` `SameSite=Lax` (Secure em produção), token opaco de 32 bytes armazenado com hash no banco, validade deslizante de 30 dias. O jogador não refaz login a cada acesso.

---

## 3. Interface e responsividade

- Canvas do jogo sempre em **9:16**, com letterbox (preenchimento) em telas de outras proporções; funciona em desktop, tablet e mobile.
- Controles por mouse e toque (seleção de carta, posicionamento, cancelamento).
- Layout do menu responsivo, com abas fixas e áreas roláveis.
- Textos de interface em **pt-BR e inglês** (i18n no frontend), com seletor de idioma.
- Nomes e descrições das cartas ficam **somente no frontend**, indexados pelo slug (os JSON do backend são neutros).

---

## 4. Regras de batalha

- Arena clássica **18x32** (grade), rio no centro, **duas pontes**, por lado: 2 torres princesa + 1 torre rei.
- Elixir máximo 10, regeneração padrão (~2,8 s por ponto), **em dobro na prorrogação**.
- Tempo normal de **3 min**; se houver empate, **prorrogação de 2 min** com morte súbita (primeira torre destruída vence).
- Sem torre destruída na prorrogação: vence quem causou mais dano às torres; dano idêntico = empate.
- Deck com **exatamente 8 cartas, sem duplicatas**, validado no servidor. Contas novas recebem um deck padrão.
- Todas as **50 cartas desbloqueadas desde o início**; sem níveis, moedas, baús ou upgrades.
- Posicionamento livre dentro do território permitido (lado do jogador e avanço após destruir torre princesa), com validação autoritativa de elixir, tempo de invocação e zona.
- Sem bots: partidas apenas entre dois humanos conectados. Para testes, a suíte headless controla os dois lados.
- **Desconexão**: a partida continua normalmente; o jogador pode reconectar e receber o estado atual. Não há derrota automática nem pausa. Sala é encerrada ao fim da partida, com janela de reconexão.

---

## 5. Arquitetura autoritativa e rede

- Servidor simula todos os ticks a **30 Hz** com passo fixo e determinístico.
- Cliente envia apenas **intenções** (entrar/sair da fila, posicionar carta, chat, ping); nunca calcula resultado.
- Servidor valida e envia **snapshot completo da arena a cada tick (30 Hz)**, serializado em **binário** (bincode) por WebSocket.
- Cliente renderiza a 60 FPS interpolando entre os dois últimos snapshots; nenhuma predição autoritativa.
- REST para auth, conteúdo, deck, perfil, estatísticas e replays. WebSocket autenticado pelo cookie de sessão.
- Física 2D (posições, colisões, projéteis em plano); a profundidade é apenas visual.

### 5.1. Perspectiva

- O servidor mantém coordenadas e lados absolutos.
- No cliente, o jogador local sempre aparece embaixo e em azul; o adversário em cima e em vermelho.
- O espelhamento é uma **rotação de 180° da renderização** (x e y), aplicada apenas no frontend, sem alterar o estado autoritativo.

---

## 6. Gráficos, animação e áudio

- **2.5D**: arena com projeção inclinada, ordenação por profundidade, sombras e unidades desenhadas como billboards com altura.
- Todos os assets gerados em runtime:
  - **Animação esquelética + IK** (ossos, poses, membros com IK para passos/ataques).
  - **Sprite baking em OffscreenCanvas** por animação/ângulo, em tempo de carregamento, com barra de progresso.
  - **Vetores desenhados em runtime** para arena, efeitos, projéteis, torres, UI e cartas.
- Direção de arte: **cartoon flat com contornos grossos**, cores saturadas, sombras simples.
- Áudio procedural com WebAudio: efeitos (invocação, impacto, feitiço, torre, vitória/derrota) e música de fundo sintetizada, com controle de volume e mute persistido.

---

## 7. Conteúdo de cartas

- **50 cartas distintas** de Clash Royale, priorizando diversidade mecânica.
- Inclui **tropas, feitiços e construções**; **sem campeões e sem evoluções**.
- Todas as 50 são **100% jogáveis** no servidor.
- Mecânicas cobertas de forma genérica e orientada a dados (sem código por carta): alvos terrestres/aéreos, corpo a corpo/à distância, splash, perfuração, carga, dash, salto, escudo, invocação ao morrer, spawners, aura, cura, congelamento, atordoamento, veneno, fogo, eletrização, lentidão, raiva, empurrão, invisibilidade, redirecionamento de alvo, entre outras identificadas pelo subagente.

### 7.1. Arquivos JSON compartilhados (`shared/`)

- `cards.json` — slug, custo de elixir, raridade, tipo (tropa/feitiço/construção), tempo de invocação, quantidade, alvo, arte e referências.
- `troops.json` — vida, velocidade, dano, alcance, cadência, alvo, massa, raio, `skills[]`.
- `spells.json` — custo, raio, duração, dano, efeitos, atraso, padrão de aplicação.
- `buildings.json` — vida, duração, cadência, alcance, invocação periódica.
- `towers.json` — torres princesa e rei, vida, dano, alcance, ativação.
- `projectiles.json` — velocidade, raio, visual, perfuração, perseguição.
- `effects.json` — congelamento, atordoamento, veneno, fogo, eletrização, lentidão, raiva, cura, escudo, etc., com atributos próprios.
- `skills.json` — habilidades reutilizáveis e parametrizáveis.
- `arenas.json` — dimensões, rio, pontes, zonas de posicionamento, torres.
- `rarities.json` e arquivos auxiliares conforme necessidade.

Contrato: as definições são carregadas pelo crate `defs`, embutidas no binário e servidas ao frontend por endpoint de conteúdo. Novas cartas devem exigir apenas novos JSON (mais traduções no frontend).

### 7.2. Subagente especializado

O subagente de cartas e estruturas deve:

1. Selecionar as 50 cartas (tropas, feitiços, construções; sem campeões/evoluções).
2. Analisar atributos, comportamento, alvos, movimento, habilidades, efeitos causados/recebidos e interações.
3. Criar/preencher todos os JSON compartilhados.
4. Criar/ampliar as estruturas de dados reutilizáveis do core para cobrir cada mecânica, evitando lógica específica por carta.
5. Documentar o mapeamento carta → estruturas e validar que todas as 50 são carregáveis e jogáveis.

---

## 8. Estado do servidor

### 8.1. Estado global

```text
ServerState {
  users[]
  rooms[]
  ...
}
```

### 8.2. Sala (Room)

```text
Room {
  troops[]
  players[]
  time
  ...
}
```

### 8.3. Tropa

```text
Troop {
  card_slug
  troop_slug
  health
  pos_x
  pos_y
  effects[]
  ...
}
```

Cada efeito tem estrutura própria tipada (ex.: congelamento com duração e fator; veneno com dano por tick e duração; fogo com dano contínuo; eletrização com propagação; atordoamento com duração).

---

## 9. Autenticação e segurança

- Senhas com Argon2id; e-mail e nome únicos (e-mail case-insensitive).
- Token de sessão opaco, hash no banco, cookie HttpOnly/SameSite=Lax.
- Toda entrada validada no servidor (deck, cards, coordenadas, chat).
- **Rate limit em toda a aplicação**: por IP em REST e por usuário/IP no WS, com limites mais rígidos em auth (registro/login), chat e comandos de batalha.
- Chat global em tempo real, sem histórico, mensagens de até 200 caracteres, com rate limit por usuário.
- Sem moderação automática de conteúdo.

---

## 10. Perfil, estatísticas e replays

- Perfil com nome, e-mail, data de criação, idioma e estatísticas: partidas, vitórias, derrotas, empates, torres destruídas.
- Histórico das partidas recentes (oponente, resultado, duração).
- **Replays**: log de snapshots comprimido (gzip/bincode) em `bytea` no Postgres, mantendo as **10 últimas partidas de cada jogador**. Player de replay no frontend com pausa, velocidade e barra de progresso, na perspectiva do dono.

---

## 11. Estrutura do repositório

```text
/
├── compose.yaml                 # PostgreSQL
├── docs/
│   └── ESCOPO.md
├── shared/                      # JSONs de definição
├── backend/                     # Cargo workspace
│   ├── Cargo.toml
│   └── crates/
│       ├── core/                # simulação autoritativa, sistemas, entidades
│       ├── defs/                # carga e validação dos JSON
│       ├── shared/              # protocolo binário, tipos comuns, constantes
│       ├── db/                  # pool, models/, queries/, migrations/
│       ├── auth/                # senhas, sessões, middleware
│       ├── net/                 # rotas REST, WebSocket, rate limit, codec
│       └── server/              # binário: config, tick loop, rooms, filas, chat
└── svelte/
    └── src/
        ├── lib/
        │   ├── game/            # engine de render, skeletal+IK, baking, câmera
        │   ├── net/             # REST + WS
        │   ├── audio/           # síntese procedural
        │   ├── i18n/            # pt-BR e en (inclui nomes/descrições)
        │   ├── stores/          # sessão, deck, conteúdo
        │   └── ui/              # componentes por tela
        └── routes/              # loading, login, registro, menu, batalha, replay
```

Regras: arquivos pequenos, um módulo por responsabilidade, sem SQL fora do crate `db`, sem lógica de jogo no frontend e sem dependência de assets externos.

---

## 12. Qualidade e testes

- Testes unitários do core: elixir, dano, alvos, pathfinding, efeitos, torres, vitória.
- Testes de integração: registro/login/sessão, deck, rate limit, handshake WS.
- Teste headless de partida completa com dois jogadores simulados, cobrindo invocação, combate e fim de jogo.
- `cargo test` no workspace e `npm run check` no frontend devem passar.
- Balanceamento: validação por simulação headless (sem prometer equilíbrio competitivo fino).

---

## 13. Fora de escopo

- Progressão, níveis de carta, baús, moedas, loja e passe.
- Campeões, evoluções, torres como cartas e modos especiais (2v2, desafios, guerras).
- Bots no matchmaking de produção.
- Verificação de e-mail, recuperação de senha, OAuth.
- Espectador, tutorial e PWA.
- Deploy em nuvem, HTTPS, CI/CD.

---

## 14. Critérios de aceite

1. Fluxo carregamento → login/registro → menu → batalha funciona em desktop e mobile, com canvas 9:16 letterboxed.
2. Sessão persiste entre recarregamentos via cookie HttpOnly.
3. Deck de 8 cartas montado, salvo e validado no servidor; grade de 4 por linha com detalhes das cartas.
4. Dois navegadores entram na fila, são pareados e jogam uma partida completa com torres, elixir e prorrogação.
5. Servidor simula e envia snapshots a 30 Hz; cliente interpola e nunca decide o resultado.
6. Cada jogador vê a si mesmo embaixo e em azul, com a arena espelhada corretamente.
7. As 50 cartas são carregadas dos JSON e jogáveis com suas mecânicas.
8. Visual 2.5D com skeletal+IK, sprite baking em OffscreenCanvas e vetores em runtime; áudio procedural.
9. Chat global em tempo real com rate limit.
10. Perfil, estatísticas e replay das 10 últimas partidas.
11. Migrações rodam na ordem na inicialização e o compose sobe o Postgres.
12. Testes automatizados passando.
