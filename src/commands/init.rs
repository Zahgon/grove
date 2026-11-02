use colored::Colorize;
use std::fs;
use std::path::Path;

use crate::git::clone_bare_repository;
use crate::utils::{extract_repo_name, find_grove_repo, handle_command_error};

pub fn run(git_url: &str) {
    // Check if we're inside an existing grove repository
    if let Some(existing) = find_grove_repo(None) {
        handle_command_error(&format!(
            "Cannot initialize grove inside an existing grove repository.\nDetected grove repository at: {}\n\nTo create a new grove setup, run 'grove init' from outside this directory hierarchy.",
            existing.display()
        ));
    }

    // Extract repository name from URL
    let repo_name = extract_repo_name(git_url).unwrap_or_else(|e| handle_command_error(&e));

    // Track if we created the directory
    let mut created_dir = false;

    // Create directory with repo name
    if !Path::new(&repo_name).exists() {
        if let Err(e) = fs::create_dir_all(&repo_name) {
            handle_command_error(&format!("Failed to create directory: {}", e));
        }
        created_dir = true;
    }

    // Define bare repo directory
    let bare_repo_dir = format!("{}/{}.git", repo_name, repo_name);

    // Check if directory already exists
    if Path::new(&bare_repo_dir).exists() {
        handle_command_error(&format!("Directory {} already exists", bare_repo_dir));
    }

    if let Err(e) = clone_bare_repository(git_url, &bare_repo_dir) {
        // Clean up on failure - only remove if we created the directory
        if created_dir {
            if let Err(cleanup_err) = fs::remove_dir_all(&repo_name) {
                eprintln!(
                    "{} Failed to clean up {}: {}",
                    "Warning:".yellow(),
                    repo_name,
                    cleanup_err
                );
            }
        }
        handle_command_error(&e);
    }

    println!(
        "{} {}",
        "✓ Initialized worktree setup:".green(),
        repo_name.bold()
    );
    println!("  {} {}", "Bare repository:".dimmed(), bare_repo_dir);
    println!();
    println!("{}", "Next steps:".bold());
    println!("  {} {}", "cd".dimmed(), bare_repo_dir);
    println!("  {} {}", "grove add".dimmed(), "<branch-name>");
}
