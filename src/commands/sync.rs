use colored::Colorize;

use crate::git::{discover_repo, get_default_branch, list_worktrees, sync_branch};
use crate::utils::handle_command_error;

pub fn run(branch: Option<&str>) {
    let repo = discover_repo().unwrap_or_else(|e| handle_command_error(&e));

    let target_branch = if let Some(b) = branch {
        b.to_string()
    } else {
        get_default_branch(&repo).unwrap_or_else(|e| handle_command_error(&e))
    };

    // Check if the branch is checked out in any worktree
    let worktrees = list_worktrees(&repo).unwrap_or_else(|e| handle_command_error(&e));
    if worktrees.iter().any(|wt| wt.branch == target_branch) {
        println!(
            "{}",
            format!(
                "Branch '{}' is checked out in a worktree. Git won't sync against a checked-out branch (a limitation of git worktrees).",
                target_branch
            )
            .yellow()
        );
        println!(
            "{}",
            format!(
                "Run 'git fetch origin', then merge or rebase from 'origin/{}'.",
                target_branch
            )
            .yellow()
        );
        std::process::exit(1);
    }

    if let Err(e) = sync_branch(&repo, &target_branch) {
        handle_command_error(&e);
    }

    println!(
        "{} {} {}",
        "✓ Synced".green(),
        target_branch.bold(),
        "from origin".dimmed()
    );
}
