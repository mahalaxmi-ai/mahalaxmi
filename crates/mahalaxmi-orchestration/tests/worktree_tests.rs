//! Integration tests for WorktreeManager.
//!
//! Each test creates a temporary git repository and exercises the worktree
//! lifecycle: create, merge, extract partial progress, cleanup.

use mahalaxmi_core::i18n::locale::SupportedLocale;
use mahalaxmi_core::i18n::I18nService;
use mahalaxmi_core::types::{TaskId, WorkerId};
use mahalaxmi_orchestration::worktree::{MergeResult, WorktreeManager};
use std::process::Command;
use tempfile::TempDir;

fn i18n() -> I18nService {
    I18nService::new(SupportedLocale::EnUs)
}

/// Create a temporary directory with an initialized git repo (with an initial commit).
fn setup_git_repo() -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    let path = dir.path();

    run_cmd(path, &["git", "init"]);
    run_cmd(path, &["git", "config", "user.email", "test@test.com"]);
    run_cmd(path, &["git", "config", "user.name", "Test"]);

    // Create an initial commit so HEAD exists
    std::fs::write(path.join("README.md"), "# Test\n").unwrap();
    run_cmd(path, &["git", "add", "."]);
    run_cmd(path, &["git", "commit", "-m", "initial commit"]);

    dir
}

/// Helper to run a command in a directory and assert success.
fn run_cmd(cwd: &std::path::Path, args: &[&str]) {
    let output = Command::new(args[0])
        .args(&args[1..])
        .current_dir(cwd)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {:?}: {}", args, e));
    assert!(
        output.status.success(),
        "command {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn create_worktree_creates_directory_and_branch() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    let worker_id = WorkerId::new(1);
    let task_id = TaskId::new("implement-auth");
    let info = mgr.create_worktree(worker_id, &task_id).unwrap();

    // Verify the worktree directory exists
    assert!(info.path.exists(), "Worktree directory should exist");

    // Verify the branch was created
    let branches = Command::new("git")
        .args(["branch", "--list", &info.branch_name])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let branch_list = String::from_utf8_lossy(&branches.stdout);
    assert!(
        branch_list.contains(&info.branch_name),
        "Branch '{}' should exist in git branches, got: {}",
        info.branch_name,
        branch_list
    );

    // Verify the worktree is tracked in the manager
    assert!(mgr.get_worktree(worker_id).is_some());
}

#[test]
fn create_worktree_twice_same_worker_returns_existing() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    let worker_id = WorkerId::new(1);
    let task_id = TaskId::new("task-a");

    let info1 = mgr.create_worktree(worker_id, &task_id).unwrap();
    let info2 = mgr.create_worktree(worker_id, &task_id).unwrap();

    // Should return the same worktree (idempotent)
    assert_eq!(info1.path, info2.path);
    assert_eq!(info1.branch_name, info2.branch_name);
}

#[test]
fn remove_worktree_cleans_up() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    let worker_id = WorkerId::new(1);
    let task_id = TaskId::new("task-cleanup");
    let info = mgr.create_worktree(worker_id, &task_id).unwrap();
    let worktree_path = info.path.clone();
    let branch = info.branch_name.clone();

    mgr.remove_worktree(worker_id).unwrap();

    // Verify the directory is gone
    assert!(
        !worktree_path.exists(),
        "Worktree directory should be removed"
    );

    // Verify the branch is gone
    let branches = Command::new("git")
        .args(["branch", "--list", &branch])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let branch_list = String::from_utf8_lossy(&branches.stdout).trim().to_string();
    assert!(
        branch_list.is_empty(),
        "Branch should be deleted, got: {}",
        branch_list
    );

    // Verify the manager no longer tracks it
    assert!(mgr.get_worktree(worker_id).is_none());
}

#[test]
fn remove_nonexistent_worktree_is_noop() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    // Should not error when removing a worktree that was never created
    mgr.remove_worktree(WorkerId::new(99)).unwrap();
}

#[test]
fn merge_clean_worktree_succeeds() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    let worker_id = WorkerId::new(1);
    let task_id = TaskId::new("add-feature");
    let info = mgr.create_worktree(worker_id, &task_id).unwrap();

    // Make a commit in the worktree
    std::fs::write(info.path.join("feature.rs"), "fn feature() {}\n").unwrap();
    run_cmd(&info.path, &["git", "add", "."]);
    run_cmd(&info.path, &["git", "commit", "-m", "add feature"]);

    // Merge back
    let result = mgr.merge_worktree(worker_id).unwrap();
    assert!(
        matches!(result, MergeResult::Clean),
        "Clean merge expected, got {:?}",
        result
    );

    // Verify the file exists in the main worktree after merge
    assert!(
        repo.path().join("feature.rs").exists(),
        "Merged file should exist in main worktree"
    );
}

