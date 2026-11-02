use colored::Colorize;

use crate::models::Worktree;
use crate::utils::handle_command_error;

/// Show an interactive fuzzy search picker and return the selected worktree.
/// Exits when there is nothing to pick, no terminal to ask on, or the pick is cancelled.
pub fn pick_worktree(worktrees: &[&Worktree], prompt: &str) -> Worktree {
    if worktrees.is_empty() {
        eprintln!("{}", "No worktrees found.".yellow());
        std::process::exit(1);
    }

    if !atty::is(atty::Stream::Stdin) {
        eprintln!("{}", "Error: Interactive selection requires a TTY.".red());
        eprintln!(
            "{}",
            "Provide a branch name argument or use 'grove list' instead.".dimmed()
        );
        std::process::exit(1);
    }

    let items: Vec<String> = worktrees
        .iter()
        .map(|wt| format!("{} ({})", wt.branch, wt.path))
        .collect();

    let selection = dialoguer::FuzzySelect::new()
        .with_prompt(prompt)
        .items(&items)
        .interact_opt();

    match selection {
        Ok(Some(idx)) => worktrees[idx].clone(),
        // Escape cancels the pick: nothing is selected, so nothing is acted on.
        Ok(None) => {
            println!("{}", "Selection cancelled.".dimmed());
            std::process::exit(0);
        }
        Err(e) => handle_command_error(&format!("Failed to read the selection: {}", e)),
    }
}
