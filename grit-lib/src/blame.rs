//! Blame line-mapping algorithm.
//!
//! Pure line-correspondence machinery used by `git blame` / `annotate`: given an
//! old and a new version of a file (as line slices), produce a per-new-line map
//! back to the originating old line. Both an exact map (via the configured diff
//! algorithm, honoring the indent heuristic) and a fuzzy fallback (for lines a
//! parent rewrote) are provided. No I/O, no repository access — strings in,
//! mappings out. The `grit` CLI's `compute_blame` drives the per-commit walk and
//! calls these.

use similar::Algorithm as SimilarAlgorithm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlameDiffAlgorithm {
    Myers,
    Histogram,
    Patience,
    Minimal,
}

impl BlameDiffAlgorithm {
    pub fn to_similar(self) -> SimilarAlgorithm {
        match self {
            // The `similar` crate doesn't expose histogram/minimal directly.
            // These mappings are chosen to match expected blame behavior in
            // upstream t8015 parity tests.
            BlameDiffAlgorithm::Myers => SimilarAlgorithm::Myers,
            BlameDiffAlgorithm::Histogram => SimilarAlgorithm::Patience,
            BlameDiffAlgorithm::Patience => SimilarAlgorithm::Patience,
            BlameDiffAlgorithm::Minimal => SimilarAlgorithm::Lcs,
        }
    }
}

pub fn parse_diff_algorithm_name(name: &str) -> Option<BlameDiffAlgorithm> {
    match name.to_ascii_lowercase().as_str() {
        "myers" | "default" => Some(BlameDiffAlgorithm::Myers),
        "histogram" => Some(BlameDiffAlgorithm::Histogram),
        "patience" => Some(BlameDiffAlgorithm::Patience),
        "minimal" => Some(BlameDiffAlgorithm::Minimal),
        _ => None,
    }
}

pub fn should_drop_tail_match_for_myers(
    diff_algorithm: BlameDiffAlgorithm,
    parent_idx: usize,
    current_idx: usize,
    parent_lines: &[&str],
) -> bool {
    if diff_algorithm != BlameDiffAlgorithm::Myers {
        return false;
    }
    if parent_lines.is_empty() || parent_idx + 1 != parent_lines.len() {
        return false;
    }
    // Preserve common append-at-end behavior. We only drop matches where the
    // final parent line got shifted to the right in the child.
    if current_idx <= parent_idx {
        return false;
    }
    // Restrict this heuristic to duplicated low-information tail lines, which
    // are the cases where xdiff/myers tie-breaking differs from `similar`.
    let tail = parent_lines[parent_idx];
    parent_lines.iter().filter(|line| **line == tail).count() >= 2
}

