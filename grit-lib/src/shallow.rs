//! Shallow repository metadata (`.git/shallow`).

use crate::objects::{parse_commit, ObjectId, ObjectKind};
use crate::odb::Odb;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Returns the set of commit OIDs recorded as shallow boundaries in `.git/shallow`.
///
/// For each listed commit, history must not be traversed past its parents (the parents may be
/// absent from the object database). This matches Git's behavior for `git fsck` and reachability.
#[must_use]
pub fn load_shallow_boundaries(git_dir: &Path) -> HashSet<ObjectId> {
    let shallow_path = git_dir.join("shallow");
    let mut set = HashSet::new();
    let Ok(contents) = fs::read_to_string(&shallow_path) else {
        return set;
    };
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Ok(oid) = line.parse::<ObjectId>() {
            set.insert(oid);
        }
    }
    set
}

/// Add `new_boundaries` to `.git/shallow`, preserving any boundaries already recorded.
///
/// Mirrors `update_shallow_ref` in upstream `receive-pack`, which appends the grafts introduced by
/// a shallow push to the receiver's `.git/shallow` file. Existing boundaries are kept and the union
/// is written back sorted (Git writes the file deduplicated; ordering is not significant to readers).
///
/// # Errors
/// Returns an error if the `.git/shallow` file cannot be written.
pub fn add_shallow_boundaries(
    git_dir: &Path,
    new_boundaries: &HashSet<ObjectId>,
) -> std::io::Result<()> {
    if new_boundaries.is_empty() {
        return Ok(());
    }
    let mut all = load_shallow_boundaries(git_dir);
    all.extend(new_boundaries.iter().copied());
    let mut hexes: Vec<String> = all.iter().map(ObjectId::to_hex).collect();
    hexes.sort();
    let mut body = hexes.join("\n");
    body.push('\n');
    fs::write(git_dir.join("shallow"), body)
}

/// Determine which of `boundaries` become *new* shallow roots when `tip` is pushed into a receiver
/// whose existing reachable history is described by `have_tips`.
///
/// This mirrors the receive-pack rule that a push introduces a "shallow update" when the pushed
/// history bottoms out at a graft commit (one whose parent objects were not transferred) that the
/// receiver cannot already reach from its own refs. The walk descends from `tip` through the
/// `source_odb`, stopping at:
///
/// * commits already reachable from `have_tips` (the receiver "has" them, so no new root), and
/// * commits in `boundaries` (recorded grafts — their parents are intentionally absent).
///
/// A boundary commit reached during the walk *before* hitting a have-commit is returned as a new
/// shallow root. An empty result means the push needs no shallow update for this tip.
///
/// `source_odb` is the *pushing* side's object store (it holds the boundary commits and the new
/// history); `have_tips` are the receiver's current ref tips (expressed as commit OIDs the receiver
/// already has).
#[must_use]
pub fn new_shallow_roots_for_push(
    source_odb: &Odb,
    tip: ObjectId,
    boundaries: &HashSet<ObjectId>,
    have_tips: &[ObjectId],
) -> HashSet<ObjectId> {
    let mut new_roots = HashSet::new();
    if boundaries.is_empty() {
        return new_roots;
    }

    // Commits the receiver already has (everything reachable from its ref tips, walked through the
    // source object store since the receiver's tips are a subset of the source's history for a
    // fast-forward-style push). Walking here marks the "uninteresting" cut for the descent below.
    let mut have: HashSet<ObjectId> = HashSet::new();
    let mut have_stack: Vec<ObjectId> = have_tips.to_vec();
    while let Some(oid) = have_stack.pop() {
        if !have.insert(oid) {
            continue;
        }
        // Do not walk past a boundary when collecting haves: the receiver only has the boundary
        // commit, not its ancestors.
        if boundaries.contains(&oid) {
            continue;
        }
        if let Some(parents) = commit_parents(source_odb, &oid) {
            have_stack.extend(parents);
        }
    }

    let mut seen: HashSet<ObjectId> = HashSet::new();
    let mut stack = vec![tip];
    while let Some(oid) = stack.pop() {
        if have.contains(&oid) {
            continue;
        }
        if !seen.insert(oid) {
            continue;
        }
        if boundaries.contains(&oid) {
            // Reached a graft commit not already on the receiver: this is a new shallow root.
            new_roots.insert(oid);
            continue;
        }
        if let Some(parents) = commit_parents(source_odb, &oid) {
            stack.extend(parents);
        }
    }

    new_roots
}

