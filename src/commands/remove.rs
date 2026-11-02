use colored::Colorize;

use crate::commands::worktree_picker::pick_worktree;
use crate::git::{discover_repo, list_worktrees, remove_worktree};
use crate::models::Worktree;
use crate::utils::{confirm, handle_command_error};

pub fn run(name: Option<&str>, force: bool, yes: bool) {
    let repo = discover_repo().unwrap_or_else(|e| handle_command_error(&e));

    let worktrees = list_worktrees(&repo).unwrap_or_else(|e| handle_command_error(&e));

    let worktree: Worktree = if let Some(name) = name {
        if name.trim().is_empty() {
            pick_worktree_to_remove(&worktrees)
        } else {
            // Not find_worktree_by_name: as in the original, remove also takes a worktree path
            // and does not guess from a branch suffix, so its rules are deliberately its own.
            match worktrees.iter().find(|wt| {
                wt.branch == name || wt.path == name || wt.path.ends_with(&format!("/{}", name))
            }) {
                Some(wt) => wt.clone(),
                None => handle_command_error(&format!(
                    "Worktree '{}' not found. Use 'grove list' to see available worktrees.",
                    name
                )),
            }
        }
    } else {
        pick_worktree_to_remove(&worktrees)
    };

    if worktree.is_main {
        handle_command_error(&format!(
            "Cannot remove the main worktree ({}). This is the primary worktree.",
            worktree.branch
        ));
    }

    if worktree.is_locked {
        handle_command_error(&format!(
            "Worktree '{}' is locked. Unlock it first with 'git worktree unlock'.",
            worktree.branch
        ));
    }

    // Block removal of dirty worktrees without --force
    if worktree.is_dirty && !force {
        println!(
            "{}",
            "Warning: This worktree has uncommitted changes.".yellow()
        );
        println!(
            "{}",
            "Use --force to remove it anyway, or commit/stash your changes first.".yellow()
        );
        std::process::exit(1);
    }

    // Confirm removal
    if !yes {
        let msg = if worktree.is_dirty {
            format!(
                "Are you sure you want to remove the worktree for '{}'? Uncommitted changes will be lost!",
                worktree.branch
            )
        } else {
            format!(
                "Are you sure you want to remove the worktree for '{}'?",
                worktree.branch
            )
        };

        if !confirm(msg) {
            println!("{}", "Operation cancelled.".blue());
            return;
        }
    }

    if let Err(e) = remove_worktree(&repo, &worktree.path, force) {
        handle_command_error(&e);
    }

    println!(
        "{} {}",
        "✓ Removed worktree:".green(),
        worktree.branch.bold()
    );
}

fn pick_worktree_to_remove(worktrees: &[Worktree]) -> Worktree {
    let removable: Vec<&Worktree> = worktrees
        .iter()
        .filter(|wt| !wt.is_main && !wt.is_locked)
        .collect();

    pick_worktree(&removable, "Select a worktree to remove (type to search)")
}
