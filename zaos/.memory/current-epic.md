# Epic active : Phase 7 — Project Portability & Workflow Init

> Milestone : 7 — App autonome et portable
> Date de debut : 2026-03-31
> Statut : EN COURS

## Objectif

Rendre ZAOS utilisable sur n'importe quel projet : embarquer les agents/rules dans le binaire, permettre de choisir et changer de projet depuis l'UI, et demarrer un workflow depuis le dashboard.

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| A1 | Module embedded.rs (include_str!) | `deployer/embedded.rs` | DONE | 6 agents + 4 rules |
| A2 | Rewrite sync sans source_dir | `deployer/sync.rs` | DONE | Itere sur embedded:: |
| A3 | Supprimer find_reference_dir | `deployer/mod.rs` | DONE | + add mod embedded |
| A4 | Supprimer bundle.resources | `tauri.conf.json` | DONE | Plus de reference/ runtime |
| B1 | project_dir → Arc<RwLock<PathBuf>> | `commands.rs` | DONE | AppState refactor |
| B2 | Update tous les commands | `commands.rs` | DONE | .read().await |
| B3 | Commande switch_project | `commands.rs` | DONE | Kill+swap+re-init |
| B4 | Commande get_project_info | `commands.rs` | DONE | Path + name |
| B5 | Update main.rs setup | `main.rs` | DONE | Plugins + commands |
| B6 | Watcher stop/restart | `watchers/service.rs` | DONE | CancellationToken |
| C1 | tauri-plugin-dialog | `Cargo.toml`, `package.json` | DONE | Dep + permission |
| C2 | projectStore.ts | `stores/projectStore.ts` | DONE | State projet |
| C3 | ProjectPicker + StatusBar | `ProjectPicker.tsx`, `StatusBar.tsx` | DONE | Folder picker |
| C4 | Listener project-changed | `useTauriEvents.ts` | DONE | Reset stores + reload all |
| C5 | Charger project info startup | `App.tsx` | DONE | 6 invokes en parallele |
| D1 | Commande start_epic | `commands.rs`, `engine.rs` | DONE | Atomique via engine.start_epic() |
| D2 | UI Start Epic | `WorkflowSection.tsx` | DONE | Form quand idle |
| D3 | CTA StartupDashboard | `StartupDashboard.tsx` | DONE | Guide utilisateur |

## Vagues

1. A1 + A2 + A3 + A4 (embedded content)
2. B1 + B2 + B3 + B4 + B5 + B6 (dynamic project dir)
3. C1 + C2 + C3 + C4 + C5 (project picker UI)
4. D1 + D2 + D3 (workflow init)
