//! Embedder-facing transfer (fetch / push) result & option types, plus the
//! negotiation-driven pack builder.
//!
//! This module is the foundation for the in-process fetch/push APIs that
//! embedders such as `jj` and GitButler consume in place of `gix` transport.
//! It defines the structured input/output types those APIs use and implements
//! the single most important primitive — [`build_pack`] — which packs **only**
//! the objects reachable from a negotiated set of `wants` and not already
//! reachable from the remote's `haves`.
//!
//! Scope note (phase 1): only the local / `file://` object+ref copy path is in
//! scope. `git://`, `http(s)`, and `ssh` transports plus credential-helper
//! execution are out of scope and are left as TODOs in later phases.
//!
//! Push *result* reporting reuses [`crate::push_report::PushRefResult`] /
//! [`crate::push_report::PushRefStatus`] rather than redefining it.

use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::path::Path;

use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest as _, Sha1};
use sha2::Sha256;

use crate::error::{Error, Result};
use crate::objects::{
    parse_commit, parse_tag, parse_tree, HashAlgo, ObjectId, ObjectKind,
};
use crate::odb::Odb;
use crate::push_report::{PushRefResult, PushRefStatus};
use crate::refspec::{parse_fetch_refspec, RefspecItem};

/// How a single reference resolved during a fetch (or would resolve in a push).
///
/// Mirrors the shapes of `gix::remote::fetch::refs::update::Mode` that `jj`
/// already consumes, so the embedder's translation layer stays a thin adapter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpdateMode {
    /// The local tracking ref did not exist and was created.
    New,
    /// The update advanced the ref along its existing history.
    FastForward,
    /// A non-fast-forward update that was applied because force was requested.
    Forced,
    /// The local ref already matched the remote value; nothing to do.
    UpToDate,
    /// No change was required (e.g. a no-op refspec).
    NoChangeNeeded,
    /// A non-fast-forward update that was rejected (force not requested).
    NonFastForwardRejected,
    /// A tag update was rejected (tags are not overwritten without force).
    TagUpdateRejected,
    /// The source object named by the refspec was not found on the remote.
    SourceObjectNotFound,
    /// The remote ref is unborn (points at nothing yet).
    Unborn,
    /// A prune/delete was requested but the local ref was already missing.
    DeletedMissing,
}

/// The resolved outcome of one reference during a fetch.
#[derive(Clone, Debug)]
pub struct RefUpdate {
    /// The remote-side ref name (e.g. `refs/heads/main`).
    pub remote_ref: String,
    /// The local-side ref name written, if any (e.g. `refs/remotes/origin/main`).
    pub local_ref: Option<String>,
    /// Previous value of the local ref (`None` when newly created).
    pub old_oid: Option<ObjectId>,
    /// New value written to the local ref (`None` for deletions / unborn).
    pub new_oid: Option<ObjectId>,
    /// How the update resolved.
    pub mode: UpdateMode,
    /// Optional human-readable note (reason text), for embedder display.
    pub note: Option<String>,
}

/// Which tags to fetch alongside the requested refs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum TagMode {
    /// Do not fetch any tags automatically.
    None,
    /// Fetch tags that point at objects being fetched (Git's default).
    #[default]
    Following,
    /// Fetch all tags from the remote.
    All,
}

/// Options controlling a fetch.
#[derive(Clone, Debug)]
pub struct FetchOptions {
    /// Positive refspecs selecting what to fetch.
    pub refspecs: Vec<String>,
    /// Negative refspecs excluding refs from the positive set.
    pub negative_refspecs: Vec<String>,
    /// Tag-following policy.
    pub tags: TagMode,
    /// Whether to prune local tracking refs that vanished on the remote.
    pub prune: bool,
    /// Compute and report updates without writing any refs or objects.
    pub dry_run: bool,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            refspecs: Vec::new(),
            negative_refspecs: Vec::new(),
            tags: TagMode::default(),
            prune: false,
            dry_run: false,
        }
    }
}

/// The structured result of a fetch, ready for the embedder's ref-store apply.
#[derive(Clone, Debug, Default)]
pub struct FetchOutcome {
    /// Per-ref resolved updates.
    pub updates: Vec<RefUpdate>,
    /// The remote's default branch (from `HEAD` symref), if known.
    pub default_branch: Option<String>,
}

/// A single ref update requested by a push.
#[derive(Clone, Debug)]
pub struct PushRefSpec {
    /// The source object to push (`None` for a deletion).
    pub src: Option<ObjectId>,
    /// The destination ref on the remote (e.g. `refs/heads/main`).
    pub dst: String,
    /// Whether a non-fast-forward update is allowed.
    pub force: bool,
    /// Whether this update deletes the remote ref.
    pub delete: bool,
    /// Compare-and-swap expectation: the remote ref's current value must match
    /// this (force-with-lease). `None` disables the value check.
    pub expected_old: Option<ObjectId>,
    /// Force-with-lease expectation that the remote ref does **not** currently
    /// exist. When `true`, a push whose destination already exists on the remote
    /// is rejected as stale (used for "create only" pushes whose lease is the
    /// ref's absence). Independent of [`Self::expected_old`].
    pub expect_absent: bool,
}

