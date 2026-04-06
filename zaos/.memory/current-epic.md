# Epic active : Bugfix B3-B4 — Policy engine bugs

> Milestone : 25 — Bugfixes backlog
> Statut : EN COURS

## Objectif

Corriger deux bugs dans policy.rs : (B3) FileDelete jamais retourne par derive_action_type, (B4) is_test_command verifie tool_name au lieu du contenu command.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Ajouter champ command_text a ActionContext | `policy.rs` | DONE | Pour B4 |
| 2 | Corriger is_test_command pour inspecter command_text | `policy.rs` | DONE | B4 fix |
| 3 | Override action_type vers FileDelete quand Bash destructif | `commands.rs` | DONE | B3 fix |
| 4 | Extraire command_text dans le forwarder | `commands.rs` | DONE | Pour B4 |
| 5 | Ajouter command_text au ActionContext debug endpoint | `commands.rs` | DONE | Pour B4 |
| 6 | Adapter les tests existants | `policy.rs` | DONE | command_text ajoute partout |
| 7 | Verification cargo test | - | DONE | 160 tests passed |
