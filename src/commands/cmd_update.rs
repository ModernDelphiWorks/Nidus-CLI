use super::super::core::core_add_paths_dproj::dproj;
use super::super::core::core_lockfile::lockfile;
use super::super::core::core_utils::utils;
use super::super::dto::config_global_dto::ConfigGlobalDTO;
use super::command_trait::cmd_trait::CliCommand;
use clap::{Arg, Command};
use colored::*;
use git2::{FetchOptions, RemoteCallbacks, Repository};

pub struct CommandUpdate;

impl CliCommand for CommandUpdate {

    fn arg() -> Arg {
        Arg::new("")
    }

    fn command() -> Command {
        Command::new("update")
            .about("⏳ Update the Nidus framework dependencies (git pull)")
            .long_about("Fetches and fast-forwards all dependency repositories in ./dependencies.\nTo update custom code-generation templates, use: Nidus template update")
            .arg(
                Arg::new("dep")
                    .long("dep")
                    .value_name("URL_OR_NAME")
                    .help("Update only the dependency matching this URL or repo name"),
            )
    }

    fn execute(_global_dto: &mut ConfigGlobalDTO, _matches: &clap::ArgMatches) {
        let filter = _matches.get_one::<String>("dep").cloned();

        let mainsrc = match _global_dto.get_command_install() {
            Some(cmd) => cmd.mainsrc.clone(),
            None => {
                println!("{}", "❌ No initialization command found.".red());
                return;
            }
        };

        let dependencies: Vec<(String, String)> = match _global_dto.get_command_install() {
            Some(cmd) => cmd.dependencies.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            None => return,
        };

        let mut updated = 0;
        let mut up_to_date = 0;
        let mut failed = 0;

        if let Some(ref f) = filter {
            println!("{} {}\n", "\n🔄 Updating dependency:".cyan(), f.bold());
        } else {
            println!("{}", "\n🔄 Updating dependencies...\n".cyan());
        }

        for (repo_url, branch_hint) in &dependencies {
            // Apply filter: match by full URL or by repo name
            if let Some(ref f) = filter {
                let repo_name = utils::extract_repo_name(repo_url).unwrap_or_default();
                if repo_url != f && repo_name != *f {
                    continue;
                }
            }
            let repo_name = match utils::extract_repo_name(repo_url) {
                Some(n) => n,
                None => {
                    eprintln!("{} Could not extract repo name from: {}", "⚠️".yellow(), repo_url);
                    failed += 1;
                    continue;
                }
            };
            let repo_path = format!("{}/{}", mainsrc, repo_name);

            match update_repo(&repo_path, branch_hint) {
                Ok(UpdateStatus::FastForwarded) => {
                    println!("{} {}", "✅ Updated:".green(), repo_name.green());
                    updated += 1;
                }
                Ok(UpdateStatus::UpToDate) => {
                    println!("{} {}", "🔁 Already up to date:".blue(), repo_name.blue());
                    up_to_date += 1;
                }
                Err(e) => {
                    eprintln!("{} {}: {}", "❌ Failed:".red(), repo_name.red(), e);
                    failed += 1;
                }
            }
        }

        println!("\n{}", "🎯 Update summary".bold().cyan());
        println!("{}: {}", "✅ Updated".bold(), updated.to_string().green());
        println!("{}: {}", "🔁 Up to date".bold(), up_to_date.to_string().blue());
        println!("{}: {}", "❌ Failed".bold(), failed.to_string().red());

        // Write nidus.lock with the current SHA of each dependency
        let deps_map: std::collections::HashMap<String, String> =
            dependencies.into_iter().collect();
        lockfile::write_lock(mainsrc.trim_end_matches('/'), &deps_map);

        // Automatically refresh .dproj search paths after update
        println!("{}", "\n🔗 Refreshing .dproj search paths...".cyan());
        if let Err(e) = dproj::update_all_dprojs_in_cwd(&mainsrc) {
            eprintln!("{} {}", "⚠️  Could not refresh .dproj:".yellow(), e);
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum UpdateStatus {
    FastForwarded,
    UpToDate,
}

/// Fetches and fast-forwards a local repository.
///
/// If `branch_hint` is empty, uses the current HEAD branch.
/// Returns `UpdateStatus` indicating whether the branch advanced or was already up to date.
pub(crate) fn update_repo(repo_path: &str, branch_hint: &str) -> Result<UpdateStatus, String> {
    let repo = Repository::open(repo_path)
        .map_err(|e| format!("Cannot open repository at '{}': {}", repo_path, e))?;

    // Use the current HEAD branch when branch_hint is empty
    let branch_name = if branch_hint.is_empty() {
        let head = repo.head().map_err(|e| format!("Cannot read HEAD: {}", e))?;
        head.shorthand().unwrap_or("main").to_string()
    } else {
        branch_hint.to_string()
    };

    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| format!("Cannot find remote 'origin': {}", e))?;

    // Credentials: SSH agent for private repos, default for public HTTPS
    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(|_url, username, allowed| {
        if allowed.is_ssh_key() {
            git2::Cred::ssh_key_from_agent(username.unwrap_or("git"))
        } else {
            git2::Cred::default()
        }
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    // Update the remote-tracking branch
    let refspec = format!(
        "refs/heads/{}:refs/remotes/origin/{}",
        branch_name, branch_name
    );
    remote
        .fetch(&[&refspec], Some(&mut fetch_opts), None)
        .map_err(|e| format!("Fetch failed: {}", e))?;

    // Locate the newly fetched commit via the remote-tracking ref
    let tracking_ref = format!("refs/remotes/origin/{}", branch_name);
    let fetch_ref = repo
        .find_reference(&tracking_ref)
        .map_err(|e| format!("Cannot find fetched ref '{}': {}", tracking_ref, e))?;
    let fetch_commit = repo
        .reference_to_annotated_commit(&fetch_ref)
        .map_err(|e| format!("Cannot resolve commit: {}", e))?;

    let (analysis, _) = repo
        .merge_analysis(&[&fetch_commit])
        .map_err(|e| format!("Merge analysis failed: {}", e))?;

    if analysis.is_up_to_date() {
        return Ok(UpdateStatus::UpToDate);
    }

    if analysis.is_fast_forward() {
        let local_ref = format!("refs/heads/{}", branch_name);
        repo.find_reference(&local_ref)
            .map_err(|e| format!("Cannot find local branch '{}': {}", local_ref, e))?
            .set_target(fetch_commit.id(), "fast-forward")
            .map_err(|e| format!("Cannot advance branch: {}", e))?;
        repo.set_head(&local_ref)
            .map_err(|e| format!("Cannot set HEAD: {}", e))?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|e| format!("Cannot checkout: {}", e))?;
        return Ok(UpdateStatus::FastForwarded);
    }

    Err(format!(
        "Cannot fast-forward branch '{}' — diverged history, manual merge required",
        branch_name
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::TempDir;

    /// Creates a commit in a non-bare repository
    fn make_commit(repo: &Repository, message: &str) {
        let sig = git2::Signature::now("Test", "test@nidus.dev").unwrap();
        let mut index = repo.index().unwrap();

        let workdir = repo.workdir().unwrap();
        std::fs::write(workdir.join("file.txt"), message).unwrap();
        index.add_path(Path::new("file.txt")).unwrap();
        index.write().unwrap();

        let tree_oid = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        let parents: Vec<git2::Commit> = match repo.head() {
            Ok(head) => vec![head.peel_to_commit().unwrap()],
            Err(_) => vec![],
        };
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
            .unwrap();
    }

    /// Sets up an upstream/clone pair for tests
    fn setup_upstream_and_clone() -> (TempDir, TempDir, String) {
        let upstream_dir = TempDir::new().unwrap();
        let clone_dir = TempDir::new().unwrap();

        let upstream = Repository::init(upstream_dir.path()).unwrap();
        make_commit(&upstream, "initial commit");

        let branch_name = upstream
            .head()
            .unwrap()
            .shorthand()
            .unwrap()
            .to_string();

        let upstream_url = format!("file://{}", upstream_dir.path().display());
        Repository::clone(&upstream_url, clone_dir.path()).unwrap();

        (upstream_dir, clone_dir, branch_name)
    }

    #[test]
    fn test_update_repo_already_up_to_date() {
        let (upstream_dir, clone_dir, branch_name) = setup_upstream_and_clone();
        // No new commit added — repo must be UpToDate
        let result = update_repo(clone_dir.path().to_str().unwrap(), &branch_name);
        assert_eq!(result.unwrap(), UpdateStatus::UpToDate);
        drop(upstream_dir);
    }

    #[test]
    fn test_update_repo_fast_forward() {
        let (upstream_dir, clone_dir, branch_name) = setup_upstream_and_clone();

        // Add a new commit to the upstream
        let upstream = Repository::open(upstream_dir.path()).unwrap();
        make_commit(&upstream, "second commit");

        // Update must fast-forward the clone
        let result = update_repo(clone_dir.path().to_str().unwrap(), &branch_name);
        assert_eq!(result.unwrap(), UpdateStatus::FastForwarded);

        // Verify that the clone was advanced to the new commit
        let clone = Repository::open(clone_dir.path()).unwrap();
        let head_msg = clone
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .message()
            .unwrap()
            .to_string();
        assert!(head_msg.contains("second commit"));
        drop(upstream_dir);
    }

    #[test]
    fn test_update_repo_detects_branch_from_head() {
        let (upstream_dir, clone_dir, _branch_name) = setup_upstream_and_clone();

        let upstream = Repository::open(upstream_dir.path()).unwrap();
        make_commit(&upstream, "third commit");

        // Pass empty branch_hint — should detect the branch from HEAD
        let result = update_repo(clone_dir.path().to_str().unwrap(), "");
        assert_eq!(result.unwrap(), UpdateStatus::FastForwarded);
        drop(upstream_dir);
    }

    #[test]
    fn test_update_repo_nonexistent_path() {
        let result = update_repo("/nonexistent/path/to/repo", "main");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot open repository"));
    }

    #[test]
    fn test_update_repo_multiple_commits_fast_forward() {
        let (upstream_dir, clone_dir, branch_name) = setup_upstream_and_clone();

        let upstream = Repository::open(upstream_dir.path()).unwrap();
        make_commit(&upstream, "commit 2");
        make_commit(&upstream, "commit 3");
        make_commit(&upstream, "commit 4");

        let result = update_repo(clone_dir.path().to_str().unwrap(), &branch_name);
        assert_eq!(result.unwrap(), UpdateStatus::FastForwarded);

        // Verify that the clone reached commit 4
        let clone = Repository::open(clone_dir.path()).unwrap();
        let msg = clone
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .message()
            .unwrap()
            .to_string();
        assert!(msg.contains("commit 4"));
        drop(upstream_dir);
    }
}