/// Options controlling a push.
#[derive(Clone, Debug, Default)]
pub struct PushOptions {
    /// Apply all updates atomically (all-or-nothing).
    pub atomic: bool,
    /// Compute results without writing to the remote.
    pub dry_run: bool,
}

/// The structured result of a push. Reuses [`PushRefResult`] for per-ref status.
#[derive(Clone, Debug, Default)]
pub struct PushOutcome {
    /// Per-ref resolved results (status, old/new oid, reason).
    pub results: Vec<PushRefResult>,
}

/// Options controlling [`build_pack`].
#[derive(Clone, Copy, Debug)]
pub struct PackBuildOptions {
    /// Build a thin pack (allow deltas against bases not present in the pack).
    ///
    /// Not yet honored in phase 1 — the builder always emits whole (non-delta)
    /// objects. Reserved so the signature is stable for later phases.
    pub thin: bool,
}

impl Default for PackBuildOptions {
    fn default() -> Self {
        Self { thin: false }
    }
}

/// Build a v2 packfile containing exactly the objects reachable from `wants`
/// but **not** reachable from `haves`, de-duplicated.
///
/// This is the negotiation-driven object selection that lets embedders avoid
/// packing the entire reachable closure of a pushed tip (the 478 MB regression
/// the spike hit). The walk is a BFS over commit parents and tree entries:
///
/// 1. Compute the object closure of `haves` (commits, their trees recursively,
///    blobs, and annotated-tag targets). Descent stops at any object already in
///    that closure.
/// 2. Walk `wants` the same way, skipping any object in the `haves` closure, and
///    collect every newly-reachable object.
/// 3. Serialize the collected objects as whole (non-delta) entries into a valid
///    PACK v2 stream, with the trailing checksum at the repository's hash width.
///
/// The produced bytes start with `PACK`, carry the exact object count, and
/// re-parse cleanly with [`crate::pack::read_object_from_pack_bytes`].
///
/// # Errors
///
/// Returns an error if a required object is missing from `odb`, if an object
/// fails to parse, or if the repository hash width is unsupported.
pub fn build_pack(
    odb: &Odb,
    wants: &[ObjectId],
    haves: &[ObjectId],
    _opts: &PackBuildOptions,
) -> Result<Vec<u8>> {
    // TODO(phase: thin/delta packs): honor `_opts.thin` and emit OFS/REF deltas.
    // Phase 1 emits whole objects only — correct and minimal in object count,
    // not byte-optimal.

    // Objects already reachable from the remote's haves: never repack these, and
    // stop descent into them. A `have` that is not present in this odb (e.g. a
    // local-only commit named by a local tracking ref) simply prunes nothing, so
    // missing haves are tolerated rather than erroring.
    let have_closure = reachable_closure(odb, haves, &HashSet::new(), true)?;

    // Objects reachable from wants but not from haves, in discovery order. A
    // missing want IS an error (we were asked to pack an object we don't have).
    let send = collect_reachable_excluding(odb, wants, &have_closure, false)?;

    serialize_pack(odb, &send)
}

/// Compute the full object closure reachable from `roots`, stopping descent into
/// any object already present in `stop`.
fn reachable_closure(
    odb: &Odb,
    roots: &[ObjectId],
    stop: &HashSet<ObjectId>,
    skip_missing: bool,
) -> Result<HashSet<ObjectId>> {
    let mut seen = HashSet::new();
    let order = collect_reachable_excluding(odb, roots, stop, skip_missing)?;
    for oid in order {
        seen.insert(oid);
    }
    Ok(seen)
}

/// BFS over `roots` collecting every reachable object (commits, trees, blobs,
/// tag targets) that is not in `exclude`, returned in discovery order with no
/// duplicates.
///
/// Discovery order keeps commits before the trees/blobs they introduce, which
/// is a reasonable, deterministic pack ordering.
fn collect_reachable_excluding(
    odb: &Odb,
    roots: &[ObjectId],
    exclude: &HashSet<ObjectId>,
    skip_missing: bool,
) -> Result<Vec<ObjectId>> {
    let mut visited: HashSet<ObjectId> = HashSet::new();
    let mut ordered: Vec<ObjectId> = Vec::new();
    let mut queue: VecDeque<ObjectId> = VecDeque::new();

    let enqueue = |oid: ObjectId,
                       queue: &mut VecDeque<ObjectId>,
                       visited: &mut HashSet<ObjectId>,
                       ordered: &mut Vec<ObjectId>|
     -> bool {
        if exclude.contains(&oid) {
            return false;
        }
        if visited.insert(oid) {
            ordered.push(oid);
            queue.push_back(oid);
            true
        } else {
            false
        }
    };

    for &root in roots {
        enqueue(root, &mut queue, &mut visited, &mut ordered);
    }

    while let Some(oid) = queue.pop_front() {
        let obj = match odb.read(&oid) {
            Ok(o) => o,
            // A root/have absent from this odb cannot be traversed; with
            // `skip_missing` it simply contributes nothing (no descent), instead
            // of failing the whole pack build.
            Err(_) if skip_missing => continue,
            Err(e) => return Err(e),
        };
        match obj.kind {
            ObjectKind::Commit => {
                let commit = parse_commit(&obj.data)?;
                for parent in commit.parents {
                    enqueue(parent, &mut queue, &mut visited, &mut ordered);
                }
                enqueue(commit.tree, &mut queue, &mut visited, &mut ordered);
            }
            ObjectKind::Tree => {
                for entry in parse_tree(&obj.data)? {
                    // Skip submodule (gitlink) entries: the commit they name
                    // lives in another object store and is not part of this pack.
                    if entry.mode == 0o160000 {
                        continue;
                    }
                    enqueue(entry.oid, &mut queue, &mut visited, &mut ordered);
                }
            }
            ObjectKind::Tag => {
                let tag = parse_tag(&obj.data)?;
                enqueue(tag.object, &mut queue, &mut visited, &mut ordered);
            }
            ObjectKind::Blob => {}
        }
    }

    Ok(ordered)
}

