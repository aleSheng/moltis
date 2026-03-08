//! External skill registry: discover and browse skills from public directories.
//!
//! Defines a `RegistryProvider` trait for querying external skill catalogs,
//! with a built-in featured list as an offline fallback.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// A skill entry from an external registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySkill {
    /// Install source identifier (e.g. "owner/repo").
    pub source: String,
    /// Skill name (kebab-case).
    pub name: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Short description.
    pub description: String,
    /// Author name.
    #[serde(default)]
    pub author: String,
    /// Category tags.
    #[serde(default)]
    pub categories: Vec<String>,
    /// SPDX license.
    #[serde(default)]
    pub license: Option<String>,
    /// GitHub stars.
    #[serde(default)]
    pub stars: Option<u64>,
    /// Download count.
    #[serde(default)]
    pub downloads: Option<u64>,
    /// Whether the skill is verified/certified.
    #[serde(default)]
    pub verified: bool,
    /// Last update timestamp (ISO 8601).
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Which registry this entry came from.
    #[serde(default)]
    pub registry: String,
    /// Actual source for installation (may differ from `source` for sub-skills).
    #[serde(default)]
    pub install_source: String,
}

/// A skill category for browsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub count: usize,
}

/// Trait for querying an external skill registry.
#[async_trait]
pub trait RegistryProvider: Send + Sync {
    /// Provider name (for UI display).
    fn name(&self) -> &str;

    /// Search skills by keyword.
    async fn search(&self, query: &str, limit: usize) -> anyhow::Result<Vec<RegistrySkill>>;

    /// Get available categories.
    async fn categories(&self) -> anyhow::Result<Vec<Category>>;

    /// Get featured/recommended skills.
    async fn featured(&self) -> anyhow::Result<Vec<RegistrySkill>>;
}

// ── Built-in featured skills (offline fallback) ─────────────────────────────

/// Compile-time embedded featured skills list.
static FEATURED_SKILLS_JSON: &str = include_str!("featured-skills.json");

/// Provider backed by the compiled-in featured skills list.
/// Always available, no network required.
pub struct BuiltinFeaturedProvider;

impl BuiltinFeaturedProvider {
    fn load_skills() -> Vec<RegistrySkill> {
        serde_json::from_str(FEATURED_SKILLS_JSON).unwrap_or_default()
    }
}

#[async_trait]
impl RegistryProvider for BuiltinFeaturedProvider {
    fn name(&self) -> &str {
        "Featured"
    }

    async fn search(&self, query: &str, limit: usize) -> anyhow::Result<Vec<RegistrySkill>> {
        let q = query.to_lowercase();
        let results: Vec<_> = Self::load_skills()
            .into_iter()
            .filter(|s| {
                s.name.to_lowercase().contains(&q)
                    || s.display_name.to_lowercase().contains(&q)
                    || s.description.to_lowercase().contains(&q)
                    || s.categories.iter().any(|c| c.to_lowercase().contains(&q))
            })
            .take(limit)
            .collect();
        Ok(results)
    }

    async fn categories(&self) -> anyhow::Result<Vec<Category>> {
        let skills = Self::load_skills();
        let mut cat_map = std::collections::HashMap::<String, usize>::new();
        for skill in &skills {
            for cat in &skill.categories {
                *cat_map.entry(cat.clone()).or_default() += 1;
            }
        }
        let mut cats: Vec<Category> = cat_map
            .into_iter()
            .map(|(id, count)| Category {
                name: id.replace('-', " "),
                id,
                count,
            })
            .collect();
        cats.sort_by(|a, b| b.count.cmp(&a.count));
        Ok(cats)
    }

    async fn featured(&self) -> anyhow::Result<Vec<RegistrySkill>> {
        Ok(Self::load_skills())
    }
}

// ── Aggregated registry ─────────────────────────────────────────────────────

/// Aggregates multiple `RegistryProvider`s and deduplicates results.
pub struct AggregatedRegistry {
    providers: Vec<Box<dyn RegistryProvider>>,
}

impl AggregatedRegistry {
    pub fn new(providers: Vec<Box<dyn RegistryProvider>>) -> Self {
        Self { providers }
    }

    /// Create with only the built-in featured provider.
    pub fn builtin_only() -> Self {
        Self {
            providers: vec![Box::new(BuiltinFeaturedProvider)],
        }
    }
}

#[async_trait]
impl RegistryProvider for AggregatedRegistry {
    fn name(&self) -> &str {
        "All"
    }

    async fn search(&self, query: &str, limit: usize) -> anyhow::Result<Vec<RegistrySkill>> {
        let mut all = Vec::new();
        for provider in &self.providers {
            match provider.search(query, limit).await {
                Ok(results) => all.extend(results),
                Err(e) => tracing::debug!(provider = provider.name(), %e, "registry search failed"),
            }
        }
        dedup_skills(&mut all);
        all.truncate(limit);
        Ok(all)
    }

    async fn categories(&self) -> anyhow::Result<Vec<Category>> {
        let mut all = Vec::new();
        for provider in &self.providers {
            if let Ok(cats) = provider.categories().await {
                all.extend(cats);
            }
        }
        // Merge duplicate category IDs.
        let mut merged = std::collections::HashMap::<String, Category>::new();
        for cat in all {
            merged
                .entry(cat.id.clone())
                .and_modify(|existing| existing.count += cat.count)
                .or_insert(cat);
        }
        let mut result: Vec<_> = merged.into_values().collect();
        result.sort_by(|a, b| b.count.cmp(&a.count));
        Ok(result)
    }

    async fn featured(&self) -> anyhow::Result<Vec<RegistrySkill>> {
        let mut all = Vec::new();
        for provider in &self.providers {
            if let Ok(featured) = provider.featured().await {
                all.extend(featured);
            }
        }
        dedup_skills(&mut all);
        Ok(all)
    }
}

/// Remove duplicate entries (same source + name), keeping the first occurrence.
fn dedup_skills(skills: &mut Vec<RegistrySkill>) {
    let mut seen = std::collections::HashSet::new();
    skills.retain(|s| seen.insert(format!("{}:{}", s.source, s.name)));
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn featured_skills_json_parses() {
        let skills: Vec<RegistrySkill> =
            serde_json::from_str(FEATURED_SKILLS_JSON).expect("featured-skills.json must parse");
        assert!(!skills.is_empty(), "featured skills list should not be empty");
    }

    #[tokio::test]
    async fn builtin_search_filters() {
        let provider = BuiltinFeaturedProvider;
        let results = provider.search("document", 10).await.unwrap();
        assert!(
            results.iter().any(|s| s.name.contains("doc")),
            "search for 'document' should return doc-related skills"
        );
    }

    #[tokio::test]
    async fn builtin_categories_are_nonempty() {
        let provider = BuiltinFeaturedProvider;
        let cats = provider.categories().await.unwrap();
        assert!(!cats.is_empty());
    }

    #[tokio::test]
    async fn aggregated_deduplicates() {
        let agg = AggregatedRegistry::new(vec![
            Box::new(BuiltinFeaturedProvider),
            Box::new(BuiltinFeaturedProvider),
        ]);
        let featured = agg.featured().await.unwrap();
        let builtin_featured = BuiltinFeaturedProvider.featured().await.unwrap();
        assert_eq!(
            featured.len(),
            builtin_featured.len(),
            "duplicates should be removed"
        );
    }
}