/// Read the parent OIDs of a commit object from `odb`, or `None` if `oid` is absent or not a commit.
fn commit_parents(odb: &Odb, oid: &ObjectId) -> Option<Vec<ObjectId>> {
    let obj = odb.read(oid).ok()?;
    if obj.kind != ObjectKind::Commit {
        return None;
    }
    let commit = parse_commit(&obj.data).ok()?;
    Some(commit.parents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

    /// Write a commit with the empty tree and the given parents, returning its OID.
    fn write_commit(odb: &Odb, parents: &[ObjectId], msg: &str) -> ObjectId {
        let mut body = format!("tree {EMPTY_TREE}\n");
        for p in parents {
            body.push_str(&format!("parent {}\n", p.to_hex()));
        }
        body.push_str("author A <a@x> 1 +0000\ncommitter A <a@x> 1 +0000\n\n");
        body.push_str(msg);
        body.push('\n');
        odb.write_loose_materialize(ObjectKind::Commit, body.as_bytes())
            .expect("write commit")
    }

    #[test]
    fn no_boundaries_means_no_shallow_roots() {
        let dir = tempdir().unwrap();
        let odb = Odb::new(&dir.path().join("objects"));
        let root = write_commit(&odb, &[], "root");
        let tip = write_commit(&odb, &[root], "tip");
        let roots = new_shallow_roots_for_push(&odb, tip, &HashSet::new(), &[]);
        assert!(roots.is_empty());
    }

    #[test]
    fn boundary_not_on_receiver_is_a_new_root() {
        // Source history: a(boundary) <- b <- c, with `a`'s parent intentionally absent.
        let dir = tempdir().unwrap();
        let odb = Odb::new(&dir.path().join("objects"));
        let a = write_commit(&odb, &[], "a");
        let b = write_commit(&odb, &[a], "b");
        let c = write_commit(&odb, &[b], "c");
        let mut boundaries = HashSet::new();
        boundaries.insert(a);
        // Receiver has nothing reachable yet, so pushing `c` introduces graft `a`.
        let roots = new_shallow_roots_for_push(&odb, c, &boundaries, &[]);
        assert_eq!(roots, boundaries);
    }

    #[test]
    fn boundary_already_on_receiver_is_not_a_new_root() {
        // If the receiver already has the boundary commit (and its ancestry), no shallow update.
        let dir = tempdir().unwrap();
        let odb = Odb::new(&dir.path().join("objects"));
        let a = write_commit(&odb, &[], "a");
        let b = write_commit(&odb, &[a], "b");
        let c = write_commit(&odb, &[b], "c");
        let mut boundaries = HashSet::new();
        boundaries.insert(a);
        // Receiver already has `b` (hence `a` via the walk), so the boundary is covered.
        let roots = new_shallow_roots_for_push(&odb, c, &boundaries, &[b]);
        assert!(roots.is_empty());
    }

    #[test]
    fn add_shallow_boundaries_unions_existing() {
        let dir = tempdir().unwrap();
        let git_dir = dir.path();
        let a = ObjectId::from_hex("1111111111111111111111111111111111111111").unwrap();
        let b = ObjectId::from_hex("2222222222222222222222222222222222222222").unwrap();
        let mut first = HashSet::new();
        first.insert(a);
        add_shallow_boundaries(git_dir, &first).unwrap();
        let mut second = HashSet::new();
        second.insert(b);
        add_shallow_boundaries(git_dir, &second).unwrap();
        let loaded = load_shallow_boundaries(git_dir);
        assert!(loaded.contains(&a));
        assert!(loaded.contains(&b));
        assert_eq!(loaded.len(), 2);
    }
}
