use colored::Colorize;
use std::process::Command;

use crate::commands::shell_init::{
    get_shell_setup_instructions, mark_shell_tip_shown, should_show_shell_tip,
};
use crate::commands::worktree_picker::pick_worktree;
use crate::git::{discover_repo, find_worktree_by_name, list_worktrees, RepoContext};
use crate::models::Worktree;
use crate::utils::{get_shell_for_platform, handle_command_error};

pub fn run(name: Option<&str>, path_only: bool) {
    if path_only
        && name.map(|n| n.trim().is_empty()).unwrap_or(true)
        && !atty::is(atty::Stream::Stdin)
    {
        handle_command_error(
            "Interactive selection requires a TTY. Provide a branch name when using --path-only.",
        );
    }

    let repo = discover_repo().unwrap_or_else(|e| handle_command_error(&e));

    let worktree = if let Some(name) = name {
        if name.trim().is_empty() {
            pick_or_error(&repo)
        } else {
            match find_worktree_by_name(&repo, name) {
                Ok(Some(wt)) => wt,
                Ok(None) => handle_command_error(&format!(
                    "Worktree '{}' not found. Use 'grove list' to see available worktrees.",
                    name
                )),
                Err(e) => handle_command_error(&e),
            }
        }
    } else {
        pick_or_error(&repo)
    };

    navigate_to_worktree(&worktree, path_only);
}

fn pick_or_error(repo: &RepoContext) -> Worktree {
    let worktrees = list_worktrees(repo).unwrap_or_else(|e| handle_command_error(&e));
    let candidates: Vec<&Worktree> = worktrees.iter().collect();
    pick_worktree(&candidates, "Select a worktree (type to search)")
}

fn navigate_to_worktree(worktree: &Worktree, path_only: bool) {
    if path_only {
        println!("{}", worktree.path);
        return;
    }

    let shell = get_shell_for_platform();

    println!(
        "{} {}",
        "✓ Entering worktree:".green(),
        worktree.branch.bold()
    );
    println!("  {} {}", "Path:".dimmed(), worktree.path);

    // Show shell integration tip on first use
    if should_show_shell_tip() {
        if let Some(instructions) = get_shell_setup_instructions() {
            println!("{}", instructions);
            mark_shell_tip_shown();
        }
    }

    println!();

    let status = Command::new(&shell)
        .current_dir(&worktree.path)
        .env("GROVE_WORKTREE", &worktree.branch)
        .status();

    match status {
        Ok(s) => {
            if let Some(signal) = s.code() {
                if signal != 0 {
                    println!("{}", "Exited worktree shell.".dimmed());
                }
            } else {
                println!("{}", "Exited worktree shell.".dimmed());
            }
        }
        Err(e) => handle_command_error(&format!("Failed to spawn shell: {}", e)),
    }

    println!("{}", "Exited worktree shell.".dimmed());
}
