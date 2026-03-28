pub mod lockfile {
    use crate::core::core_utils::utils;
    use crate::dto::lock_dto::NidusLock;
    use git2::Repository;

    /// Reads the HEAD commit SHA from a local git repository.
    pub fn read_commit_sha(repo_path: &str) -> Option<String> {
        Repository::open(repo_path).ok().and_then(|repo| {
            repo.head().ok().and_then(|head| {
                head.peel_to_commit().ok().map(|c| c.id().to_string())
            })
        })
    }

    /// Writes/updates nidus.lock based on the currently cloned repositories.
    /// Called after `install` and `update`.
    pub fn write_lock(mainsrc: &str, dependencies: &std::collections::HashMap<String, String>) {
        let mut lock = NidusLock::load().unwrap_or_default();
        lock.generated_at = chrono::Utc::now().to_rfc3339();

        for (url, branch) in dependencies {
            let name = match utils::extract_repo_name(url) {
                Some(n) => n,
                None => continue,
            };
            let dest = format!("{}/{}", mainsrc, name);
            if let Some(sha) = read_commit_sha(&dest) {
                lock.add_entry(url, branch, &sha);
            }
        }

        if let Err(e) = lock.save() {
            eprintln!("Warning: Could not write nidus.lock: {}", e);
        } else {
            println!("{}", colored::Colorize::dimmed("nidus.lock updated."));
        }
    }
}