/// The pack object type code for a Git object kind (PACK v2 base types).
fn pack_type_code(kind: ObjectKind) -> u8 {
    match kind {
        ObjectKind::Commit => 1,
        ObjectKind::Tree => 2,
        ObjectKind::Blob => 3,
        ObjectKind::Tag => 4,
    }
}

/// Append a PACK object header: 3-bit type + variable-length size (little-endian
/// 7-bit groups, MSB = continuation). Lifted from the CLI pack writer.
fn encode_pack_object_header(buf: &mut Vec<u8>, type_code: u8, payload_len: usize) {
    let mut size = payload_len;
    let first = ((type_code & 0x7) << 4) | (size & 0x0f) as u8;
    size >>= 4;
    if size > 0 {
        buf.push(first | 0x80);
        while size > 0 {
            let b = (size & 0x7f) as u8;
            size >>= 7;
            buf.push(if size > 0 { b | 0x80 } else { b });
        }
    } else {
        buf.push(first);
    }
}

/// Serialize `oids` as a PACK v2 stream of whole (non-delta) objects, terminated
/// by the trailing pack checksum at the repository hash width.
fn serialize_pack(odb: &Odb, oids: &[ObjectId]) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"PACK");
    buf.extend_from_slice(&2u32.to_be_bytes());
    let count = u32::try_from(oids.len())
        .map_err(|_| Error::CorruptObject("pack object count exceeds u32".to_owned()))?;
    buf.extend_from_slice(&count.to_be_bytes());

    for oid in oids {
        let obj = odb.read(oid)?;
        encode_pack_object_header(&mut buf, pack_type_code(obj.kind), obj.data.len());
        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(&obj.data).map_err(Error::Io)?;
        let compressed = enc.finish().map_err(Error::Io)?;
        buf.extend_from_slice(&compressed);
    }

    append_pack_trailer(&mut buf, odb.hash_algo());
    Ok(buf)
}

/// Append the trailing pack checksum: the hash of everything written so far, at
/// the repository's hash width (SHA-1 → 20 bytes, SHA-256 → 32 bytes).
fn append_pack_trailer(buf: &mut Vec<u8>, algo: HashAlgo) {
    match algo {
        HashAlgo::Sha1 => {
            let mut hasher = Sha1::new();
            hasher.update(&*buf);
            buf.extend_from_slice(&hasher.finalize());
        }
        HashAlgo::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(&*buf);
            buf.extend_from_slice(&hasher.finalize());
        }
    }
}