#[test]
fn merge_worktree_no_commits_returns_clean() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    let worker_id = WorkerId::new(1);
    let task_id = TaskId::new("empty-task");
    mgr.create_worktree(worker_id, &task_id).unwrap();

    // No commits in worktree — merge should return Clean
    let result = mgr.merge_worktree(worker_id).unwrap();
    assert!(matches!(result, MergeResult::Clean));
}

#[test]
fn merge_conflicting_worktree_reports_conflict() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    let worker_id = WorkerId::new(1);
    let task_id = TaskId::new("conflict-task");
    let info = mgr.create_worktree(worker_id, &task_id).unwrap();

    // Modify the same file in both main and worktree
    // First, modify in the worktree and commit
    std::fs::write(info.path.join("README.md"), "# Modified by worker\n").unwrap();
    run_cmd(&info.path, &["git", "add", "."]);
    run_cmd(&info.path, &["git", "commit", "-m", "worker change"]);

    // Then, modify in main and commit (conflicting change)
    std::fs::write(repo.path().join("README.md"), "# Modified in main\n").unwrap();
    run_cmd(repo.path(), &["git", "add", "."]);
    run_cmd(repo.path(), &["git", "commit", "-m", "main change"]);

    // Merge should report conflict
    let result = mgr.merge_worktree(worker_id).unwrap();
    match result {
        MergeResult::Conflict {
            conflicting_files, ..
        } => {
            assert!(
                conflicting_files.iter().any(|f| f.contains("README.md")),
                "README.md should be in conflicting files: {:?}",
                conflicting_files
            );
        }
        MergeResult::Clean => panic!("Expected conflict, got clean merge"),
    }
}

#[test]
fn cleanup_all_removes_everything() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    // Create 3 worktrees
    for i in 1..=3 {
        let wid = WorkerId::new(i);
        let tid = TaskId::new(&format!("task-{i}"));
        mgr.create_worktree(wid, &tid).unwrap();
    }

    assert_eq!(mgr.list_active().len(), 3);

    mgr.cleanup_all().unwrap();

    assert_eq!(mgr.list_active().len(), 0);

    // Verify worktree directories are gone
    let worktrees_dir = repo.path().join(".mahalaxmi").join("worktrees");
    assert!(
        !worktrees_dir.exists() || std::fs::read_dir(&worktrees_dir).unwrap().count() == 0,
        "Worktree directories should be cleaned up"
    );
}

#[test]
fn worktree_not_created_if_not_git_repo() {
    let dir = TempDir::new().expect("failed to create temp dir");
    // Do NOT init git here
    let result = WorktreeManager::new(dir.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default());
    assert!(result.is_err(), "Should fail for non-git directory");
}

#[test]
fn extract_partial_progress_shows_modifications() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    let worker_id = WorkerId::new(1);
    let task_id = TaskId::new("partial-work");
    let info = mgr.create_worktree(worker_id, &task_id).unwrap();

    // Make some changes and commit in the worktree
    std::fs::write(info.path.join("partial.rs"), "fn partial() {}\n").unwrap();
    std::fs::write(info.path.join("helper.rs"), "fn help() {}\n").unwrap();
    run_cmd(&info.path, &["git", "add", "."]);
    run_cmd(&info.path, &["git", "commit", "-m", "partial work"]);

    let progress = mgr.extract_partial_progress(worker_id).unwrap();

    assert_eq!(progress.commit_count, 1);
    assert!(
        progress.files_modified.len() >= 2,
        "Should have at least 2 files modified: {:?}",
        progress.files_modified
    );
    assert!(
        progress
            .files_modified
            .iter()
            .any(|f| f.contains("partial.rs")),
        "Should list partial.rs: {:?}",
        progress.files_modified
    );
}

#[test]
fn extract_partial_progress_nonexistent_worker_errors() {
    let repo = setup_git_repo();
    let mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    let result = mgr.extract_partial_progress(WorkerId::new(99));
    assert!(result.is_err());
}

#[test]
fn ensure_gitignore_adds_mahalaxmi_entry() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    // Creating a worktree triggers ensure_gitignore
    let worker_id = WorkerId::new(1);
    let task_id = TaskId::new("gitignore-test");
    mgr.create_worktree(worker_id, &task_id).unwrap();

    let gitignore = std::fs::read_to_string(repo.path().join(".gitignore")).unwrap();
    assert!(
        gitignore.contains(".mahalaxmi/"),
        ".gitignore should contain .mahalaxmi/ entry, got: {}",
        gitignore
    );
}

