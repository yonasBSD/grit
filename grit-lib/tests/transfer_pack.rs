//! Integration tests for the negotiation-driven pack builder
//! (`grit_lib::transfer::build_pack`).
//!
//! These build a tiny repo with the system `git`, then assert that
//! `build_pack(wants, haves)` selects *only* the objects newly introduced by the
//! wanted tip (the minimal-selection regression guard for the 478 MB finding),
//! and that the produced bytes are a structurally valid PACK v2 stream.

use std::path::Path;
use std::process::Command;

use grit_lib::objects::ObjectId;
use grit_lib::odb::Odb;
use grit_lib::transfer::{build_pack, PackBuildOptions};
use grit_lib::unpack_objects::pack_bytes_to_object_map;

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "T")
        .env("GIT_AUTHOR_EMAIL", "t@example.com")
        .env("GIT_COMMITTER_NAME", "T")
        .env("GIT_COMMITTER_EMAIL", "t@example.com")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 git output")
}

fn rev_parse(dir: &Path, rev: &str) -> ObjectId {
    let hex = git(dir, &["rev-parse", rev]);
    ObjectId::from_hex(hex.trim()).expect("valid oid")
}

/// Header object count from a PACK v2 byte stream.
fn pack_header_count(bytes: &[u8]) -> u32 {
    assert_eq!(&bytes[0..4], b"PACK", "must start with PACK magic");
    assert_eq!(u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]), 2);
    u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]])
}

struct Fixture {
    dir: tempfile::TempDir,
    c1: ObjectId,
    c2: ObjectId,
    /// Objects introduced by C1: the commit, its tree, and its blob.
    c1_objects: Vec<ObjectId>,
    /// Objects introduced by C2 only: the new commit, the new tree, the new blob.
    c2_new_objects: Vec<ObjectId>,
}

fn build_fixture() -> Fixture {
    let tmp = tempfile::tempdir().expect("tempdir");
    let dir = tmp.path();

    git(dir, &["init", "-q", "-b", "main", "."]);

    // C1: base commit with one file.
    std::fs::write(dir.join("a.txt"), b"hello\n").unwrap();
    git(dir, &["add", "a.txt"]);
    git(dir, &["commit", "-q", "-m", "c1"]);
    let c1 = rev_parse(dir, "HEAD");

    let c1_tree = rev_parse(dir, "HEAD^{tree}");
    let c1_blob = rev_parse(dir, "HEAD:a.txt");

    // C2: add a second file (introduces a new commit, a new root tree, a new blob).
    std::fs::write(dir.join("b.txt"), b"world\n").unwrap();
    git(dir, &["add", "b.txt"]);
    git(dir, &["commit", "-q", "-m", "c2"]);
    let c2 = rev_parse(dir, "HEAD");

    let c2_tree = rev_parse(dir, "HEAD^{tree}");
    let c2_blob = rev_parse(dir, "HEAD:b.txt");

    Fixture {
        dir: tmp,
        c1,
        c2,
        c1_objects: vec![c1, c1_tree, c1_blob],
        // a.txt blob is unchanged in C2, so it is NOT a new object.
        c2_new_objects: vec![c2, c2_tree, c2_blob],
    }
}

fn open_odb(dir: &Path) -> Odb {
    let git_dir = dir.join(".git");
    Odb::new(&git_dir.join("objects")).with_config_git_dir(git_dir)
}

#[test]
fn build_pack_selects_only_new_objects() {
    let fx = build_fixture();
    let odb = open_odb(fx.dir.path());

    let pack = build_pack(&odb, &[fx.c2], &[fx.c1], &PackBuildOptions::default()).expect("build");

    // (a) starts with PACK.
    assert_eq!(&pack[0..4], b"PACK");

    // (b) header object count equals ONLY the new objects introduced by C2
    // (new commit + new tree + new blob = 3) — proving minimal selection. The
    // a.txt blob and C1's objects, reachable from the have, are excluded.
    let count = pack_header_count(&pack);
    assert_eq!(
        count,
        fx.c2_new_objects.len() as u32,
        "expected exactly the 3 objects new in C2, got {count}"
    );

    // (c) structurally valid: re-parse with grit-lib's pack reader (this also
    // verifies the trailing checksum) and confirm the resolved object set is
    // exactly the new objects.
    let map = pack_bytes_to_object_map(&pack, &odb).expect("re-parse pack");
    assert_eq!(map.len(), fx.c2_new_objects.len());
    for oid in &fx.c2_new_objects {
        assert!(map.contains_key(oid), "pack missing new object {oid}");
    }
    for oid in &fx.c1_objects {
        assert!(
            !map.contains_key(oid),
            "pack should not contain have-reachable object {oid}"
        );
    }
}

#[test]
fn build_pack_with_empty_haves_packs_full_closure() {
    let fx = build_fixture();
    let odb = open_odb(fx.dir.path());

    let pack = build_pack(&odb, &[fx.c2], &[], &PackBuildOptions::default()).expect("build");

    assert_eq!(&pack[0..4], b"PACK");

    // Full closure from C2: C1, C2, C1's tree, C2's tree, a.txt blob, b.txt blob
    // = 6 distinct objects.
    let map = pack_bytes_to_object_map(&pack, &odb).expect("re-parse pack");
    assert_eq!(
        pack_header_count(&pack) as usize,
        map.len(),
        "header count must match resolved object count"
    );
    assert_eq!(map.len(), 6, "full closure of C2 is 6 objects, got {}", map.len());

    // Every object from both commits must be present.
    for oid in fx.c1_objects.iter().chain(fx.c2_new_objects.iter()) {
        assert!(map.contains_key(oid), "full pack missing {oid}");
    }
    assert!(map.contains_key(&fx.c1));
    assert!(map.contains_key(&fx.c2));
}