/// Fetch refs and objects from one on-disk git repository into another, entirely
/// in-process (no subprocess, no wire protocol).
///
/// This is the local / `file://` fetch path. It:
///
/// 1. Enumerates the remote's refs with [`crate::ls_remote::ls_remote`] and
///    captures the remote `HEAD` symref for [`FetchOutcome::default_branch`].
/// 2. Parses `opts.refspecs` with [`crate::refspec`] and, for each remote ref
///    that matches a positive refspec (and is not excluded by a negative one),
///    computes the destination local tracking ref and the wanted remote oid.
///    [`TagMode`] adds tags: `All` brings every `refs/tags/*`, `Following` brings
///    tags pointing at objects already being fetched, `None` skips `refs/tags/*`.
/// 3. Copies the minimal set of objects (reachable from the wanted oids, stopping
///    at objects already present locally) from the remote odb into the local odb
///    via [`build_pack`] + [`crate::unpack_objects::unpack_objects`].
/// 4. Classifies each ref update as [`UpdateMode`] (`New` / `UpToDate` /
///    `FastForward` / `Forced` / `NonFastForwardRejected`) using local ancestry,
///    and — unless `opts.dry_run` — writes the local tracking ref.
/// 5. When `opts.prune` is set, deletes local tracking refs whose remote
///    counterpart is gone, recording them as [`UpdateMode::DeletedMissing`].
///
/// Both repositories must use the same object hash algorithm; the hash width is
/// threaded through [`Odb::hash_algo`] so SHA-256 repos work.
///
/// # Errors
///
/// Returns an error if either repository cannot be opened, if a refspec is
/// invalid, if a required remote object cannot be read, or on I/O failure while
/// writing objects or refs.
//
// TODO(phase: remote transports): `git://`, `http(s)`, and `ssh` fetch (wire
// protocol handshake + negotiation + credential helpers) are out of scope here
// and live in a later phase.
pub fn fetch_local(
    local_git_dir: &Path,
    remote_git_dir: &Path,
    opts: &FetchOptions,
) -> Result<FetchOutcome> {
    let local_odb = open_odb(local_git_dir);
    let remote_odb = open_odb(remote_git_dir);

    // 1. Enumerate remote refs (with HEAD symref for the default branch).
    let remote_entries = crate::ls_remote::ls_remote(
        remote_git_dir,
        &remote_odb,
        &crate::ls_remote::Options {
            symref: true,
            ..Default::default()
        },
    )?;

    let mut default_branch = None;
    // remote ref name -> oid (excluding HEAD and peeled `^{}` entries).
    let mut remote_refs: Vec<(String, ObjectId)> = Vec::new();
    for entry in &remote_entries {
        if entry.name == "HEAD" {
            default_branch = entry
                .symref_target
                .as_ref()
                .map(|t| t.strip_prefix("refs/heads/").unwrap_or(t).to_owned());
            continue;
        }
        if entry.name.ends_with("^{}") {
            continue;
        }
        remote_refs.push((entry.name.clone(), entry.oid));
    }

    // 2. Parse refspecs.
    let mut positive: Vec<RefspecItem> = Vec::new();
    let mut negatives: Vec<RefspecItem> = Vec::new();
    for spec in &opts.refspecs {
        let item = parse_fetch_refspec(spec)
            .map_err(|e| Error::Message(format!("invalid refspec '{spec}': {e}")))?;
        if item.negative {
            negatives.push(item);
        } else {
            positive.push(item);
        }
    }
    for spec in &opts.negative_refspecs {
        let item = parse_fetch_refspec(spec)
            .map_err(|e| Error::Message(format!("invalid negative refspec '{spec}': {e}")))?;
        negatives.push(item);
    }

    // Compute the matched (remote_ref, local_ref, wanted_oid, force) set.
    // `local_ref == None` means "fetch but do not store" (empty dst).
    let mut matched: Vec<MatchedRef> = Vec::new();
    let mut matched_oids: HashSet<ObjectId> = HashSet::new();
    let mut seen_remote_ref: HashSet<String> = HashSet::new();

    for (name, oid) in &remote_refs {
        if name.starts_with("refs/tags/") {
            // Tags are governed by TagMode below, not the head refspecs, unless
            // a refspec explicitly names them. Still allow an explicit refspec
            // match here; TagMode adds the rest.
        }
        if ref_excluded(name, &negatives) {
            continue;
        }
        if let Some(local_ref) = match_positive(name, &positive) {
            if seen_remote_ref.insert(name.clone()) {
                matched_oids.insert(*oid);
                matched.push(MatchedRef {
                    remote_ref: name.clone(),
                    local_ref,
                    oid: *oid,
                    force: refspecs_force(name, &positive),
                    is_tag: name.starts_with("refs/tags/"),
                });
            }
        }
    }

    // TagMode: add tags. We need the closure of objects already being fetched to
    // decide "Following".
    apply_tag_mode(
        opts.tags,
        &remote_refs,
        &remote_odb,
        &negatives,
        &mut matched,
        &mut matched_oids,
        &mut seen_remote_ref,
    )?;

    // 3. Determine wants (matched oids not present locally) and haves (current
    //    local tracking-ref tips) and copy the minimal object set.
    let wants: Vec<ObjectId> = matched_oids
        .iter()
        .copied()
        .filter(|oid| !local_odb.exists(oid))
        .collect();

    let mut haves: Vec<ObjectId> = Vec::new();
    let mut have_seen: HashSet<ObjectId> = HashSet::new();
    for m in &matched {
        if let Some(local_ref) = &m.local_ref {
            if let Ok(old) = crate::refs::resolve_ref(local_git_dir, local_ref) {
                if have_seen.insert(old) {
                    haves.push(old);
                }
            }
        }
    }

    if !wants.is_empty() && !opts.dry_run {
        let pack = build_pack(&remote_odb, &wants, &haves, &PackBuildOptions::default())?;
        let mut cursor = std::io::Cursor::new(pack);
        crate::unpack_objects::unpack_objects(
            &mut cursor,
            &local_odb,
            &crate::unpack_objects::UnpackOptions {
                quiet: true,
                ..Default::default()
            },
        )?;
    }

    // 4. Classify and apply ref updates. Ancestry checks use the local repo,
    //    which now contains the fetched objects.
    let local_repo = if opts.dry_run {
        None
    } else {
        crate::repo::Repository::open(local_git_dir, None).ok()
    };

    let mut updates: Vec<RefUpdate> = Vec::new();
    for m in &matched {
        let Some(local_ref) = &m.local_ref else {
            // dst empty: fetched but not stored. Report as a no-store update.
            updates.push(RefUpdate {
                remote_ref: m.remote_ref.clone(),
                local_ref: None,
                old_oid: None,
                new_oid: Some(m.oid),
                mode: UpdateMode::NoChangeNeeded,
                note: Some("not stored (empty destination)".to_owned()),
            });
            continue;
        };

        let old = crate::refs::resolve_ref(local_git_dir, local_ref).ok();
        let mode = classify_update(
            old.as_ref(),
            &m.oid,
            m.force,
            m.is_tag,
            local_repo.as_ref(),
        );

        let write = matches!(
            mode,
            UpdateMode::New | UpdateMode::FastForward | UpdateMode::Forced
        );
        if write && !opts.dry_run {
            crate::refs::write_ref(local_git_dir, local_ref, &m.oid)?;
        }

        updates.push(RefUpdate {
            remote_ref: m.remote_ref.clone(),
            local_ref: Some(local_ref.clone()),
            old_oid: old,
            new_oid: Some(m.oid),
            mode,
            note: None,
        });
    }

    // 5. Prune: delete local tracking refs whose remote counterpart is gone.
    if opts.prune {
        prune_tracking_refs(
            local_git_dir,
            &positive,
            &remote_refs,
            opts.dry_run,
            &mut updates,
        )?;
    }

    Ok(FetchOutcome {
        updates,
        default_branch,
    })
}

