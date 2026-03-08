//! Bundled skills: compile-time embedded official skills provisioned on first startup.
//!
//! Skills are extracted to the personal skills directory (`<data_dir>/skills/`)
//! so they appear as auto-enabled personal skills. Existing same-name skills
//! are never overwritten (the user may have modified them).

use std::path::Path;

use include_dir::{include_dir, Dir};
use tracing::{debug, info};

/// Compile-time embedded official skill directories.
static BUNDLED_SKILLS: Dir = include_dir!("$CARGO_MANIFEST_DIR/bundled");

/// Extract bundled skills to the personal skills directory.
///
/// Returns the number of skills newly provisioned. Existing same-name
/// directories are skipped (idempotent, user modifications preserved).
pub async fn provision_bundled_skills(skills_dir: &Path) -> anyhow::Result<usize> {
    tokio::fs::create_dir_all(skills_dir).await?;

    let mut provisioned = 0;
    for dir in BUNDLED_SKILLS.dirs() {
        let name = match dir.path().file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        let target = skills_dir.join(name);
        if target.exists() {
            debug!(skill = name, "bundled skill already exists, skipping");
            continue;
        }
        extract_dir(dir, &target)?;
        provisioned += 1;
        debug!(skill = name, "provisioned bundled skill");
    }

    if provisioned > 0 {
        info!(count = provisioned, "provisioned bundled skills");
    }

    Ok(provisioned)
}

/// Recursively extract an `include_dir` directory to the filesystem.
fn extract_dir(dir: &Dir, target: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(target)?;

    for file in dir.files() {
        let file_name = match file.path().file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        std::fs::write(target.join(file_name), file.contents())?;
    }

    for sub in dir.dirs() {
        let sub_name = match sub.path().file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        extract_dir(sub, &target.join(sub_name))?;
    }

    Ok(())
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_skills_are_embedded() {
        // Verify that the compile-time include captured our bundled skills.
        let names: Vec<&str> = BUNDLED_SKILLS
            .dirs()
            .filter_map(|d| d.path().file_name().and_then(|n| n.to_str()))
            .collect();
        assert!(
            names.contains(&"commit"),
            "expected 'commit' in bundled skills, got: {names:?}"
        );
        assert!(
            names.contains(&"explain"),
            "expected 'explain' in bundled skills, got: {names:?}"
        );
        assert!(names.len() >= 8, "expected at least 8 bundled skills");
    }

    #[tokio::test]
    async fn provision_creates_skills_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let skills_dir = tmp.path().join("skills");

        let count = provision_bundled_skills(&skills_dir).await.unwrap();
        assert!(count >= 8, "expected at least 8 provisioned, got {count}");

        // Each skill should have a SKILL.md
        let commit_md = skills_dir.join("commit").join("SKILL.md");
        assert!(commit_md.exists(), "commit/SKILL.md should exist");

        let content = std::fs::read_to_string(&commit_md).unwrap();
        assert!(content.contains("name: commit"));
    }

    #[tokio::test]
    async fn provision_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let skills_dir = tmp.path().join("skills");

        let first = provision_bundled_skills(&skills_dir).await.unwrap();
        assert!(first >= 8);

        // Second run should provision zero (all exist).
        let second = provision_bundled_skills(&skills_dir).await.unwrap();
        assert_eq!(second, 0, "second provision should skip existing skills");
    }

    #[tokio::test]
    async fn provision_does_not_overwrite_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let skills_dir = tmp.path().join("skills");
        let custom_dir = skills_dir.join("commit");
        std::fs::create_dir_all(&custom_dir).unwrap();
        std::fs::write(custom_dir.join("SKILL.md"), "custom content").unwrap();

        provision_bundled_skills(&skills_dir).await.unwrap();

        // The user's custom content should be preserved.
        let content = std::fs::read_to_string(custom_dir.join("SKILL.md")).unwrap();
        assert_eq!(content, "custom content");
    }
}
