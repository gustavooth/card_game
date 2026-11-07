# Royale Arena

Jogo de cartas e batalhas 1v1 inspirado em Clash Royale, com backend autoritativo em Rust,
frontend SvelteKit e renderização 2.5D procedural em Canvas2D.

O escopo fechado está em [`docs/ESCOPO.md`](docs/ESCOPO.md); o contrato dos dados está em
[`docs/DEFINICOES.md`](docs/DEFINICOES.md) e o catálogo das 50 cartas em
[`docs/CARTAS.md`](docs/CARTAS.md).

## Requisitos

- Rust 1.90+ (edition 2024)
- Node.js 22+
- Docker + Docker Compose

## Como rodar

```bash
# 1. PostgreSQL
docker compose up -d

# 2. Backend (porta 8090)
cd backend
cargo run -p royale-server

# 3. Frontend (porta 5173)
cd svelte
npm install
npm run dev
```

Abra `http://localhost:5173`. A interface é **mobile-first** (shell retrato centralizado no desktop,
barra de recursos, navegação inferior e HUD de cartas na batalha). Crie duas contas (ou use duas
janelas anônimas) para o PvP real: o matchmaking não usa bots.

Variáveis do backend (opcionais, os padrões funcionam com o compose):

```bash
DATABASE_URL=postgres://royale:royale@localhost:5433/royale
BIND_ADDR=0.0.0.0:8090
COOKIE_SECURE=false
SESSION_DAYS=30
ALLOWED_ORIGINS=http://localhost:5173,http://127.0.0.1:5173
```

O Vite faz proxy de `/api` e `/ws` para `127.0.0.1:8090`, então cookies de sessão
`HttpOnly` funcionam sem CORS no desenvolvimento.

## Testes

```bash
cd backend
cargo test --workspace       # core, auth, banco e partida headless
cargo check --workspace

cd svelte
npm run check                # svelte-check + TypeScript
npm run build
```

O teste `full_match_reaches_combat_and_ends` executa uma partida completa na engine,
com deploy das duas mãos, combate, torres e fim de jogo.

## Arquitetura

```text
shared/*.json                  definições compartilhadas (50 cartas)
backend/crates/shared          protocolo binário, snapshots e constantes
backend/crates/defs            carga e validação dos JSON
backend/crates/core            engine autoritativa: tick 30 Hz, física 2D, efeitos, feitiços
backend/crates/db              deadpool-postgres, models/, queries/, migrations/
backend/crates/auth            Argon2id, tokens opacos e sessões
backend/crates/net             axum REST + WebSocket, rate limit, chat
backend/crates/server          main, tick loop, salas, matchmaking, replays
svelte/src/lib/game            render 2.5D, skeletal+IK, OffscreenCanvas, HUD
svelte/src/lib/audio           síntese procedural WebAudio
svelte/src/lib/stores          sessão, conteúdo, batalha e chat
```

- O servidor simula a **30 Hz** e envia um snapshot completo em MessagePack a cada tick.
- O cliente interpola snapshots a 60 FPS e só envia intenções (deploy, fila, chat).
- A perspectiva é espelhada em 180° no cliente: o jogador local fica sempre embaixo e azul.
- A arena é 18×32, com rio, duas pontes, duas torres princesa e uma torre rei por lado.
- Regras: 3 min, prorrogação de 2 min com elixir em dobro e morte súbita.

## Replays

As 10 últimas partidas de cada jogador são gravadas no PostgreSQL como frames de snapshot
comprimidos com gzip (`replays.data`). O replay abre em `/replay/:id` com play/pause,
velocidade e barra de progresso, na perspectiva do dono.

## Controles

- Arraste uma carta para a arena ou toque na carta e depois no campo.
- Teclas `1`–`4` selecionam cartas; `Esc` cancela a seleção.
- O posicionamento é validado no servidor (elixir, zona liberada, sobreposição e rio).