/// Push refs and objects from one on-disk git repository into another, entirely
/// in-process (no subprocess, no wire protocol).
///
/// This is the local / `file://` push (send-pack) counterpart to
/// [`fetch_local`]. For each [`PushRefSpec`] it:
///
/// 1. Resolves the source oid from the LOCAL repo (for a non-delete update) and
///    reads the remote's current value of `dst`.
/// 2. Enforces the update rules and produces a [`PushRefResult`] with the right
///    [`crate::push_report::PushRefStatus`]:
///    * `expected_old` set and mismatching the remote's current value →
///      [`PushRefStatus::RejectStale`] (compare-and-swap / force-with-lease).
///    * deletion → succeed when present, or [`PushRefStatus::UpToDate`] when the
///      ref is already gone.
///    * non-fast-forward (remote current is not an ancestor of the source)
///      without `force` → [`PushRefStatus::RejectNonFastForward`]; with `force`
///      it is accepted and reported as forced.
///    * unchanged (remote already at the source) → [`PushRefStatus::UpToDate`].
///    * otherwise [`PushRefStatus::Ok`].
/// 3. For accepted non-delete updates, copies the minimal object closure from the
///    LOCAL odb into the REMOTE odb via [`build_pack`] +
///    [`crate::unpack_objects::unpack_objects`], excluding objects already
///    reachable from the remote's existing ref tips.
/// 4. Applies the ref change on the remote (unless `opts.dry_run`).
///
/// When `opts.atomic` is set and any ref is rejected, no ref or object is
/// written and every otherwise-accepted ref is reported as
/// [`PushRefStatus::AtomicPushFailed`].
///
/// Both repositories must use the same object hash algorithm; the hash width is
/// threaded through [`Odb::hash_algo`] so SHA-256 repos work.
///
/// # Errors
///
/// Returns an error if either repository cannot be opened, if a source object is
/// missing from the local odb, or on I/O failure while writing objects or refs.
//
// TODO(phase: remote transports): `git://`, `http(s)`, and `ssh` push
// (receive-pack handshake + report-status parsing + credential helpers) are out
// of scope here and live in a later phase.
pub fn push_local(
    local_git_dir: &Path,
    remote_git_dir: &Path,
    refs: &[PushRefSpec],
    opts: &PushOptions,
) -> Result<PushOutcome> {
    let local_odb = open_odb(local_git_dir);
    let remote_odb = open_odb(remote_git_dir);

    // Ancestry (fast-forward) checks run against the LOCAL repo, where the source
    // commits live. A remote-current oid that is not reachable from the source is
    // simply "not an ancestor", which is the correct non-fast-forward verdict.
    let local_repo = crate::repo::Repository::open(local_git_dir, None).ok();

    // The remote's existing ref tips become the `haves` for pack building, so the
    // copied object closure excludes everything the remote already has.
    let remote_have_tips: Vec<ObjectId> = crate::refs::list_refs(remote_git_dir, "refs/")?
        .into_iter()
        .map(|(_, oid)| oid)
        .collect();

    // First pass: decide each ref's status without mutating anything.
    let mut decisions: Vec<PushDecision> = Vec::with_capacity(refs.len());
    for spec in refs {
        decisions.push(decide_push(
            spec,
            &local_odb,
            remote_git_dir,
            local_repo.as_ref(),
        )?);
    }

    // Atomic: if any update would be rejected, apply none and demote the
    // otherwise-accepted updates to AtomicPushFailed.
    let any_rejected = decisions.iter().any(|d| d.result.status.is_error());
    if opts.atomic && any_rejected {
        for d in &mut decisions {
            if matches!(d.result.status, PushRefStatus::Ok) {
                d.result.status = PushRefStatus::AtomicPushFailed;
                d.apply = false;
            }
        }
        return Ok(PushOutcome {
            results: decisions.into_iter().map(|d| d.result).collect(),
        });
    }

    // Second pass: apply accepted updates (copy objects, then move/delete refs).
    for d in &mut decisions {
        if !d.apply || opts.dry_run {
            continue;
        }
        match &d.action {
            PushAction::Update(src) => {
                let pack = build_pack(
                    &local_odb,
                    &[*src],
                    &remote_have_tips,
                    &PackBuildOptions::default(),
                )?;
                let mut cursor = std::io::Cursor::new(pack);
                crate::unpack_objects::unpack_objects(
                    &mut cursor,
                    &remote_odb,
                    &crate::unpack_objects::UnpackOptions {
                        quiet: true,
                        ..Default::default()
                    },
                )?;
                crate::refs::write_ref(remote_git_dir, &d.result.remote_ref, src)?;
            }
            PushAction::Delete => {
                crate::refs::delete_ref(remote_git_dir, &d.result.remote_ref)?;
            }
            PushAction::None => {}
        }
    }

    Ok(PushOutcome {
        results: decisions.into_iter().map(|d| d.result).collect(),
    })
}

