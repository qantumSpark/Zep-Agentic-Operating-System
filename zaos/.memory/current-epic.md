# Epic active : _aucun_

> Milestone : 13 — Product Contract Layer
> Statut : IDLE (entre epics)

## Objectif

Structs Rust pour les 5 artefacts produit, parsers markdown tolerants, methodes MemoryReader, commande Tauri get_product_contract agregee. Types dans models.rs, parsing dans reader.rs.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Structs & models | `memory/models.rs`, `memory/mod.rs` | DONE | 10 structs + cargo check OK |
| 2 | Parsers brief + goals | `memory/reader.rs` | DONE | parse_product_brief_md, parse_experience_goals_md + read |
| 3 | Parsers checks + readiness | `memory/reader.rs` | DONE | parse_acceptance_checks_md, parse_release_readiness_md + read |
| 4 | Parser insights + agrege | `memory/reader.rs` | DONE | parse_session_insights_md + read_product_contract tokio::join! |
| 5 | Commande Tauri | `commands.rs`, `main.rs` | DONE | get_product_contract + invoke_handler |
| 6 | Build & test | — | DONE | 72 tests OK, build 0 warnings |
