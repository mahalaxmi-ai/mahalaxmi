//! Integration tests for symbol ranking.
//!
//! Verifies the PageRank-style ranking algorithm, kind-specific bonuses,
//! convergence behavior, determinism, and top-N ordering.

use mahalaxmi_indexing::graph::{FileDependency, FileDependencyGraph};
use mahalaxmi_indexing::ranking::{RankingConfig, SymbolRanking};
use mahalaxmi_indexing::types::{ExtractedSymbol, SymbolKind, Visibility};
use std::path::{Path, PathBuf};

fn make_symbol(name: &str, kind: SymbolKind, file: &str) -> ExtractedSymbol {
    ExtractedSymbol::new(
        name.to_owned(),
        kind,
        Visibility::Public,
        PathBuf::from(file),
        1,
        10,
    )
}

// ---------------------------------------------------------------------------
// Star topology ranking
// ---------------------------------------------------------------------------

#[test]
fn ranking_with_star_topology() {
    // One central file (/hub.rs) imported by 5 leaf files.
    // The hub should rank highest.
    let mut graph = FileDependencyGraph::new();
    let hub = PathBuf::from("/hub.rs");

    for i in 0..5 {
        let leaf = PathBuf::from(format!("/leaf_{i}.rs"));
        graph.add_dependency(FileDependency::new(&leaf, &hub, "hub"));
    }

    let symbols = vec![
        make_symbol("hub_fn", SymbolKind::Function, "/hub.rs"),
        make_symbol("leaf_0_fn", SymbolKind::Function, "/leaf_0.rs"),
        make_symbol("leaf_1_fn", SymbolKind::Function, "/leaf_1.rs"),
        make_symbol("leaf_2_fn", SymbolKind::Function, "/leaf_2.rs"),
        make_symbol("leaf_3_fn", SymbolKind::Function, "/leaf_3.rs"),
        make_symbol("leaf_4_fn", SymbolKind::Function, "/leaf_4.rs"),
    ];

    let config = RankingConfig::new();
    let ranking = SymbolRanking::compute(&symbols, &graph, &config);

    let hub_score = ranking.file_score(Path::new("/hub.rs"));
    for i in 0..5 {
        let leaf_score = ranking.file_score(Path::new(&format!("/leaf_{i}.rs")));
        assert!(
            hub_score > leaf_score,
            "hub ({hub_score}) should rank higher than leaf_{i} ({leaf_score})"
        );
    }

    assert!(
        ranking.score_for("hub_fn") > ranking.score_for("leaf_0_fn"),
        "hub_fn should have higher symbol score than leaf_0_fn"
    );
}

// ---------------------------------------------------------------------------
// Damping factor effect
// ---------------------------------------------------------------------------

#[test]
fn ranking_damping_factor_affects_scores() {
    let mut graph = FileDependencyGraph::new();
    graph.add_dependency(FileDependency::new("/a.rs", "/b.rs", "b"));
    graph.add_dependency(FileDependency::new("/c.rs", "/b.rs", "b"));

    let symbols = vec![
        make_symbol("fn_a", SymbolKind::Function, "/a.rs"),
        make_symbol("fn_b", SymbolKind::Function, "/b.rs"),
        make_symbol("fn_c", SymbolKind::Function, "/c.rs"),
    ];

    let config_low = RankingConfig {
        damping_factor: 0.5,
        max_iterations: 50,
        convergence_threshold: 1e-8,
    };
    let config_high = RankingConfig {
        damping_factor: 0.85,
        max_iterations: 50,
        convergence_threshold: 1e-8,
    };

    let ranking_low = SymbolRanking::compute(&symbols, &graph, &config_low);
    let ranking_high = SymbolRanking::compute(&symbols, &graph, &config_high);

    let b_score_low = ranking_low.file_score(Path::new("/b.rs"));
    let b_score_high = ranking_high.file_score(Path::new("/b.rs"));

    // Higher damping factor amplifies the effect of incoming links.
    // /b.rs has 2 incoming edges, so it should benefit more from higher damping.
    // The ratio between hub and leaf scores should differ between the two configs.
    let a_score_low = ranking_low.file_score(Path::new("/a.rs"));
    let a_score_high = ranking_high.file_score(Path::new("/a.rs"));

    let ratio_low = b_score_low / a_score_low.max(1e-12);
    let ratio_high = b_score_high / a_score_high.max(1e-12);

    assert!(
        (ratio_low - ratio_high).abs() > 1e-6,
        "different damping factors should produce different score ratios: \
         ratio_low={ratio_low}, ratio_high={ratio_high}"
    );
}