/// What a single accepted push update does once applied.
enum PushAction {
    /// Copy the closure of `src` to the remote and move the ref to `src`.
    Update(ObjectId),
    /// Delete the remote ref.
    Delete,
    /// No mutation (up-to-date or rejected).
    None,
}

/// A decided-but-not-yet-applied push update.
struct PushDecision {
    result: PushRefResult,
    action: PushAction,
    /// Whether the second pass should apply `action`.
    apply: bool,
}

/// Decide the status of a single [`PushRefSpec`] without mutating either repo.
fn decide_push(
    spec: &PushRefSpec,
    local_odb: &Odb,
    remote_git_dir: &Path,
    local_repo: Option<&crate::repo::Repository>,
) -> Result<PushDecision> {
    let remote_current = crate::refs::resolve_ref(remote_git_dir, &spec.dst).ok();

    // Absence lease (force-with-lease that the ref not exist): a push that
    // expected the destination to be absent is rejected when it is present, even
    // if it happens to already point at the source (Git enforces the lease
    // strictly). Checked before the up-to-date shortcut below.
    if spec.expect_absent && remote_current.is_some() {
        return Ok(PushDecision {
            result: PushRefResult {
                local_ref: None,
                remote_ref: spec.dst.clone(),
                old_oid: remote_current,
                new_oid: spec.src,
                forced: false,
                deletion: spec.delete,
                status: PushRefStatus::RejectStale,
                message: Some("stale info".to_owned()),
            },
            action: PushAction::None,
            apply: false,
        });
    }

    // Up-to-date trumps the lease: pushing a non-delete to where the remote ref
    // already points is a no-op that succeeds even when `expected_old` (the
    // force-with-lease expectation) differs ("moving a bookmark to the same place
    // it already is is OK"). This check must precede the compare-and-swap check.
    if !spec.delete {
        if let Some(src) = spec.src {
            if remote_current == Some(src) {
                return Ok(PushDecision {
                    result: PushRefResult {
                        local_ref: None,
                        remote_ref: spec.dst.clone(),
                        old_oid: remote_current,
                        new_oid: Some(src),
                        forced: false,
                        deletion: false,
                        status: PushRefStatus::UpToDate,
                        message: None,
                    },
                    action: PushAction::None,
                    apply: false,
                });
            }
        }
    }

    // Compare-and-swap (force-with-lease): the remote's current value must match
    // the caller's expectation, otherwise reject as stale. A `None` expectation
    // disables the check. An expectation that the ref be absent is honored too.
    if let Some(expected) = spec.expected_old {
        if remote_current != Some(expected) {
            return Ok(PushDecision {
                result: PushRefResult {
                    local_ref: None,
                    remote_ref: spec.dst.clone(),
                    old_oid: remote_current,
                    new_oid: spec.src,
                    forced: false,
                    deletion: spec.delete,
                    status: PushRefStatus::RejectStale,
                    message: Some("stale info".to_owned()),
                },
                action: PushAction::None,
                apply: false,
            });
        }
    }

    if spec.delete {
        let (status, action, apply) = match remote_current {
            Some(_) => (PushRefStatus::Ok, PushAction::Delete, true),
            None => (PushRefStatus::UpToDate, PushAction::None, false),
        };
        return Ok(PushDecision {
            result: PushRefResult {
                local_ref: None,
                remote_ref: spec.dst.clone(),
                old_oid: remote_current,
                new_oid: None,
                forced: false,
                deletion: true,
                status,
                message: None,
            },
            action,
            apply,
        });
    }

    // Non-delete updates require a source object that exists locally.
    let Some(src) = spec.src else {
        return Err(Error::Message(format!(
            "push to '{}' has no source object and is not a deletion",
            spec.dst
        )));
    };
    if !local_odb.exists(&src) {
        return Err(Error::Message(format!(
            "source object {src} for '{}' is missing from the local object store",
            spec.dst
        )));
    }

    // Unchanged: the remote is already at the source.
    if remote_current == Some(src) {
        return Ok(PushDecision {
            result: PushRefResult {
                local_ref: None,
                remote_ref: spec.dst.clone(),
                old_oid: remote_current,
                new_oid: Some(src),
                forced: false,
                deletion: false,
                status: PushRefStatus::UpToDate,
                message: None,
            },
            action: PushAction::None,
            apply: false,
        });
    }

    // New ref: nothing on the remote yet — always allowed.
    let Some(old) = remote_current else {
        return Ok(PushDecision {
            result: PushRefResult {
                local_ref: None,
                remote_ref: spec.dst.clone(),
                old_oid: None,
                new_oid: Some(src),
                forced: false,
                deletion: false,
                status: PushRefStatus::Ok,
                message: None,
            },
            action: PushAction::Update(src),
            apply: true,
        });
    };

    // Existing ref: fast-forward when the remote's current commit is an ancestor
    // of the source. Otherwise it is a non-fast-forward update, allowed only with
    // force (reported as forced).
    let is_ff = local_repo
        .map(|r| crate::merge_base::is_ancestor(r, old, src).unwrap_or(false))
        .unwrap_or(false);

    if is_ff {
        Ok(PushDecision {
            result: PushRefResult {
                local_ref: None,
                remote_ref: spec.dst.clone(),
                old_oid: Some(old),
                new_oid: Some(src),
                forced: false,
                deletion: false,
                status: PushRefStatus::Ok,
                message: None,
            },
            action: PushAction::Update(src),
            apply: true,
        })
    } else if spec.force {
        Ok(PushDecision {
            result: PushRefResult {
                local_ref: None,
                remote_ref: spec.dst.clone(),
                old_oid: Some(old),
                new_oid: Some(src),
                forced: true,
                deletion: false,
                status: PushRefStatus::Ok,
                message: None,
            },
            action: PushAction::Update(src),
            apply: true,
        })
    } else {
        Ok(PushDecision {
            result: PushRefResult {
                local_ref: None,
                remote_ref: spec.dst.clone(),
                old_oid: Some(old),
                new_oid: Some(src),
                forced: false,
                deletion: false,
                status: PushRefStatus::RejectNonFastForward,
                message: Some("non-fast-forward".to_owned()),
            },
            action: PushAction::None,
            apply: false,
        })
    }
}

