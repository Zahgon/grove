use colored::Colorize;

use crate::git::{discover_repo, list_worktrees};
use crate::models::{Worktree, WorktreeListOptions};
use crate::utils::{format_created_time, format_path_with_tilde, handle_command_error};

pub fn run(details: bool, dirty: bool, locked: bool, json: bool) {
    let repo = discover_repo().unwrap_or_else(|e| handle_command_error(&e));

    let options = WorktreeListOptions {
        dirty,
        locked,
        details,
    };

    let worktrees = list_worktrees(&repo).unwrap_or_else(|e| handle_command_error(&e));

    if json {
        let filtered: Vec<&Worktree> = worktrees
            .iter()
            .filter(|wt| should_include_worktree(wt, &options))
            .collect();
        match serde_json::to_string_pretty(&filtered) {
            Ok(output) => println!("{}", output),
            Err(e) => handle_command_error(&format!("Failed to serialize JSON: {}", e)),
        }
        return;
    }

    // Show legend
    println!(
        "{} {} = clean, {} = dirty",
        "Legend:".dimmed(),
        "green".green(),
        "yellow".yellow()
    );
    if details {
        println!("{}", "Symbols: 🔒 = locked, ⚠ = prunable".dimmed());
    }
    println!();

    let mut found_any = false;
    let mut matched_any = false;

    for wt in &worktrees {
        found_any = true;
        if !should_include_worktree(wt, &options) {
            continue;
        }
        matched_any = true;
        print_worktree_item(wt, &options);
    }

    if !found_any {
        println!("{}", "No worktrees found.".yellow());
    } else if !matched_any {
        println!("{}", "No worktrees found matching the criteria.".yellow());
    }
}

fn should_include_worktree(worktree: &Worktree, options: &WorktreeListOptions) -> bool {
    if options.dirty && !worktree.is_dirty {
        return false;
    }
    if options.locked && !worktree.is_locked {
        return false;
    }
    true
}

fn print_worktree_item(worktree: &Worktree, options: &WorktreeListOptions) {
    let display_path = format_path_with_tilde(&worktree.path);

    let branch_display = if worktree.is_dirty {
        format!("[{}]", worktree.branch).yellow().to_string()
    } else {
        format!("[{}]", worktree.branch).green().to_string()
    };

    let mut symbols = String::new();
    if worktree.is_locked {
        symbols.push_str(" 🔒");
    }
    if worktree.is_prunable {
        symbols.push_str(" ⚠");
    }

    let created_str = format_created_time(&worktree.created_at);

    // Calculate widths
    let terminal_width = terminal_size().unwrap_or(80);
    let path_width = std::cmp::max(20, terminal_width / 2);
    let branch_width = std::cmp::max(15, terminal_width * 3 / 10);

    let truncated_path = if display_path.len() > path_width {
        format!(
            "...{}",
            &display_path[display_path.len() - (path_width - 3)..]
        )
    } else {
        display_path.clone()
    };

    let path_spacing = " ".repeat(path_width.saturating_sub(truncated_path.len()));
    let branch_text = format!("[{}]{}", worktree.branch, symbols);
    let branch_spacing = " ".repeat(branch_width.saturating_sub(branch_text.len()));

    println!(
        "{}{}  {}{}{}  {}",
        truncated_path,
        path_spacing,
        branch_display,
        symbols,
        branch_spacing,
        created_str.dimmed()
    );

    if options.details {
        let head_short = if worktree.head.len() > 8 {
            &worktree.head[..8]
        } else {
            &worktree.head
        };
        println!("  {} {}", "→".dimmed(), head_short.dimmed());
    }
}

fn terminal_size() -> Option<usize> {
    // Try to get terminal width. Without tput or a usable answer the caller assumes 80 columns.
    if let Ok(output) = std::process::Command::new("tput").arg("cols").output() {
        if output.status.success() {
            if let Ok(cols) = String::from_utf8_lossy(&output.stdout)
                .trim()
                .parse::<usize>()
            {
                return Some(cols);
            }
        }
    }
    None
}
