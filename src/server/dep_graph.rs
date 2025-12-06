//! Dependency graph for tracking reverse include relationships between files.
//!
//! This module tracks which files include a given file, enabling "find references"
//! to search dependent files. Forward dependencies (what a file includes) are
//! already tracked in `ParsedCode.includes`.

use std::collections::{HashMap, HashSet};

use lsp_types::Url;

/// Tracks which files include a given file (reverse dependency edges).
///
/// For file A that includes file B, `included_by[B]` contains A.
/// This enables finding all files that might reference symbols from B.
#[derive(Default)]
pub struct DependencyGraph {
    /// Reverse edges: file -> files that include it
    included_by: HashMap<Url, HashSet<Url>>,
    /// Track what each file includes, so we can update reverse edges on change
    includes_cache: HashMap<Url, HashSet<Url>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update the graph when a file is parsed/changed.
    /// Removes old reverse edges for this file and adds new ones.
    pub fn update_file(&mut self, file: &Url, new_includes: &[Url]) {
        // Remove old reverse edges
        if let Some(old_includes) = self.includes_cache.remove(file) {
            for old_include in old_includes {
                if let Some(dependents) = self.included_by.get_mut(&old_include) {
                    dependents.remove(file);
                    if dependents.is_empty() {
                        self.included_by.remove(&old_include);
                    }
                }
            }
        }

        // Add new reverse edges
        if !new_includes.is_empty() {
            let includes_set: HashSet<Url> = new_includes.iter().cloned().collect();

            for include in &includes_set {
                self.included_by
                    .entry(include.clone())
                    .or_default()
                    .insert(file.clone());
            }

            self.includes_cache.insert(file.clone(), includes_set);
        }
    }

    /// Remove a file from the graph (when closed/deleted).
    pub fn remove_file(&mut self, file: &Url) {
        // Remove reverse edges where this file is the dependent
        if let Some(old_includes) = self.includes_cache.remove(file) {
            for old_include in old_includes {
                if let Some(dependents) = self.included_by.get_mut(&old_include) {
                    dependents.remove(file);
                    if dependents.is_empty() {
                        self.included_by.remove(&old_include);
                    }
                }
            }
        }

        // Remove reverse edges where this file is the dependency
        // (files that include this one will be updated when re-parsed)
        self.included_by.remove(file);
    }

    /// Get all files that directly include this file.
    /// Returns files in arbitrary order.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn get_dependents(&self, file: &Url) -> Vec<Url> {
        self.included_by
            .get(file)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all files that include this file, recursively.
    /// This finds all transitive dependents.
    pub fn get_all_dependents(&self, file: &Url) -> Vec<Url> {
        let mut result = HashSet::new();
        let mut queue = vec![file.clone()];

        while let Some(current) = queue.pop() {
            if let Some(dependents) = self.included_by.get(&current) {
                for dep in dependents {
                    if result.insert(dep.clone()) {
                        queue.push(dep.clone());
                    }
                }
            }
        }

        result.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(s: &str) -> Url {
        Url::parse(&format!("file:///{s}")).unwrap()
    }

    #[test]
    fn test_update_file_adds_edges() {
        let mut graph = DependencyGraph::new();

        // A includes B and C
        graph.update_file(&url("a.scad"), &[url("b.scad"), url("c.scad")]);

        assert_eq!(graph.get_dependents(&url("b.scad")), vec![url("a.scad")]);
        assert_eq!(graph.get_dependents(&url("c.scad")), vec![url("a.scad")]);
        assert!(graph.get_dependents(&url("a.scad")).is_empty());
    }

    #[test]
    fn test_update_file_replaces_edges() {
        let mut graph = DependencyGraph::new();

        // A includes B
        graph.update_file(&url("a.scad"), &[url("b.scad")]);
        assert_eq!(graph.get_dependents(&url("b.scad")), vec![url("a.scad")]);

        // Now A includes C instead
        graph.update_file(&url("a.scad"), &[url("c.scad")]);
        assert!(graph.get_dependents(&url("b.scad")).is_empty());
        assert_eq!(graph.get_dependents(&url("c.scad")), vec![url("a.scad")]);
    }

    #[test]
    fn test_remove_file() {
        let mut graph = DependencyGraph::new();

        graph.update_file(&url("a.scad"), &[url("b.scad")]);
        graph.remove_file(&url("a.scad"));

        assert!(graph.get_dependents(&url("b.scad")).is_empty());
    }

    #[test]
    fn test_multiple_dependents() {
        let mut graph = DependencyGraph::new();

        // Both A and C include B
        graph.update_file(&url("a.scad"), &[url("b.scad")]);
        graph.update_file(&url("c.scad"), &[url("b.scad")]);

        let dependents = graph.get_dependents(&url("b.scad"));
        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&url("a.scad")));
        assert!(dependents.contains(&url("c.scad")));
    }

    #[test]
    fn test_transitive_dependents() {
        let mut graph = DependencyGraph::new();

        // A includes B, B includes C
        graph.update_file(&url("a.scad"), &[url("b.scad")]);
        graph.update_file(&url("b.scad"), &[url("c.scad")]);

        // Direct dependents of C is just B
        assert_eq!(graph.get_dependents(&url("c.scad")), vec![url("b.scad")]);

        // All dependents of C includes both A and B
        let all_deps = graph.get_all_dependents(&url("c.scad"));
        assert_eq!(all_deps.len(), 2);
        assert!(all_deps.contains(&url("a.scad")));
        assert!(all_deps.contains(&url("b.scad")));
    }

    #[test]
    fn test_update_file_empty_includes() {
        let mut graph = DependencyGraph::new();
        graph.update_file(&url("a.scad"), &[url("b.scad")]);
        graph.update_file(&url("a.scad"), &[]); // Clear includes
        assert!(graph.get_dependents(&url("b.scad")).is_empty());
    }

    #[test]
    fn test_remove_nonexistent_file() {
        let mut graph = DependencyGraph::new();
        graph.remove_file(&url("nonexistent.scad")); // Should not panic
        assert!(graph.get_dependents(&url("nonexistent.scad")).is_empty());
    }

    #[test]
    fn test_circular_includes() {
        let mut graph = DependencyGraph::new();
        graph.update_file(&url("a.scad"), &[url("b.scad")]);
        graph.update_file(&url("b.scad"), &[url("a.scad")]);

        // Should not infinite loop - cycle detection via HashSet
        let deps = graph.get_all_dependents(&url("a.scad"));
        assert!(deps.contains(&url("b.scad")));
    }

    #[test]
    fn test_diamond_dependency() {
        let mut graph = DependencyGraph::new();
        // A includes B and C, B and C both include D
        graph.update_file(&url("a.scad"), &[url("b.scad"), url("c.scad")]);
        graph.update_file(&url("b.scad"), &[url("d.scad")]);
        graph.update_file(&url("c.scad"), &[url("d.scad")]);

        // D's direct dependents are B and C
        let direct = graph.get_dependents(&url("d.scad"));
        assert_eq!(direct.len(), 2);

        // D's all dependents are A, B, and C
        let all = graph.get_all_dependents(&url("d.scad"));
        assert_eq!(all.len(), 3);
        assert!(all.contains(&url("a.scad")));
        assert!(all.contains(&url("b.scad")));
        assert!(all.contains(&url("c.scad")));
    }
}