/// A remote ref selected for fetch, with its computed local destination.
struct MatchedRef {
    remote_ref: String,
    /// Destination local tracking ref, or `None` for an empty (no-store) dst.
    local_ref: Option<String>,
    oid: ObjectId,
    force: bool,
    is_tag: bool,
}

/// Open an [`Odb`] for a git directory, attaching the git dir so `hash_algo`
/// (and MIDX config) resolve correctly.
fn open_odb(git_dir: &Path) -> Odb {
    Odb::new(&git_dir.join("objects")).with_config_git_dir(git_dir.to_path_buf())
}

/// Match a ref name against the positive refspecs, returning the destination
/// local ref name (`Some(name)`), `None`+stored=false collapsed: returns
/// `Some(Some(dst))` to store, `Some(None)` to fetch-without-store, or `None`
/// when no positive refspec matches.
fn match_positive(refname: &str, positive: &[RefspecItem]) -> Option<Option<String>> {
    for item in positive {
        let Some(src) = item.src.as_deref() else {
            continue;
        };
        if let Some(dst) = apply_refspec(src, item.dst.as_deref(), refname) {
            // Empty dst means "fetch but do not store".
            if dst.is_empty() {
                return Some(None);
            }
            return Some(Some(dst));
        }
    }
    None
}

/// Whether any positive refspec matching `refname` requested force (`+`).
fn refspecs_force(refname: &str, positive: &[RefspecItem]) -> bool {
    positive.iter().any(|item| {
        item.force
            && item
                .src
                .as_deref()
                .is_some_and(|src| apply_refspec(src, item.dst.as_deref(), refname).is_some())
    })
}

/// Whether `refname` is excluded by any negative refspec.
fn ref_excluded(refname: &str, negatives: &[RefspecItem]) -> bool {
    negatives.iter().any(|item| {
        item.src
            .as_deref()
            .is_some_and(|src| glob_matches(src, refname))
    })
}

/// Apply a `<src>[:<dst>]` refspec to `refname`, returning the destination ref.
///
/// Supports a single `*` wildcard (Git's fetch refspec form). When `dst` is
/// `None` the destination equals the matched source (rare for tracking fetches);
/// when `dst` is `Some("")` the empty string is returned (fetch-without-store).
fn apply_refspec(src: &str, dst: Option<&str>, refname: &str) -> Option<String> {
    match src.find('*') {
        Some(star) => {
            let prefix = &src[..star];
            let suffix = &src[star + 1..];
            if !refname.starts_with(prefix)
                || !refname.ends_with(suffix)
                || refname.len() < prefix.len() + suffix.len()
            {
                return None;
            }
            let middle = &refname[prefix.len()..refname.len() - suffix.len()];
            match dst {
                None => Some(refname.to_owned()),
                Some("") => Some(String::new()),
                Some(d) => Some(d.replacen('*', middle, 1)),
            }
        }
        None => {
            if src != refname {
                return None;
            }
            match dst {
                None => Some(refname.to_owned()),
                Some("") => Some(String::new()),
                Some(d) => Some(d.to_owned()),
            }
        }
    }
}

/// Whether `pattern` (a refspec src side, possibly with one `*`) matches `refname`.
fn glob_matches(pattern: &str, refname: &str) -> bool {
    match pattern.find('*') {
        Some(star) => {
            let prefix = &pattern[..star];
            let suffix = &pattern[star + 1..];
            refname.starts_with(prefix)
                && refname.ends_with(suffix)
                && refname.len() >= prefix.len() + suffix.len()
        }
        None => pattern == refname,
    }
}

