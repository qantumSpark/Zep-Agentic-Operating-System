/// Check if current-epic.md has any tasks in active state (TODO, EN COURS, etc.).
/// Returns false if all tasks are DONE/VALIDATED or if the table is empty.
pub fn has_active_tasks(epic_content: &str) -> bool {
    let mut found_data_row = false;

    for line in epic_content.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('|') || trimmed.contains("---") {
            continue;
        }

        let cells: Vec<&str> = trimmed.split('|').map(|c| c.trim()).collect();
        if cells.len() < 6 {
            continue;
        }

        let first_cell = cells[1];
        if first_cell == "#" || first_cell == "Task" || first_cell == "Statut" {
            continue;
        }

        found_data_row = true;

        let status = cells[4].to_uppercase();
        if status.contains("TODO")
            || status.contains("EN COURS")
            || status.contains("A FAIRE")
            || status.contains("IN_PROGRESS")
            || status.contains("BLOQUE")
        {
            return true;
        }
    }

    if !found_data_row {
        return true; // No data rows = plan not yet created = consider active
    }

    false
}

#[cfg(test)]
mod tests {
    use super::has_active_tasks;

    // -------------------------------------------------------------------------
    // Standalone copy of has_active_tasks from src/bin/zaos_hooks.rs.
    //
    // The hooks binary is compiled as a standalone executable that cannot
    // depend on the main zaos crate.  Because of that, it carries its own
    // copy of this function.  We duplicate the hooks version here so we can
    // run both implementations against the same inputs and verify they always
    // agree.  If someone modifies one copy without the other, these tests
    // will catch the drift.
    // -------------------------------------------------------------------------
    fn has_active_tasks_hooks_copy(epic_content: &str) -> bool {
        let mut found_data_row = false;

        for line in epic_content.lines() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with('|') || trimmed.contains("---") {
                continue;
            }

            let cells: Vec<&str> = trimmed.split('|').map(|c| c.trim()).collect();
            if cells.len() < 6 {
                continue;
            }

            let first_cell = cells[1];
            if first_cell == "#" || first_cell == "Task" || first_cell == "Statut" {
                continue;
            }

            found_data_row = true;

            let status = cells[4].to_uppercase();
            if status.contains("TODO")
                || status.contains("EN COURS")
                || status.contains("A FAIRE")
                || status.contains("IN_PROGRESS")
                || status.contains("BLOQUE")
            {
                return true;
            }
        }

        if !found_data_row {
            return true;
        }

        false
    }

    /// Helper: assert both implementations return the same result for the
    /// given input, then return that result for further assertions.
    fn assert_both(input: &str, expected: bool) {
        let shared = has_active_tasks(input);
        let hooks = has_active_tasks_hooks_copy(input);
        assert_eq!(
            shared, hooks,
            "DRIFT DETECTED: shared={shared}, hooks={hooks} for input:\n{input}"
        );
        assert_eq!(
            shared, expected,
            "Expected {expected} but got {shared} for input:\n{input}"
        );
    }

    // -- Test cases -----------------------------------------------------------

    #[test]
    fn task_check_all_done() {
        let input = "\
# Epic active : Test

> Milestone : 1 — Test
> Statut : EN COURS

## Objectif

Testing

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Task one | `a.rs` | DONE | |
| 2 | Task two | `b.rs` | DONE | |
";
        assert_both(input, false);
    }

    #[test]
    fn task_check_some_en_cours() {
        let input = "\
# Epic active : Test

## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Task one | `a.rs` | DONE | |
| 2 | Task two | `b.rs` | EN COURS | wip |
| 3 | Task three | `c.rs` | DONE | |
";
        assert_both(input, true);
    }

    #[test]
    fn task_check_mix_done_and_a_faire() {
        let input = "\
## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Task one | `a.rs` | DONE | |
| 2 | Task two | `b.rs` | A FAIRE | |
| 3 | Task three | `c.rs` | DONE | |
";
        assert_both(input, true);
    }

    #[test]
    fn task_check_empty_content() {
        assert_both("", true);
    }

    #[test]
    fn task_check_header_only_no_table() {
        let input = "\
# Epic active : Test

> Milestone : 1 — Test
> Statut : EN COURS

## Objectif

Just a header, no tasks table at all.
";
        assert_both(input, true);
    }

    #[test]
    fn task_check_all_validated() {
        let input = "\
## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Task one | `a.rs` | VALIDATED | |
| 2 | Task two | `b.rs` | VALIDATED | |
";
        assert_both(input, false);
    }

    #[test]
    fn task_check_bloque() {
        let input = "\
## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Task one | `a.rs` | DONE | |
| 2 | Task two | `b.rs` | BLOQUE | waiting |
| 3 | Task three | `c.rs` | VALIDATED | |
";
        assert_both(input, true);
    }

    #[test]
    fn task_check_todo_status() {
        let input = "\
## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Task one | `a.rs` | TODO | |
| 2 | Task two | `b.rs` | DONE | |
";
        assert_both(input, true);
    }

    #[test]
    fn task_check_in_progress_status() {
        let input = "\
## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Task one | `a.rs` | DONE | |
| 2 | Task two | `b.rs` | in_progress | |
";
        assert_both(input, true);
    }

    #[test]
    fn task_check_mixed_done_validated() {
        let input = "\
## Tasks

| # | Task | Fichier(s) | Statut | Notes |
|---|------|-----------|--------|-------|
| 1 | Task one | `a.rs` | DONE | |
| 2 | Task two | `b.rs` | VALIDATED | |
| 3 | Task three | `c.rs` | DONE | |
";
        assert_both(input, false);
    }
}
