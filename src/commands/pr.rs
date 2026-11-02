use colored::Colorize;
use regex::Regex;
use std::process::Command;

use crate::git::{
    add_worktree, discover_repo, fetch_pull_request, find_worktree_by_name, project_root, repo_path,
};
use crate::utils::handle_command_error;

pub fn run(pr_num: u64) {
    // Check gh CLI is available
    if Command::new("gh").arg("--version").output().is_err() {
        handle_command_error(
            "gh CLI is not installed. Please install it from https://cli.github.com/",
        );
    }

    let repo = discover_repo().unwrap_or_else(|e| handle_command_error(&e));

    let bare_repo_path = repo_path(&repo).to_path_buf();
    let project_root = project_root(&repo);

    println!(
        "{}",
        format!("Fetching PR #{} information...", pr_num).dimmed()
    );

    // Get PR info via gh CLI
    let output = Command::new("gh")
        .args([
            "pr",
            "view",
            &pr_num.to_string(),
            "--json",
            "headRefName,headRepository",
        ])
        .current_dir(&bare_repo_path)
        .output();

    let pr_info: serde_json::Value = match output {
        Ok(o) if o.status.success() => serde_json::from_slice(&o.stdout).unwrap_or_else(|_| {
            handle_command_error(&format!("Failed to parse PR #{} info.", pr_num))
        }),
        _ => handle_command_error(&format!(
            "Failed to fetch PR #{}. Make sure the PR exists and you have access to the repository.",
            pr_num
        )),
    };

    let branch_name = pr_info["headRefName"].as_str().unwrap_or("").to_string();

    if branch_name.is_empty() {
        handle_command_error(&format!(
            "Could not determine branch name for PR #{}",
            pr_num
        ));
    }

    let worktree_name = format!("pr-{}-{}", pr_num, sanitize_branch_name(&branch_name));
    let worktree_path = project_root.join(&worktree_name);
    let worktree_path_str = worktree_path.to_string_lossy().to_string();

    // Check if worktree already exists
    let existing =
        find_worktree_by_name(&repo, &worktree_name).unwrap_or_else(|e| handle_command_error(&e));
    if existing.is_some() {
        println!(
            "{} {}",
            "⚠ Worktree already exists:".yellow(),
            worktree_path_str.bold()
        );
        return;
    }

    // Fetch PR branch
    println!(
        "{}",
        format!("Fetching PR branch: {}...", branch_name).dimmed()
    );
    if let Err(e) = fetch_pull_request(&repo, pr_num) {
        handle_command_error(&format!("Failed to fetch PR #{}: {}", pr_num, e));
    }

    // Create worktree
    println!(
        "{}",
        format!("Creating worktree: {}...", worktree_name).dimmed()
    );
    if let Err(e) = add_worktree(
        &repo,
        &worktree_path_str,
        &format!("pr-{}", pr_num),
        false,
        None,
    ) {
        handle_command_error(&e);
    }

    println!(
        "{} {}",
        "✓ Created worktree for PR".green(),
        format!("#{}", pr_num).bold()
    );
    println!("  {} {}", "Branch:".dimmed(), branch_name.bold());
    println!("  {} {}", "Path:".dimmed(), worktree_path_str.bold());
    println!();
    println!("{}", "To switch to this worktree, run:".dimmed());
    println!("  {}", format!("grove go {}", worktree_name).cyan());
}

/// Reduce a branch name to a directory-safe form: anything but ASCII letters, digits, `-` and
/// `_` becomes `-`, runs of `-` collapse to one, and none is left at either end.
fn sanitize_branch_name(branch_name: &str) -> String {
    let replaced: String = branch_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();

    Regex::new("-+")
        .unwrap()
        .replace_all(&replaced, "-")
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_keeps_a_plain_branch_name() {
        assert_eq!(sanitize_branch_name("topic_1"), "topic_1");
    }

    #[test]
    fn sanitize_replaces_unsafe_characters() {
        assert_eq!(sanitize_branch_name("feature/my.fix"), "feature-my-fix");
    }

    #[test]
    fn sanitize_collapses_runs_and_trims_the_ends() {
        assert_eq!(sanitize_branch_name("a---b"), "a-b");
        assert_eq!(sanitize_branch_name("-fix/é!-"), "fix");
    }
}