// ---------------------------------------------------------------------------
// Kind-specific bonuses
// ---------------------------------------------------------------------------

#[test]
fn trait_symbols_get_bonus() {
    let mut graph = FileDependencyGraph::new();
    graph.add_file("/a.rs");

    let symbols = vec![
        make_symbol("MyTrait", SymbolKind::Trait, "/a.rs"),
        make_symbol("my_fn", SymbolKind::Function, "/a.rs"),
    ];
    let config = RankingConfig::new();
    let ranking = SymbolRanking::compute(&symbols, &graph, &config);

    // Trait bonus is 1.5, Function bonus is 1.0
    assert!(
        ranking.score_for("MyTrait") > ranking.score_for("my_fn"),
        "Trait symbol ({}) should score higher than Function ({})",
        ranking.score_for("MyTrait"),
        ranking.score_for("my_fn")
    );
}

#[test]
fn class_symbols_get_bonus() {
    let mut graph = FileDependencyGraph::new();
    graph.add_file("/a.rs");

    let symbols = vec![
        make_symbol("MyClass", SymbolKind::Class, "/a.rs"),
        make_symbol("my_fn", SymbolKind::Function, "/a.rs"),
    ];
    let config = RankingConfig::new();
    let ranking = SymbolRanking::compute(&symbols, &graph, &config);

    // Class bonus is 1.3, Function bonus is 1.0
    assert!(
        ranking.score_for("MyClass") > ranking.score_for("my_fn"),
        "Class symbol ({}) should score higher than Function ({})",
        ranking.score_for("MyClass"),
        ranking.score_for("my_fn")
    );
}

// ---------------------------------------------------------------------------
// Convergence
// ---------------------------------------------------------------------------

#[test]
fn convergence_happens() {
    // Build a ring graph (each node links to the next, last links back to first)
    // so there are no dangling nodes. In a fully-connected ring, PageRank scores
    // converge to 1/N per node and the total sums to 1.0.
    let mut graph = FileDependencyGraph::new();
    let files: Vec<PathBuf> = (0..5)
        .map(|i| PathBuf::from(format!("/f_{i}.rs")))
        .collect();

    // Ring: f0 -> f1 -> f2 -> f3 -> f4 -> f0
    for i in 0..5 {
        let from = &files[i];
        let to = &files[(i + 1) % 5];
        graph.add_dependency(FileDependency::new(from, to, "link"));
    }

    let symbols: Vec<ExtractedSymbol> = (0..5)
        .map(|i| {
            make_symbol(
                &format!("fn_{i}"),
                SymbolKind::Function,
                &format!("/f_{i}.rs"),
            )
        })
        .collect();

    let config = RankingConfig {
        damping_factor: 0.85,
        max_iterations: 100,
        convergence_threshold: 1e-10,
    };
    let ranking = SymbolRanking::compute(&symbols, &graph, &config);

    let total_file_score: f64 = files.iter().map(|f| ranking.file_score(f.as_path())).sum();

    // With no dangling nodes, PageRank total should be ~1.0
    assert!(
        (total_file_score - 1.0).abs() < 0.01,
        "file scores should sum to ~1.0, got {total_file_score}"
    );

    // In a symmetric ring, all file scores should be approximately equal
    let expected_per_file = 1.0 / 5.0;
    for file in &files {
        let score = ranking.file_score(file.as_path());
        assert!(
            (score - expected_per_file).abs() < 0.01,
            "each file in a symmetric ring should score ~{expected_per_file}, got {score} for {}",
            file.display()
        );
    }
}