#[test]
fn ensure_gitignore_idempotent() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    // Create two worktrees — ensure_gitignore runs twice
    let wid1 = WorkerId::new(1);
    let wid2 = WorkerId::new(2);
    mgr.create_worktree(wid1, &TaskId::new("t1")).unwrap();

    // Remove first to allow second creation
    mgr.remove_worktree(wid1).unwrap();
    mgr.create_worktree(wid2, &TaskId::new("t2")).unwrap();

    let gitignore = std::fs::read_to_string(repo.path().join(".gitignore")).unwrap();
    let count = gitignore
        .lines()
        .filter(|l| l.trim() == ".mahalaxmi/")
        .count();
    assert_eq!(
        count, 1,
        ".mahalaxmi/ should appear exactly once in .gitignore, got {} times",
        count
    );
}

#[test]
fn list_active_returns_all_worktrees() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    for i in 1..=3 {
        mgr.create_worktree(WorkerId::new(i), &TaskId::new(&format!("task-{i}")))
            .unwrap();
    }

    let active = mgr.list_active();
    assert_eq!(active.len(), 3);
}

#[test]
fn create_worktree_succeeds_when_stale_branch_exists() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    let worker_id = WorkerId::new(1);
    let task_id = TaskId::new("task-0");

    // Create and remove a worktree (simulates a previous cycle with BranchOnly
    // strategy that keeps the branch but removes the worktree directory).
    let info = mgr.create_worktree(worker_id, &task_id).unwrap();
    let branch_name = info.branch_name.clone();

    // Remove worktree directory but keep the branch (like BranchOnly does)
    mgr.remove_worktree_keep_branch(worker_id).unwrap();

    // Verify stale branch exists
    let branches = Command::new("git")
        .args(["branch", "--list", &branch_name])
        .current_dir(repo.path())
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&branches.stdout).contains(&branch_name),
        "Stale branch should still exist"
    );

    // Now a new cycle tries to create the same worktree — should succeed.
    // Use Ignore policy: the previous cycle may have written .gitignore (ensure_gitignore),
    // which leaves the working tree dirty. That is expected between back-to-back cycles.
    let mut mgr2 = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::Ignore).unwrap();
    let info2 = mgr2.create_worktree(worker_id, &task_id).unwrap();

    assert!(
        info2.path.exists(),
        "New worktree should be created despite stale branch"
    );
    // Branch names include a UUID suffix so each cycle gets a unique branch.
    // The new branch must differ from the stale one and must still contain
    // the task slug so it's identifiable.
    assert_ne!(
        info2.branch_name, branch_name,
        "New cycle should get a new unique branch name (UUID suffix)"
    );
    assert!(
        info2.branch_name.contains("task-0"),
        "New branch should contain the task slug"
    );
}

/// Set up a git repo with a local bare repo acting as origin.
/// Returns `(main_repo_dir, bare_repo_dir)`.  Both TempDirs must be kept
/// alive for the duration of the test.
fn setup_repo_with_remote() -> (TempDir, TempDir) {
    let bare_dir = TempDir::new().unwrap();
    run_cmd(bare_dir.path(), &["git", "init", "--bare"]);

    let repo_dir = TempDir::new().unwrap();
    let path = repo_dir.path();
    run_cmd(path, &["git", "init"]);
    run_cmd(path, &["git", "config", "user.email", "test@test.com"]);
    run_cmd(path, &["git", "config", "user.name", "Test"]);
    run_cmd(
        path,
        &[
            "git",
            "remote",
            "add",
            "origin",
            bare_dir.path().to_str().unwrap(),
        ],
    );

    std::fs::write(path.join("README.md"), "# Test\n").unwrap();
    run_cmd(path, &["git", "add", "."]);
    run_cmd(path, &["git", "commit", "-m", "initial commit"]);
    // Push to create the target branch on origin
    run_cmd(path, &["git", "push", "origin", "HEAD:main"]);

    (repo_dir, bare_dir)
}

/// Create a fresh repo, fetch `origin/main` from the bare repo, make a commit,
/// and push it back.  Uses `git init` + fetch instead of clone to avoid
/// detached-HEAD issues when the bare repo's symbolic HEAD points to a branch
/// name that differs from what was pushed (e.g. `master` vs `main`).
fn push_remote_change(bare_path: &std::path::Path, filename: &str, content: &str) {
    let tmp = TempDir::new().unwrap();
    let p = tmp.path();
    run_cmd(p, &["git", "init"]);
    run_cmd(p, &["git", "config", "user.email", "test@test.com"]);
    run_cmd(p, &["git", "config", "user.name", "Test"]);
    run_cmd(
        p,
        &[
            "git",
            "remote",
            "add",
            "origin",
            bare_path.to_str().unwrap(),
        ],
    );
    run_cmd(p, &["git", "fetch", "origin"]);
    // Check out origin/main as a local tracking branch named "main".
    run_cmd(p, &["git", "checkout", "-b", "main", "origin/main"]);
    std::fs::write(p.join(filename), content).unwrap();
    run_cmd(p, &["git", "add", "."]);
    run_cmd(
        p,
        &[
            "git",
            "commit",
            "-m",
            &format!("remote change: {}", filename),
        ],
    );
    run_cmd(p, &["git", "push", "origin", "main"]);
    // tmp dropped here — push is already done
}