/// Whether the indent heuristic is enabled for this blame run
/// (`--indent-heuristic` / `--no-indent-heuristic` / `diff.indentHeuristic`).
static BLAME_INDENT_HEURISTIC: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn blame_indent_heuristic_enabled() -> bool {
    BLAME_INDENT_HEURISTIC.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn set_blame_indent_heuristic(enabled: bool) {
    BLAME_INDENT_HEURISTIC.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

/// Map each line in `new` to its origin in `old` (if any).
pub fn build_line_map(
    old: &[&str],
    new: &[&str],
    diff_algorithm: BlameDiffAlgorithm,
) -> Vec<Option<usize>> {
    // Ensure trailing newlines so `from_lines` splits consistently
    let mut old_joined = old.join("\n");
    old_joined.push('\n');
    let mut new_joined = new.join("\n");
    new_joined.push('\n');

    // `--indent-heuristic` / `diff.indentHeuristic` shifts ambiguous add/delete
    // runs like Git's xdiff, which changes which lines blame attributes (t4061).
    let ops = crate::diff_indent_heuristic::diff_lines_ops_compacted(
        &old_joined,
        &new_joined,
        diff_algorithm.to_similar(),
        blame_indent_heuristic_enabled(),
    );

    let mut result = vec![None; new.len()];
    for op in ops {
        if let similar::DiffOp::Equal {
            old_index,
            new_index,
            len,
        } = op
        {
            for k in 0..len {
                if new_index + k < result.len() {
                    result[new_index + k] = Some(old_index + k);
                }
            }
        }
    }

    result
}

pub fn build_fuzzy_line_map(
    old: &[&str],
    new: &[&str],
    exact_map: &[Option<usize>],
) -> Vec<Option<usize>> {
    let mut fuzzy_map = exact_map.to_vec();
    let mut used_old = vec![0usize; old.len()];
    for old_idx in fuzzy_map.iter().flatten() {
        if *old_idx < used_old.len() {
            used_old[*old_idx] += 1;
        }
    }

    // First, greedily recover exact-text matches among unresolved lines.
    // This is important for reorders where Myers may not anchor every moved
    // line and fuzzy similarity can otherwise pair the wrong include.
    for new_idx in 0..fuzzy_map.len() {
        if fuzzy_map[new_idx].is_some() {
            continue;
        }
        let mut best: Option<(usize, usize, usize)> = None;
        for old_idx in 0..old.len() {
            if old[old_idx] != new[new_idx] {
                continue;
            }
            let candidate = (used_old[old_idx], old_idx.abs_diff(new_idx), old_idx);
            if best.is_none_or(|b| candidate < b) {
                best = Some(candidate);
            }
        }
        if let Some((_, _, old_idx)) = best {
            fuzzy_map[new_idx] = Some(old_idx);
            used_old[old_idx] += 1;
        }
    }

    let mut anchors: Vec<(usize, usize)> = exact_map
        .iter()
        .enumerate()
        .filter_map(|(new_idx, old_idx)| old_idx.map(|old| (new_idx, old)))
        .collect();
    anchors.sort_unstable();

    let mut prev_new = usize::MAX;
    let mut prev_old = usize::MAX;
    for (next_new, next_old) in anchors
        .iter()
        .copied()
        .chain(std::iter::once((new.len(), old.len())))
    {
        let new_start = if prev_new == usize::MAX {
            0
        } else {
            prev_new + 1
        };
        let new_end = next_new;
        let old_start = if prev_old == usize::MAX {
            0
        } else {
            prev_old + 1
        };
        let old_end = next_old;

        if new_start < new_end && old_start < old_end {
            let segment_matches =
                fuzzy_match_segment(old, old_start, old_end, new, new_start, new_end);
            for (new_idx, old_idx) in segment_matches {
                if fuzzy_map[new_idx].is_none() {
                    fuzzy_map[new_idx] = Some(old_idx);
                    used_old[old_idx] += 1;
                }
            }
        }

        prev_new = next_new;
        prev_old = next_old;
    }

    // Context-aware recovery for split/expanded lines:
    // if an unresolved line sits between mapped neighbors, prefer mapping
    // to those neighboring source lines when there is meaningful overlap.
    for new_idx in 0..fuzzy_map.len() {
        if fuzzy_map[new_idx].is_some() {
            continue;
        }
        let prev_old = (0..new_idx).rev().find_map(|i| fuzzy_map[i]);
        let next_old = ((new_idx + 1)..fuzzy_map.len()).find_map(|i| fuzzy_map[i]);

        let mut candidates = Vec::new();
        if let Some(o) = prev_old {
            candidates.push(o);
        }
        if let Some(o) = next_old {
            if candidates.last().copied() != Some(o) {
                candidates.push(o);
            }
        }

        let mut best: Option<(f64, usize)> = None;
        for old_idx in candidates {
            if old_idx >= old.len() {
                continue;
            }
            let (sim, lcs) = line_similarity_and_lcs(old[old_idx], new[new_idx]);
            let exact_text = old[old_idx].trim() == new[new_idx].trim();
            // Keep this narrow: strong overlap or exact text only.
            if !exact_text && lcs < 6 && !(sim >= 0.35 && lcs >= 3) {
                continue;
            }

            let mut score = lcs as f64 + sim * 10.0;
            score -= 0.2 * used_old[old_idx] as f64;

            if let (Some(lo), Some(hi)) = (prev_old, next_old) {
                if lo <= hi && (old_idx < lo || old_idx > hi) {
                    score -= 3.0;
                }
            }

            if best.is_none_or(|(best_score, best_old)| {
                score > best_score
                    || ((score - best_score).abs() < 1e-9
                        && old_idx.abs_diff(new_idx) < best_old.abs_diff(new_idx))
            }) {
                best = Some((score, old_idx));
            }
        }

        if let Some((_, old_idx)) = best {
            fuzzy_map[new_idx] = Some(old_idx);
            used_old[old_idx] += 1;
        }
    }

    // Final best-effort fill for unresolved lines. This handles cases where
    // anchor segmentation leaves an empty old-range but we still want to
    // pass through ignored commits by similarity.
    for new_idx in 0..fuzzy_map.len() {
        if fuzzy_map[new_idx].is_some() {
            continue;
        }
        let prev_old = (0..new_idx).rev().find_map(|i| fuzzy_map[i]);
        let next_old = ((new_idx + 1)..fuzzy_map.len()).find_map(|i| fuzzy_map[i]);

        let mut best: Option<(f64, usize)> = None;
        for old_idx in 0..old.len() {
            let (sim, lcs) = line_similarity_and_lcs(old[old_idx], new[new_idx]);
            let exact_text = old[old_idx].trim() == new[new_idx].trim();
            let new_len = new[new_idx].trim().chars().count();
            if !exact_text && (new_len < 3 || sim < 0.45 || lcs < 2) {
                continue;
            }

            let mut score = sim;
            score -= 0.004 * old_idx.abs_diff(new_idx) as f64;
            score -= 0.08 * used_old[old_idx] as f64;

            // Encourage monotonic local ordering when neighboring anchors
            // are themselves ordered; avoid forcing this for reorders.
            if let (Some(lo), Some(hi)) = (prev_old, next_old) {
                if lo <= hi {
                    if old_idx < lo {
                        score -= 0.20 * (lo - old_idx) as f64;
                    } else if old_idx > hi {
                        score -= 0.20 * (old_idx - hi) as f64;
                    }
                }
            } else if let Some(lo) = prev_old {
                if old_idx < lo {
                    score -= 0.20 * (lo - old_idx) as f64;
                }
            } else if let Some(hi) = next_old {
                if old_idx > hi {
                    score -= 0.20 * (old_idx - hi) as f64;
                }
            }

            if best.is_none_or(|(best_score, best_idx)| {
                score > best_score
                    || ((score - best_score).abs() < 1e-9
                        && old_idx.abs_diff(new_idx) < best_idx.abs_diff(new_idx))
            }) {
                best = Some((score, old_idx));
            }
        }

        if let Some((_, old_idx)) = best {
            fuzzy_map[new_idx] = Some(old_idx);
            used_old[old_idx] += 1;
        }
    }

    fuzzy_map
}

fn fuzzy_match_segment(
    old: &[&str],
    old_start: usize,
    old_end: usize,
    new: &[&str],
    new_start: usize,
    new_end: usize,
) -> Vec<(usize, usize)> {
    let m = old_end.saturating_sub(old_start);
    let n = new_end.saturating_sub(new_start);
    if m == 0 || n == 0 {
        return Vec::new();
    }

    // DP over new-lines where the state tracks the last matched old-line.
    // State 0 means "no old line selected yet", state s>0 means old index s-1.
    // Transitions keep order (non-decreasing old index), but allow reusing
    // the same old line for multiple split lines in the new content.
    let states = m + 1;
    let neg_inf = f64::NEG_INFINITY;
    let mut dp = vec![neg_inf; states];
    dp[0] = 0.0;

    let mut back_prev = vec![vec![usize::MAX; states]; n + 1];
    let mut back_pick = vec![vec![None; states]; n + 1];

    for j in 0..n {
        let mut next_dp = vec![neg_inf; states];

        for state in 0..states {
            let base = dp[state];
            if !base.is_finite() {
                continue;
            }

            // Option 1: do not match this new line.
            if base > next_dp[state] {
                next_dp[state] = base;
                back_prev[j + 1][state] = state;
                back_pick[j + 1][state] = None;
            }

            // Option 2: match this new line to some old line >= last matched.
            let start_k = if state == 0 { 0 } else { state - 1 };
            for k in start_k..m {
                let old_idx = old_start + k;
                let new_idx = new_start + j;
                let (sim, lcs) = line_similarity_and_lcs(old[old_idx], new[new_idx]);
                let exact_text = old[old_idx].trim() == new[new_idx].trim();
                let new_len = new[new_idx].trim().chars().count();
                if !exact_text && (new_len < 3 || sim < 0.45 || lcs < 2) {
                    continue;
                }

                // Slight locality bias to stabilize tie-breaking.
                let distance = k.abs_diff(j) as f64;
                let score = base + sim - 0.002 * distance;
                let next_state = k + 1;
                if score > next_dp[next_state] {
                    next_dp[next_state] = score;
                    back_prev[j + 1][next_state] = state;
                    back_pick[j + 1][next_state] = Some(k);
                }
            }
        }

        dp = next_dp;
    }

    let mut best_state = 0usize;
    for state in 1..states {
        if dp[state] > dp[best_state] {
            best_state = state;
        }
    }

    let mut matches = Vec::new();
    let mut state = best_state;
    for j in (1..=n).rev() {
        if let Some(k) = back_pick[j][state] {
            matches.push((new_start + (j - 1), old_start + k));
        }
        let prev = back_prev[j][state];
        if prev == usize::MAX {
            break;
        }
        state = prev;
    }
    matches.reverse();
    matches
}

fn line_similarity_and_lcs(a: &str, b: &str) -> (f64, usize) {
    let a = a.trim();
    let b = b.trim();
    if a.is_empty() || b.is_empty() {
        return (0.0, 0);
    }
    if a == b {
        return (1.0, a.chars().count());
    }

    let a = a.to_ascii_lowercase();
    let b = b.to_ascii_lowercase();
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let n = b_chars.len();
    if a_chars.is_empty() || b_chars.is_empty() {
        return (0.0, 0);
    }

    let mut prev = vec![0usize; n + 1];
    let mut curr = vec![0usize; n + 1];
    for i in 1..=a_chars.len() {
        for j in 1..=n {
            curr[j] = if a_chars[i - 1] == b_chars[j - 1] {
                prev[j - 1] + 1
            } else {
                prev[j].max(curr[j - 1])
            };
        }
        std::mem::swap(&mut prev, &mut curr);
        curr.fill(0);
    }

    let lcs = prev[n];
    let sim = (2.0 * lcs as f64) / (a_chars.len() as f64 + b_chars.len() as f64);
    (sim, lcs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_line_map_maps_unchanged_lines_and_drops_replaced() {
        let old = ["alpha", "beta", "gamma"];
        let new = ["alpha", "BETA", "gamma"];
        let map = build_line_map(&old, &new, BlameDiffAlgorithm::Myers);
        // unchanged lines map back to their old index; the replaced line is None.
        assert_eq!(map, vec![Some(0), None, Some(2)]);
    }

    #[test]
    fn build_line_map_identity_for_equal_files() {
        let lines = ["one", "two", "three"];
        let map = build_line_map(&lines, &lines, BlameDiffAlgorithm::Histogram);
        assert_eq!(map, vec![Some(0), Some(1), Some(2)]);
    }

    #[test]
    fn parse_diff_algorithm_name_known_and_unknown() {
        assert_eq!(parse_diff_algorithm_name("myers"), Some(BlameDiffAlgorithm::Myers));
        assert_eq!(
            parse_diff_algorithm_name("histogram"),
            Some(BlameDiffAlgorithm::Histogram)
        );
        assert_eq!(parse_diff_algorithm_name("nope"), None);
    }
}