// ---------------------------------------------------------------------------
// Top N limit
// ---------------------------------------------------------------------------

#[test]
fn top_n_respects_limit() {
    let mut graph = FileDependencyGraph::new();
    for i in 0..10 {
        graph.add_file(format!("/file_{i}.rs"));
    }

    let symbols: Vec<ExtractedSymbol> = (0..10)
        .map(|i| {
            make_symbol(
                &format!("sym_{i}"),
                SymbolKind::Function,
                &format!("/file_{i}.rs"),
            )
        })
        .collect();

    let config = RankingConfig::new();
    let ranking = SymbolRanking::compute(&symbols, &graph, &config);

    let top_3 = ranking.top_n(3);
    assert_eq!(top_3.len(), 3, "top_n(3) should return exactly 3 entries");

    // They should be in descending order
    for i in 1..top_3.len() {
        assert!(
            top_3[i - 1].1 >= top_3[i].1,
            "top_n results should be in descending score order"
        );
    }
}

// ---------------------------------------------------------------------------
// Determinism
// ---------------------------------------------------------------------------

#[test]
fn ranking_deterministic() {
    let mut graph = FileDependencyGraph::new();
    graph.add_dependency(FileDependency::new("/a.rs", "/b.rs", "b"));
    graph.add_dependency(FileDependency::new("/c.rs", "/b.rs", "b"));
    graph.add_file("/d.rs");

    let symbols = vec![
        make_symbol("fn_a", SymbolKind::Function, "/a.rs"),
        make_symbol("fn_b", SymbolKind::Struct, "/b.rs"),
        make_symbol("fn_c", SymbolKind::Trait, "/c.rs"),
        make_symbol("fn_d", SymbolKind::Function, "/d.rs"),
    ];

    let config = RankingConfig::new();

    let ranking_1 = SymbolRanking::compute(&symbols, &graph, &config);
    let ranking_2 = SymbolRanking::compute(&symbols, &graph, &config);

    for sym in &symbols {
        let name = sym.qualified_name();
        let score_1 = ranking_1.score_for(&name);
        let score_2 = ranking_2.score_for(&name);
        assert!(
            (score_1 - score_2).abs() < 1e-12,
            "ranking should be deterministic: {name} got {score_1} vs {score_2}"
        );
    }

    for file in ["/a.rs", "/b.rs", "/c.rs", "/d.rs"] {
        let s1 = ranking_1.file_score(Path::new(file));
        let s2 = ranking_2.file_score(Path::new(file));
        assert!(
            (s1 - s2).abs() < 1e-12,
            "file score for {file} should be deterministic: {s1} vs {s2}"
        );
    }
}

// ---------------------------------------------------------------------------
// Top files ordering
// ---------------------------------------------------------------------------

#[test]
fn top_files_ordering() {
    // Build a graph where /core.rs is imported by many files.
    let mut graph = FileDependencyGraph::new();
    for i in 0..5 {
        graph.add_dependency(FileDependency::new(
            format!("/user_{i}.rs"),
            "/core.rs",
            "core",
        ));
    }
    // Add an isolated file that imports nothing and is imported by nobody.
    graph.add_file("/isolated.rs");

    let config = RankingConfig::new();
    let ranking = SymbolRanking::compute(&[], &graph, &config);

    let top = ranking.top_files(10);
    assert!(
        !top.is_empty(),
        "top_files should return entries for a non-empty graph"
    );

    // Verify descending order
    for i in 1..top.len() {
        assert!(
            top[i - 1].1 >= top[i].1,
            "top_files should be in descending order: {} ({}) vs {} ({})",
            top[i - 1].0.display(),
            top[i - 1].1,
            top[i].0.display(),
            top[i].1,
        );
    }

    // /core.rs should be the highest-ranked file
    assert_eq!(
        top[0].0,
        Path::new("/core.rs"),
        "/core.rs should be the top-ranked file"
    );
}