#[test]
fn push_branch_merges_onto_target_when_provided() {
    let (repo, bare) = setup_repo_with_remote();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    // Create a worker worktree and commit a new file (no conflict expected).
    let worker_id = WorkerId::new(1);
    let info = mgr
        .create_worktree(worker_id, &TaskId::new("add-feature"))
        .unwrap();
    std::fs::write(info.path.join("feature.rs"), "fn feature() {}\n").unwrap();
    run_cmd(&info.path, &["git", "add", "."]);
    run_cmd(&info.path, &["git", "commit", "-m", "add feature"]);

    // Advance origin/main with a non-conflicting change to a different file.
    push_remote_change(bare.path(), "other.rs", "fn other() {}\n");

    // push_branch with target="main" should fetch, merge, and push cleanly.
    let result = mgr.push_branch(worker_id, Some("main"));
    assert!(
        result.is_ok(),
        "push_branch should succeed after non-conflicting merge: {:?}",
        result
    );

    // Verify the branch was pushed to origin.
    let ls_remote = Command::new("git")
        .args(["ls-remote", "--heads", "origin"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let remote_refs = String::from_utf8_lossy(&ls_remote.stdout);
    assert!(
        remote_refs.contains(&info.branch_name),
        "Worker branch '{}' should be on remote after push; remote refs: {}",
        info.branch_name,
        remote_refs
    );
}

#[test]
fn push_branch_resolves_conflict_with_worker_changes() {
    // When the same file is changed by both the worker and a concurrent remote
    // commit, push_branch must succeed by keeping the worker's version (-X ours
    // in the merge / -X theirs in the rebase fallback).  Workers have
    // authoritative ownership of their assigned files (C1 constraint).
    let (repo, bare) = setup_repo_with_remote();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    // Worker worktree: write a "worker" version of README.md and commit.
    let worker_id = WorkerId::new(1);
    let info = mgr
        .create_worktree(worker_id, &TaskId::new("conflict-task"))
        .unwrap();
    std::fs::write(info.path.join("README.md"), "# Worker version\n").unwrap();
    run_cmd(&info.path, &["git", "add", "."]);
    run_cmd(
        &info.path,
        &["git", "commit", "-m", "worker changes README"],
    );

    // Advance origin/main with a conflicting change to the same file.
    push_remote_change(bare.path(), "README.md", "# Remote version\n");

    // push_branch must succeed — -X ours resolves the conflict in the worker's favour.
    let result = mgr.push_branch(worker_id, Some("main"));
    assert!(
        result.is_ok(),
        "push_branch should succeed by resolving conflict with worker's changes: {:?}",
        result
    );

    // Verify the worker's version of the file is preserved after the merge.
    let content = std::fs::read_to_string(info.path.join("README.md")).unwrap();
    assert_eq!(
        content, "# Worker version\n",
        "Worker's file content should win the conflict resolution"
    );

    // Verify the branch was pushed to origin.
    let ls_remote = Command::new("git")
        .args(["ls-remote", "--heads", "origin"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let remote_refs = String::from_utf8_lossy(&ls_remote.stdout);
    assert!(
        remote_refs.contains(&info.branch_name),
        "Worker branch '{}' should be on remote after conflict-resolved push; remote refs: {}",
        info.branch_name,
        remote_refs
    );
}

#[test]
fn branch_name_sanitization() {
    let repo = setup_git_repo();
    let mut mgr = WorktreeManager::new(repo.path().to_path_buf(), i18n(), mahalaxmi_core::config::DirtyBranchPolicy::default()).unwrap();

    // Task ID with special characters
    let worker_id = WorkerId::new(1);
    let task_id = TaskId::new("fix: broken/auth [urgent]");
    let info = mgr.create_worktree(worker_id, &task_id).unwrap();

    // Branch name should not contain special git characters
    assert!(
        !info.branch_name.contains('/') || info.branch_name.starts_with("mahalaxmi/"),
        "Branch name should only have the expected slash prefix: {}",
        info.branch_name
    );
    assert!(
        !info.branch_name.contains('[') && !info.branch_name.contains(']'),
        "Branch name should not contain brackets: {}",
        info.branch_name
    );
}