/// Add tags to the matched set according to [`TagMode`].
#[allow(clippy::too_many_arguments)]
fn apply_tag_mode(
    mode: TagMode,
    remote_refs: &[(String, ObjectId)],
    remote_odb: &Odb,
    negatives: &[RefspecItem],
    matched: &mut Vec<MatchedRef>,
    matched_oids: &mut HashSet<ObjectId>,
    seen_remote_ref: &mut HashSet<String>,
) -> Result<()> {
    if mode == TagMode::None {
        return Ok(());
    }

    // For Following we need the set of objects reachable from the already-matched
    // (non-tag) refs, so we can keep tags pointing into that closure.
    let following_closure: HashSet<ObjectId> = if mode == TagMode::Following {
        let roots: Vec<ObjectId> = matched.iter().map(|m| m.oid).collect();
        reachable_closure(remote_odb, &roots, &HashSet::new(), true)?
    } else {
        HashSet::new()
    };

    for (name, oid) in remote_refs {
        if !name.starts_with("refs/tags/") {
            continue;
        }
        if seen_remote_ref.contains(name) || ref_excluded(name, negatives) {
            continue;
        }
        let keep = match mode {
            TagMode::All => true,
            TagMode::Following => {
                // Keep when the tag (or what it peels to) is in the fetched
                // closure. Peel annotated tags to their target.
                let peeled = peel_tag_target(remote_odb, *oid)?;
                following_closure.contains(oid) || following_closure.contains(&peeled)
            }
            TagMode::None => false,
        };
        if keep {
            seen_remote_ref.insert(name.clone());
            matched_oids.insert(*oid);
            matched.push(MatchedRef {
                remote_ref: name.clone(),
                local_ref: Some(name.clone()),
                oid: *oid,
                force: false,
                is_tag: true,
            });
        }
    }
    Ok(())
}

/// Peel an (annotated) tag to the non-tag object it ultimately points at.
/// Returns the input oid unchanged for non-tag objects or on read failure.
fn peel_tag_target(odb: &Odb, oid: ObjectId) -> Result<ObjectId> {
    let mut current = oid;
    for _ in 0..16 {
        let obj = match odb.read(&current) {
            Ok(o) => o,
            Err(_) => return Ok(current),
        };
        if obj.kind != ObjectKind::Tag {
            return Ok(current);
        }
        current = parse_tag(&obj.data)?.object;
    }
    Ok(current)
}

/// Classify a single ref update into an [`UpdateMode`].
fn classify_update(
    old: Option<&ObjectId>,
    new: &ObjectId,
    force: bool,
    is_tag: bool,
    repo: Option<&crate::repo::Repository>,
) -> UpdateMode {
    let Some(old) = old else {
        return UpdateMode::New;
    };
    if old == new {
        return UpdateMode::UpToDate;
    }
    // Fast-forward when old is an ancestor of new (commit history only).
    let ff = repo
        .map(|r| crate::merge_base::is_ancestor(r, *old, *new).unwrap_or(false))
        .unwrap_or(false);
    if ff && !is_tag {
        return UpdateMode::FastForward;
    }
    if force {
        return UpdateMode::Forced;
    }
    if is_tag {
        return UpdateMode::TagUpdateRejected;
    }
    UpdateMode::NonFastForwardRejected
}

/// Delete local tracking refs whose remote counterpart no longer exists.
///
/// A local tracking ref is a prune candidate when it lives under the destination
/// namespace of some positive wildcard refspec and no current remote ref maps to
/// it. Matches `git fetch --prune` for the common `refs/remotes/<remote>/*` case.
fn prune_tracking_refs(
    local_git_dir: &Path,
    positive: &[RefspecItem],
    remote_refs: &[(String, ObjectId)],
    dry_run: bool,
    updates: &mut Vec<RefUpdate>,
) -> Result<()> {
    // Set of local tracking refs that the current remote justifies.
    let mut live: HashSet<String> = HashSet::new();
    for (name, _) in remote_refs {
        if let Some(Some(dst)) = match_positive(name, positive) {
            live.insert(dst);
        }
    }

    // For each wildcard positive refspec, enumerate existing local refs under its
    // destination prefix and prune those not in `live`.
    let mut pruned: HashMap<String, ObjectId> = HashMap::new();
    for item in positive {
        let Some(dst) = item.dst.as_deref() else {
            continue;
        };
        let Some(star) = dst.find('*') else {
            continue;
        };
        let prefix = &dst[..star];
        for (name, oid) in crate::refs::list_refs(local_git_dir, prefix)? {
            if !name.starts_with(prefix) {
                continue;
            }
            if !live.contains(&name) {
                pruned.entry(name).or_insert(oid);
            }
        }
    }

    for (name, oid) in pruned {
        if !dry_run {
            crate::refs::delete_ref(local_git_dir, &name)?;
        }
        updates.push(RefUpdate {
            remote_ref: String::new(),
            local_ref: Some(name),
            old_oid: Some(oid),
            new_oid: None,
            mode: UpdateMode::DeletedMissing,
            note: Some("pruned (gone on remote)".to_owned()),
        });
    }
    Ok(())
}
