//! `grit bundle` — move objects and refs by archive.
//!
//! Implements create, verify, list-heads, and unbundle subcommands.

use crate::explicit_exit::SilentNonZeroExit;
use anyhow::{bail, Context, Result};
use clap::{Args as ClapArgs, Subcommand};
use grit_lib::git_date::approx::approxidate_careful;
use grit_lib::git_date::parse::parse_date_basic;
use sha1::{Digest, Sha1};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};

use grit_lib::objects::{parse_commit, parse_tag, ObjectId, ObjectKind};
use grit_lib::refs;
use grit_lib::repo::Repository;
use grit_lib::rev_list::{rev_list, split_revision_token, RevListOptions};

/// Arguments for `grit bundle`.
#[derive(Debug, ClapArgs)]
pub struct Args {
    #[command(subcommand)]
    pub action: BundleAction,
}

#[derive(Debug, Subcommand)]
pub enum BundleAction {
    /// Create a bundle file.
    Create(CreateArgs),
    /// Verify a bundle file.
    Verify(VerifyArgs),
    /// List references in a bundle.
    #[command(name = "list-heads")]
    ListHeads(ListHeadsArgs),
    /// Unbundle objects from a bundle file.
    Unbundle(UnbundleArgs),
}

#[derive(Debug, ClapArgs)]
pub struct CreateArgs {
    /// Suppress progress output.
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Force progress output.
    #[arg(long = "progress")]
    pub progress: bool,

    /// Re-enable progress output after --quiet.
    #[arg(long = "no-quiet")]
    pub no_quiet: bool,

    /// Output bundle file path.
    #[arg(value_name = "FILE")]
    pub file: String,

    /// Bundle format version (supports 2 and 3).
    #[arg(long = "version", value_name = "N")]
    pub version: Option<u8>,

    /// Read revision arguments from standard input.
    #[arg(long = "stdin")]
    pub stdin: bool,

    /// Ignore missing refs while parsing revision arguments.
    #[arg(long = "ignore-missing")]
    pub ignore_missing: bool,

    /// Revision arguments (refs, commit ranges, --all).
    #[arg(value_name = "REV", num_args = 0.., allow_hyphen_values = true, trailing_var_arg = true)]
    pub rev_list_args: Vec<String>,
}

#[derive(Debug, ClapArgs)]
pub struct VerifyArgs {
    /// Suppress progress output.
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Bundle file to verify.
    #[arg(value_name = "FILE")]
    pub file: String,
}

#[derive(Debug, ClapArgs)]
pub struct ListHeadsArgs {
    /// Bundle file.
    #[arg(value_name = "FILE")]
    pub file: String,
}

#[derive(Debug, ClapArgs)]
pub struct UnbundleArgs {
    /// Force progress output.
    #[arg(long = "progress")]
    pub progress: bool,

    /// Bundle file to unbundle.
    #[arg(value_name = "FILE")]
    pub file: String,
}

/// Run `grit bundle`.
pub fn run(args: Args) -> Result<()> {
    match args.action {
        BundleAction::Create(a) => run_create(a),
        BundleAction::Verify(a) => run_verify(a),
        BundleAction::ListHeads(a) => run_list_heads(a),
        BundleAction::Unbundle(a) => run_unbundle(a),
    }
}

// ---------------------------------------------------------------------------
// create
// ---------------------------------------------------------------------------

fn run_create(args: CreateArgs) -> Result<()> {
    let repo = Repository::discover(None).context("not a git repository")?;
    let version = args.version.unwrap_or(2);
    if version != 2 && version != 3 {
        bail!("unsupported bundle version {version}");
    }
    let rev_args = collect_create_rev_args(&args)?;

    let mut refs = collect_refs_for_bundle(&repo, &rev_args, args.ignore_missing)?;
    if refs.is_empty() {
        bail!("refusing to create empty bundle");
    }

    let (positive, negative) = parse_bundle_rev_list_args(&repo, &rev_args, args.ignore_missing)?;
    let cutoffs = parse_bundle_rev_cutoffs(&rev_args)?;
    let max_count = parse_max_count_arg(&rev_args);
    let opts = RevListOptions {
        objects: true,
        boundary: !negative.is_empty() || cutoffs.since.is_some() || cutoffs.until.is_some(),
        max_count,
        since_cutoff: cutoffs.since,
        until_cutoff: cutoffs.until,
        ignore_missing: args.ignore_missing,
        ..Default::default()
    };
    let listed =
        rev_list(&repo, &positive, &negative, &opts).map_err(|e| anyhow::anyhow!("{e}"))?;
    let mut prerequisites = listed.boundary_commits;
    if cutoffs.since.is_some() || cutoffs.until.is_some() || prerequisites.len() <= 2 {
        sort_bundle_prerequisites_desc(&repo, &mut prerequisites);
    } else {
        sort_bundle_prerequisites_asc(&repo, &mut prerequisites);
    }
    let mut oids = BTreeSet::new();
    for c in &listed.commits {
        oids.insert(*c);
    }
    if max_count.is_some() {
        for c in &listed.commits {
            let Ok(obj) = read_object(&repo, c) else {
                continue;
            };
            if obj.kind != ObjectKind::Commit {
                continue;
            }
            if let Ok(commit) = parse_commit(&obj.data) {
                let mut tip_tree_objects = BTreeSet::new();
                walk_reachable(&repo, &commit.tree, &mut tip_tree_objects)?;

                let mut parent_tree_objects = BTreeSet::new();
                for parent in &commit.parents {
                    let Ok(parent_obj) = read_object(&repo, parent) else {
                        continue;
                    };
                    if parent_obj.kind != ObjectKind::Commit {
                        continue;
                    }
                    if let Ok(parent_commit) = parse_commit(&parent_obj.data) {
                        walk_reachable(&repo, &parent_commit.tree, &mut parent_tree_objects)?;
                    }
                }

                for oid in tip_tree_objects {
                    if !parent_tree_objects.contains(&oid) {
                        oids.insert(oid);
                    }
                }
            }
        }
        for oid in refs.values() {
            if read_object(&repo, oid).is_ok_and(|obj| obj.kind == ObjectKind::Tag) {
                oids.insert(*oid);
            }
        }
        refs.retain(|_, oid| oids.contains(oid));
    } else {
        for (oid, _) in &listed.objects {
            if let Ok(obj) = read_object(&repo, oid) {
                if obj.kind == ObjectKind::Commit {
                    continue;
                }
            }
            oids.insert(*oid);
        }
        if !prerequisites.is_empty() {
            let mut prerequisite_objects = std::collections::BTreeSet::new();
            for boundary in &prerequisites {
                walk_reachable(&repo, boundary, &mut prerequisite_objects)?;
            }
            oids.retain(|oid| !prerequisite_objects.contains(oid));
            let has_blob = oids
                .iter()
                .any(|oid| read_object(&repo, oid).is_ok_and(|obj| obj.kind == ObjectKind::Blob));
            if !has_blob {
                for commit_oid in &listed.commits {
                    let Ok(commit_obj) = read_object(&repo, commit_oid) else {
                        continue;
                    };
                    if commit_obj.kind != ObjectKind::Commit {
                        continue;
                    }
                    let Ok(commit) = parse_commit(&commit_obj.data) else {
                        continue;
                    };
                    if let Some(blob_oid) = find_first_blob_in_tree(&repo, commit.tree) {
                        oids.insert(blob_oid);
                        break;
                    }
                }
            }
        }
        if cutoffs.since.is_some() || cutoffs.until.is_some() {
            include_prerequisite_commit_roots(&repo, &prerequisites, &mut oids);
            let included_commits = listed.commits.iter().copied().collect::<BTreeSet<_>>();
            retain_refs_for_included_commits(
                &repo,
                &included_commits,
                &cutoffs,
                &mut refs,
                &mut oids,
            );
        }
    }

    // Read all objects.
    let mut objects: Vec<(ObjectId, ObjectKind, Vec<u8>)> = Vec::new();
    for oid in &oids {
        let obj = read_object(&repo, oid)?;
        objects.push((*oid, obj.kind, obj.data));
    }

    // Build pack data.
    let pack_data = build_pack_data(&objects)?;

    // Write bundle file.
    if args.file == "-" {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        write_bundle(&mut out, version, &prerequisites, &refs, &pack_data)?;
    } else {
        let mut out =
            fs::File::create(&args.file).with_context(|| format!("cannot create {}", args.file))?;
        write_bundle(&mut out, version, &prerequisites, &refs, &pack_data)?;
    }

    if args.progress || (!args.quiet && !args.no_quiet && args.file != "-") {
        eprintln!("Total {} (delta 0), reused 0 (delta 0)", objects.len());
    }

    Ok(())
}

fn collect_create_rev_args(args: &CreateArgs) -> Result<Vec<String>> {
    let mut rev_args = Vec::new();
    if args.stdin {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        rev_args.extend(
            input
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToOwned::to_owned),
        );
    }
    rev_args.extend(args.rev_list_args.iter().cloned());
    Ok(rev_args)
}

fn write_bundle(
    out: &mut dyn Write,
    version: u8,
    prerequisites: &[ObjectId],
    refs: &BTreeMap<String, ObjectId>,
    pack_data: &[u8],
) -> Result<()> {
    if version == 3 {
        out.write_all(b"# v3 git bundle\n")?;
        out.write_all(b"@object-format=sha1\n")?;
    } else {
        out.write_all(b"# v2 git bundle\n")?;
    }

    for oid in prerequisites {
        writeln!(out, "-{} ", oid.to_hex())?;
    }
    for (refname, oid) in bundle_refs_for_output(refs) {
        writeln!(out, "{} {}", oid.to_hex(), refname)?;
    }
    out.write_all(b"\n")?;
    out.write_all(pack_data)?;
    Ok(())
}

fn collect_refs_for_bundle(
    repo: &Repository,
    rev_args: &[String],
    ignore_missing: bool,
) -> Result<BTreeMap<String, ObjectId>> {
    let mut refs = BTreeMap::new();

    let include_all = rev_args.iter().any(|a| a == "--all");

    if include_all {
        collect_all_refs(repo, &mut refs)?;
        return Ok(refs);
    }
    if rev_args.is_empty() {
        if let Ok(oid) = resolve_ref(repo, "HEAD") {
            refs.insert("HEAD".to_string(), oid);
        }
        return Ok(refs);
    }

    let mut i = 0usize;
    while i < rev_args.len() {
        let arg = &rev_args[i];
        if option_takes_value(arg) {
            i += 2;
            continue;
        }
        if arg == "--not" {
            i += 1;
            while i < rev_args.len() && rev_args[i] != "--not" {
                i += 1;
            }
            continue;
        }
        if arg == "--all" || (arg.starts_with('-') && arg != "--not") {
            i += 1;
            continue;
        }
        if let Some(tip_spec) = bundle_ref_tip_spec(arg) {
            match resolve_ref(repo, &tip_spec) {
                Ok(oid) => {
                    let full_name = full_ref_name_for_tip(repo, &tip_spec);
                    refs.insert(full_name, oid);
                }
                Err(e) if ignore_missing => {
                    let _ = e;
                }
                Err(e) => {
                    return Err(e)
                        .with_context(|| format!("cannot resolve '{tip_spec}' (from '{arg}')"));
                }
            }
        }
        i += 1;
    }

    Ok(refs)
}

fn bundle_ref_tip_spec(arg: &str) -> Option<String> {
    if arg.starts_with('^') {
        return None;
    }
    if let Some(base) = arg.strip_suffix("^!") {
        return Some(if base.is_empty() { "HEAD" } else { base }.to_string());
    }
    let (pos_specs, _neg) = split_revision_token(arg);
    pos_specs.last().cloned()
}

fn full_ref_name_for_tip(repo: &Repository, tip_spec: &str) -> String {
    if tip_spec.starts_with("refs/") || tip_spec == "HEAD" {
        tip_spec.to_string()
    } else if resolve_ref(repo, &format!("refs/heads/{tip_spec}")).is_ok() {
        format!("refs/heads/{tip_spec}")
    } else if resolve_ref(repo, &format!("refs/tags/{tip_spec}")).is_ok() {
        format!("refs/tags/{tip_spec}")
    } else {
        tip_spec.to_string()
    }
}

fn parse_max_count_arg(rev_args: &[String]) -> Option<usize> {
    let mut i = 0usize;
    while i < rev_args.len() {
        let arg = &rev_args[i];
        if arg == "--max-count" {
            if let Some(value) = rev_args.get(i + 1).and_then(|v| v.parse::<usize>().ok()) {
                return Some(value);
            }
            i += 2;
            continue;
        }
        if let Some(value) = arg
            .strip_prefix("--max-count=")
            .and_then(|v| v.parse::<usize>().ok())
        {
            return Some(value);
        }
        let Some(n) = arg.strip_prefix('-') else {
            i += 1;
            continue;
        };
        if !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(v) = n.parse::<usize>() {
                return Some(v);
            }
        }
        i += 1;
    }
    None
}

fn include_prerequisite_commit_roots(
    repo: &Repository,
    prerequisites: &[ObjectId],
    oids: &mut BTreeSet<ObjectId>,
) {
    for oid in prerequisites {
        let Ok(obj) = read_object(repo, oid) else {
            continue;
        };
        if obj.kind != ObjectKind::Commit {
            continue;
        }
        oids.insert(*oid);
        if let Ok(commit) = parse_commit(&obj.data) {
            oids.insert(commit.tree);
        }
    }
}

fn retain_refs_for_included_commits(
    repo: &Repository,
    included_commits: &BTreeSet<ObjectId>,
    cutoffs: &BundleRevCutoffs,
    refs: &mut BTreeMap<String, ObjectId>,
    oids: &mut BTreeSet<ObjectId>,
) {
    refs.retain(|_, oid| ref_peels_to_included_commit(repo, oid, included_commits, cutoffs, oids));
}

fn ref_peels_to_included_commit(
    repo: &Repository,
    oid: &ObjectId,
    included_commits: &BTreeSet<ObjectId>,
    cutoffs: &BundleRevCutoffs,
    oids: &mut BTreeSet<ObjectId>,
) -> bool {
    let Ok(obj) = read_object(repo, oid) else {
        return false;
    };
    match obj.kind {
        ObjectKind::Commit => included_commits.contains(oid),
        ObjectKind::Tag => {
            if oids.contains(oid) {
                return true;
            }
            let Ok(tag) = parse_tag(&obj.data) else {
                return false;
            };
            if tag
                .tagger
                .as_deref()
                .is_some_and(|tagger| signature_time_is_in_cutoffs(tagger, cutoffs))
            {
                oids.insert(*oid);
                return true;
            }
            if ref_peels_to_included_commit(repo, &tag.object, included_commits, cutoffs, oids) {
                oids.insert(*oid);
                true
            } else {
                false
            }
        }
        ObjectKind::Tree | ObjectKind::Blob => false,
    }
}

fn signature_time_is_in_cutoffs(sig: &str, cutoffs: &BundleRevCutoffs) -> bool {
    let ts = parse_signature_time(sig);
    if let Some(until) = cutoffs.until {
        if ts > until {
            return false;
        }
    }
    if let Some(since) = cutoffs.since {
        if ts < since {
            return false;
        }
    }
    true
}

fn sort_bundle_prerequisites_desc(repo: &Repository, prerequisites: &mut [ObjectId]) {
    prerequisites.sort_by(|a, b| {
        commit_time(repo, b)
            .cmp(&commit_time(repo, a))
            .then_with(|| a.to_hex().cmp(&b.to_hex()))
    });
}

fn sort_bundle_prerequisites_asc(repo: &Repository, prerequisites: &mut [ObjectId]) {
    prerequisites.sort_by(|a, b| {
        commit_time(repo, a)
            .cmp(&commit_time(repo, b))
            .then_with(|| a.to_hex().cmp(&b.to_hex()))
    });
}

fn commit_time(repo: &Repository, oid: &ObjectId) -> i64 {
    let Ok(obj) = read_object(repo, oid) else {
        return 0;
    };
    if obj.kind != ObjectKind::Commit {
        return 0;
    }
    let Ok(commit) = parse_commit(&obj.data) else {
        return 0;
    };
    parse_signature_time(&commit.committer)
}

fn parse_signature_time(sig: &str) -> i64 {
    let parts: Vec<&str> = sig.rsplitn(3, ' ').collect();
    if parts.len() >= 2 {
        parts[1].parse::<i64>().unwrap_or(0)
    } else {
        0
    }
}

fn commit_subject(repo: &Repository, oid: &ObjectId) -> Option<String> {
    let obj = read_object(repo, oid).ok()?;
    if obj.kind != ObjectKind::Commit {
        return None;
    }
    let commit = parse_commit(&obj.data).ok()?;
    commit.message.lines().next().map(ToOwned::to_owned)
}

fn parse_bundle_rev_list_args(
    repo: &Repository,
    rev_args: &[String],
    ignore_missing: bool,
) -> Result<(Vec<String>, Vec<String>)> {
    let mut positive: Vec<String> = Vec::new();
    let mut negative: Vec<String> = Vec::new();
    let include_all = rev_args.iter().any(|a| a == "--all");

    let mut i = 0usize;
    while i < rev_args.len() {
        let arg = &rev_args[i];
        if arg == "--since"
            || arg == "--after"
            || arg == "--until"
            || arg == "--before"
            || option_takes_value(arg)
        {
            i += 2;
            continue;
        }
        if arg.starts_with("--since=")
            || arg.starts_with("--after=")
            || arg.starts_with("--until=")
            || arg.starts_with("--before=")
        {
            i += 1;
            continue;
        }
        if arg == "--not" {
            i += 1;
            while i < rev_args.len() && rev_args[i] != "--not" {
                let tok = &rev_args[i];
                if tok == "--all" || (tok.starts_with('-') && tok != "--not") {
                    i += 1;
                    continue;
                }
                let (p, n) = split_bundle_revision_token(repo, tok)?;
                append_bundle_rev_specs(repo, &mut negative, p, ignore_missing);
                append_bundle_rev_specs(repo, &mut negative, n, ignore_missing);
                i += 1;
            }
            continue;
        }
        if arg == "--all" || (arg.starts_with('-') && arg != "--not") {
            i += 1;
            continue;
        }
        let (p, n) = split_bundle_revision_token(repo, arg)?;
        append_bundle_rev_specs(repo, &mut positive, p, ignore_missing);
        append_bundle_rev_specs(repo, &mut negative, n, ignore_missing);
        i += 1;
    }

    if include_all && positive.is_empty() {
        let mut refs = BTreeMap::new();
        collect_all_refs(repo, &mut refs)?;
        positive.extend(refs.keys().cloned());
    }

    if positive.is_empty() && !include_all {
        if let Ok(_) = resolve_ref(repo, "HEAD") {
            positive.push("HEAD".to_string());
        }
    }

    Ok((positive, negative))
}

fn append_bundle_rev_specs(
    repo: &Repository,
    out: &mut Vec<String>,
    specs: Vec<String>,
    ignore_missing: bool,
) {
    for spec in specs {
        if ignore_missing
            && grit_lib::rev_parse::resolve_revision_for_range_end(repo, &spec).is_err()
        {
            continue;
        }
        out.push(spec);
    }
}

fn option_takes_value(arg: &str) -> bool {
    matches!(arg, "--max-count")
}

fn split_bundle_revision_token(
    repo: &Repository,
    token: &str,
) -> Result<(Vec<String>, Vec<String>)> {
    if let Some(base) = token.strip_suffix("^!") {
        let base = if base.is_empty() { "HEAD" } else { base };
        let oid = grit_lib::rev_parse::resolve_revision_as_commit_without_index_dwim(repo, base)
            .with_context(|| format!("cannot resolve '{base}'"))?;
        let obj = read_object(repo, &oid)?;
        if obj.kind != ObjectKind::Commit {
            return Ok((vec![base.to_string()], Vec::new()));
        }
        let commit = parse_commit(&obj.data).map_err(|e| anyhow::anyhow!("{e}"))?;
        let parents = commit.parents.into_iter().map(|p| p.to_hex()).collect();
        return Ok((vec![base.to_string()], parents));
    }
    Ok(split_revision_token(token))
}

#[derive(Default)]
struct BundleRevCutoffs {
    since: Option<i64>,
    until: Option<i64>,
}

fn parse_bundle_rev_cutoffs(rev_args: &[String]) -> Result<BundleRevCutoffs> {
    let mut cutoffs = BundleRevCutoffs::default();
    let mut i = 0usize;
    while i < rev_args.len() {
        let arg = &rev_args[i];
        match arg.as_str() {
            "--since" | "--after" => {
                i += 1;
                if let Some(v) = rev_args.get(i) {
                    cutoffs.since = Some(parse_bundle_date(v)?);
                }
            }
            "--until" | "--before" => {
                i += 1;
                if let Some(v) = rev_args.get(i) {
                    cutoffs.until = Some(parse_bundle_date(v)?);
                }
            }
            _ if arg.starts_with("--since=") || arg.starts_with("--after=") => {
                let value = arg.split_once('=').map(|(_, v)| v).unwrap_or_default();
                cutoffs.since = Some(parse_bundle_date(value)?);
            }
            _ if arg.starts_with("--until=") || arg.starts_with("--before=") => {
                let value = arg.split_once('=').map(|(_, v)| v).unwrap_or_default();
                cutoffs.until = Some(parse_bundle_date(value)?);
            }
            _ => {}
        }
        i += 1;
    }
    Ok(cutoffs)
}

fn parse_bundle_date(s: &str) -> Result<i64> {
    let trimmed = s.trim();
    let mut approx_err = 0;
    let approx = approxidate_careful(trimmed, Some(&mut approx_err));
    if approx_err == 0 {
        return i64::try_from(approx).context("date out of range for bundle cutoff");
    }
    if let Ok((ts, _)) = parse_date_basic(trimmed) {
        return i64::try_from(ts).context("date out of range for bundle cutoff");
    }
    if trimmed.len() >= 10 && trimmed.as_bytes()[4] == b'-' && trimmed.as_bytes()[7] == b'-' {
        let parts: Vec<&str> = trimmed[..10].split('-').collect();
        if parts.len() == 3 {
            if let (Ok(y), Ok(m), Ok(d)) = (
                parts[0].parse::<i32>(),
                parts[1].parse::<u8>(),
                parts[2].parse::<u8>(),
            ) {
                if let Ok(month) = time::Month::try_from(m) {
                    if let Ok(date) = time::Date::from_calendar_date(y, month, d) {
                        if let Ok(dt) = date.with_hms(0, 0, 0) {
                            return Ok(dt.assume_utc().unix_timestamp());
                        }
                    }
                }
            }
        }
    }
    trimmed
        .parse::<i64>()
        .with_context(|| format!("invalid date '{trimmed}'"))
}

fn collect_all_refs(repo: &Repository, refs: &mut BTreeMap<String, ObjectId>) -> Result<()> {
    // HEAD
    if let Ok(oid) = resolve_ref(repo, "HEAD") {
        refs.insert("HEAD".to_string(), oid);
    }

    // Walk refs/ directory.
    let refs_dir = repo.git_dir.join("refs");
    if refs_dir.exists() {
        walk_refs_dir(&refs_dir, "refs", repo, refs)?;
    }

    Ok(())
}

fn walk_refs_dir(
    dir: &std::path::Path,
    prefix: &str,
    repo: &Repository,
    refs: &mut BTreeMap<String, ObjectId>,
) -> Result<()> {
    let rd = fs::read_dir(dir)?;
    for entry in rd {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let full_ref = format!("{prefix}/{name_str}");

        if path.is_dir() {
            walk_refs_dir(&path, &full_ref, repo, refs)?;
        } else if path.is_file() {
            if let Ok(oid) = resolve_ref(repo, &full_ref) {
                refs.insert(full_ref, oid);
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// verify
// ---------------------------------------------------------------------------

fn run_verify(args: VerifyArgs) -> Result<()> {
    let repo = Repository::discover(None).ok();
    let data = read_bundle_arg(&args.file)?;
    let header = parse_bundle_header(&data)?;

    // Validate pack data.
    let pack_data = &data[header.pack_start..];
    if pack_data.len() < 12 + 20 {
        bail!("bundle pack data too small");
    }
    if &pack_data[0..4] != b"PACK" {
        bail!("bundle does not contain valid pack data");
    }

    if let Some(repo) = &repo {
        if !header.prerequisites.is_empty() {
            let prereq_oids: Vec<_> = header.prerequisites.iter().map(|(oid, _)| *oid).collect();
            let missing: Vec<_> = prereq_oids
                .iter()
                .copied()
                .filter(|oid| read_object(repo, oid).is_err())
                .collect();
            if !missing.is_empty() {
                eprintln!("error: Repository lacks these prerequisite commits:");
                for oid in missing {
                    eprintln!("error: {} ", oid.to_hex());
                }
                return Err(anyhow::Error::new(SilentNonZeroExit { code: 1 }));
            }
            if !grit_lib::connectivity::bundle_prerequisites_connected_to_refs(repo, &prereq_oids)
                .map_err(|e| anyhow::anyhow!("{e}"))?
            {
                eprintln!("error: some prerequisite commits are not connected to the repository");
                return Err(anyhow::Error::new(SilentNonZeroExit { code: 1 }));
            }
        }
    }

    print_bundle_verify_info(&header);
    if !args.quiet {
        eprintln!("{} is okay", bundle_display_name(&args.file));
    }
    Ok(())
}

fn print_bundle_verify_info(header: &BundleHeader) {
    match header.refs.len() {
        1 => println!("The bundle contains this ref:"),
        n => println!("The bundle contains these {n} refs:"),
    }
    for (refname, oid) in bundle_refs_for_output(&header.refs) {
        println!("{} {refname}", oid.to_hex());
    }
    match header.prerequisites.len() {
        0 => println!("The bundle records a complete history."),
        1 => println!("The bundle requires this ref:"),
        n => println!("The bundle requires these {n} refs:"),
    }
    for (oid, comment) in &header.prerequisites {
        println!("{} {comment}", oid.to_hex());
    }
    let display_hash = std::env::var("GIT_DEFAULT_HASH").unwrap_or_default();
    println!("The bundle uses this hash algorithm: {display_hash}");
    if let Some(filter) = &header.filter {
        println!("The bundle uses this filter: {filter}");
    }
}

// ---------------------------------------------------------------------------
// list-heads
// ---------------------------------------------------------------------------

fn run_list_heads(args: ListHeadsArgs) -> Result<()> {
    let data = read_bundle_arg(&args.file)?;
    let header = parse_bundle_header(&data)?;

    for (refname, oid) in bundle_refs_for_output(&header.refs) {
        println!("{} {refname}", oid.to_hex());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// unbundle
// ---------------------------------------------------------------------------

fn run_unbundle(args: UnbundleArgs) -> Result<()> {
    let repo = Repository::discover(None).context("not a git repository")?;
    let data = read_bundle_arg(&args.file)?;
    let header = parse_bundle_header(&data)?;

    let pack_data = &data[header.pack_start..];
    if pack_data.len() < 12 + 20 {
        bail!("bundle pack data too small");
    }

    let prereq_oids: Vec<_> = header.prerequisites.iter().map(|(oid, _)| *oid).collect();
    if !grit_lib::connectivity::bundle_prerequisites_connected_to_refs(&repo, &prereq_oids)
        .map_err(|e| anyhow::anyhow!("{e}"))?
    {
        eprintln!("error: some prerequisite commits are not connected to the repository");
        return Err(anyhow::Error::new(SilentNonZeroExit { code: 1 }));
    }

    // Use unpack-objects to extract into the ODB.
    let opts = grit_lib::unpack_objects::UnpackOptions {
        strict: false,
        dry_run: false,
        quiet: !args.progress,
        max_input_bytes: None,
    };
    let _count = grit_lib::unpack_objects::unpack_objects(&mut &pack_data[..], &repo.odb, &opts)
        .map_err(|e| anyhow::anyhow!("unbundle failed: {e}"))?;

    for (refname, oid) in &header.refs {
        println!("{} {refname}", oid.to_hex());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// shared helpers
// ---------------------------------------------------------------------------

struct BundleHeader {
    refs: BTreeMap<String, ObjectId>,
    prerequisites: Vec<(ObjectId, String)>,
    object_format: String,
    filter: Option<String>,
    pack_start: usize,
}

fn read_bundle_arg(file: &str) -> Result<Vec<u8>> {
    if file == "-" {
        let mut data = Vec::new();
        std::io::stdin().read_to_end(&mut data)?;
        return Ok(data);
    }
    fs::read(file).with_context(|| format!("cannot read {file}"))
}

fn bundle_display_name(file: &str) -> &str {
    if file == "-" {
        "<stdin>"
    } else {
        file
    }
}

fn bundle_refs_for_output(refs: &BTreeMap<String, ObjectId>) -> Vec<(&String, &ObjectId)> {
    let mut ordered = Vec::with_capacity(refs.len());
    ordered.extend(refs.iter().filter(|(name, _)| name.as_str() != "HEAD"));
    if let Some(head) = refs.get_key_value("HEAD") {
        ordered.push(head);
    }
    ordered
}

/// Parse the bundle header, returning refs/prerequisites and the pack byte offset.
fn parse_bundle_header(data: &[u8]) -> Result<BundleHeader> {
    let header_v2 = b"# v2 git bundle\n";
    let header_v3 = b"# v3 git bundle\n";
    let mut pos = if data.starts_with(header_v2) {
        header_v2.len()
    } else if data.starts_with(header_v3) {
        header_v3.len()
    } else {
        bail!("not a git bundle");
    };
    let mut refs = BTreeMap::new();
    let mut prerequisites = Vec::new();
    let mut object_format = "sha1".to_string();
    let mut filter = None;

    loop {
        // Find end of line.
        let eol = data[pos..]
            .iter()
            .position(|&b| b == b'\n')
            .map(|i| pos + i)
            .ok_or_else(|| anyhow::anyhow!("truncated bundle header"))?;

        let line = &data[pos..eol];
        if line.is_empty() {
            // Blank line → pack data follows.
            pos = eol + 1;
            break;
        }

        let line_str = std::str::from_utf8(line).context("invalid UTF-8 in bundle header")?;

        // Prerequisite lines start with '-'.
        if let Some(rest) = line_str.strip_prefix('-') {
            let (hex, comment) = rest.split_once(' ').unwrap_or((rest, ""));
            let oid = ObjectId::from_hex(hex)
                .map_err(|e| anyhow::anyhow!("bad prerequisite oid in bundle header: {e}"))?;
            prerequisites.push((oid, comment.to_string()));
            pos = eol + 1;
            continue;
        }
        if let Some(cap) = line_str.strip_prefix('@') {
            if let Some(value) = cap.strip_prefix("object-format=") {
                object_format = value.to_string();
            } else if let Some(value) = cap.strip_prefix("filter=") {
                filter = Some(value.to_string());
            }
            pos = eol + 1;
            continue;
        }

        // ref line: "<hex-oid> <refname>"
        if let Some((hex, refname)) = line_str.split_once(' ') {
            let oid = ObjectId::from_hex(hex)
                .map_err(|e| anyhow::anyhow!("bad oid in bundle header: {e}"))?;
            refs.insert(refname.to_string(), oid);
        }

        pos = eol + 1;
    }

    Ok(BundleHeader {
        refs,
        prerequisites,
        object_format,
        filter,
        pack_start: pos,
    })
}

fn resolve_ref(repo: &Repository, refname: &str) -> Result<ObjectId> {
    refs::resolve_ref(&repo.git_dir, refname)
        .or_else(|_| refs::resolve_ref(&repo.git_dir, &format!("refs/heads/{refname}")))
        .or_else(|_| refs::resolve_ref(&repo.git_dir, &format!("refs/tags/{refname}")))
        .map_err(|e| anyhow::anyhow!("cannot resolve ref '{refname}': {e}"))
}

fn walk_reachable(
    repo: &Repository,
    oid: &ObjectId,
    oids: &mut std::collections::BTreeSet<ObjectId>,
) -> Result<()> {
    if !oids.insert(*oid) {
        return Ok(());
    }
    let obj = match read_object(repo, oid) {
        Ok(o) => o,
        Err(_) => return Ok(()),
    };
    match obj.kind {
        ObjectKind::Commit => {
            if let Ok(text) = std::str::from_utf8(&obj.data) {
                for line in text.lines() {
                    if let Some(hex) = line.strip_prefix("tree ") {
                        if let Ok(tree_oid) = ObjectId::from_hex(hex.trim()) {
                            walk_reachable(repo, &tree_oid, oids)?;
                        }
                    } else if let Some(hex) = line.strip_prefix("parent ") {
                        if let Ok(parent_oid) = ObjectId::from_hex(hex.trim()) {
                            walk_reachable(repo, &parent_oid, oids)?;
                        }
                    } else if line.is_empty() {
                        break;
                    }
                }
            }
        }
        ObjectKind::Tree => {
            let data = &obj.data;
            let mut pos = 0;
            while pos < data.len() {
                let nul = data[pos..].iter().position(|&b| b == 0).map(|i| pos + i);
                if let Some(nul) = nul {
                    if nul + 21 <= data.len() {
                        if let Ok(entry_oid) = ObjectId::from_bytes(&data[nul + 1..nul + 21]) {
                            walk_reachable(repo, &entry_oid, oids)?;
                        }
                        pos = nul + 21;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        ObjectKind::Tag => {
            if let Ok(text) = std::str::from_utf8(&obj.data) {
                if let Some(first_line) = text.lines().next() {
                    if let Some(hex) = first_line.strip_prefix("object ") {
                        if let Ok(target_oid) = ObjectId::from_hex(hex.trim()) {
                            walk_reachable(repo, &target_oid, oids)?;
                        }
                    }
                }
            }
        }
        ObjectKind::Blob => {}
    }
    Ok(())
}

fn find_first_blob_in_tree(repo: &Repository, tree_oid: ObjectId) -> Option<ObjectId> {
    let tree_obj = read_object(repo, &tree_oid).ok()?;
    if tree_obj.kind != ObjectKind::Tree {
        return None;
    }
    let mut pos = 0usize;
    while pos < tree_obj.data.len() {
        let Some(nul_rel) = tree_obj.data[pos..].iter().position(|&b| b == 0) else {
            break;
        };
        let nul = pos + nul_rel;
        if nul + 21 > tree_obj.data.len() {
            break;
        }
        let mode_end = tree_obj.data[pos..nul].iter().position(|&b| b == b' ')?;
        let mode = std::str::from_utf8(&tree_obj.data[pos..pos + mode_end]).ok()?;
        let oid = ObjectId::from_bytes(&tree_obj.data[nul + 1..nul + 21]).ok()?;
        if mode == "100644" || mode == "100755" || mode == "120000" {
            return Some(oid);
        }
        if mode == "40000" {
            if let Some(found) = find_first_blob_in_tree(repo, oid) {
                return Some(found);
            }
        }
        pos = nul + 21;
    }
    None
}

fn read_object(repo: &Repository, oid: &ObjectId) -> Result<grit_lib::objects::Object> {
    if let Ok(obj) = repo.odb.read(oid) {
        return Ok(obj);
    }
    // Try pack files.
    let indexes = grit_lib::pack::read_local_pack_indexes(repo.odb.objects_dir())
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    for idx in &indexes {
        if let Some(entry) = idx
            .entries
            .iter()
            .find(|e| grit_lib::pack::pack_index_entry_matches_sha1_oid(e, oid))
        {
            let pack_bytes = fs::read(&idx.pack_path)?;
            return read_from_pack(&pack_bytes, entry.offset, &indexes);
        }
    }
    bail!("object not found: {}", oid.to_hex())
}

fn read_from_pack(
    pack_bytes: &[u8],
    offset: u64,
    indexes: &[grit_lib::pack::PackIndex],
) -> Result<grit_lib::objects::Object> {
    let mut pos = offset as usize;
    let c = pack_bytes
        .get(pos)
        .copied()
        .ok_or_else(|| anyhow::anyhow!("truncated"))?;
    pos += 1;
    let type_code = (c >> 4) & 0x7;
    let mut size = (c & 0x0f) as usize;
    let mut shift = 4u32;
    let mut cur = c;
    while cur & 0x80 != 0 {
        cur = pack_bytes
            .get(pos)
            .copied()
            .ok_or_else(|| anyhow::anyhow!("truncated"))?;
        pos += 1;
        size |= ((cur & 0x7f) as usize) << shift;
        shift += 7;
    }

    match type_code {
        1..=4 => {
            let kind = match type_code {
                1 => ObjectKind::Commit,
                2 => ObjectKind::Tree,
                3 => ObjectKind::Blob,
                4 => ObjectKind::Tag,
                _ => unreachable!(),
            };
            use flate2::read::ZlibDecoder;
            let mut dec = ZlibDecoder::new(&pack_bytes[pos..]);
            let mut data = Vec::with_capacity(size);
            dec.read_to_end(&mut data)?;
            Ok(grit_lib::objects::Object::new(kind, data))
        }
        6 => {
            let mut c2 = pack_bytes
                .get(pos)
                .copied()
                .ok_or_else(|| anyhow::anyhow!("truncated"))?;
            pos += 1;
            let mut neg_off = (c2 & 0x7f) as u64;
            while c2 & 0x80 != 0 {
                c2 = pack_bytes
                    .get(pos)
                    .copied()
                    .ok_or_else(|| anyhow::anyhow!("truncated"))?;
                pos += 1;
                neg_off = ((neg_off + 1) << 7) | (c2 & 0x7f) as u64;
            }
            let base_offset = offset
                .checked_sub(neg_off)
                .ok_or_else(|| anyhow::anyhow!("ofs-delta underflow"))?;

            use flate2::read::ZlibDecoder;
            let mut dec = ZlibDecoder::new(&pack_bytes[pos..]);
            let mut delta = Vec::with_capacity(size);
            dec.read_to_end(&mut delta)?;

            let base = read_from_pack(pack_bytes, base_offset, indexes)?;
            let result = grit_lib::unpack_objects::apply_delta(&base.data, &delta)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(grit_lib::objects::Object::new(base.kind, result))
        }
        7 => {
            if pos + 20 > pack_bytes.len() {
                bail!("truncated ref-delta");
            }
            let base_oid = ObjectId::from_bytes(&pack_bytes[pos..pos + 20])
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            pos += 20;

            use flate2::read::ZlibDecoder;
            let mut dec = ZlibDecoder::new(&pack_bytes[pos..]);
            let mut delta = Vec::with_capacity(size);
            dec.read_to_end(&mut delta)?;

            let mut base_obj = None;
            for idx in indexes {
                if let Some(e) = idx
                    .entries
                    .iter()
                    .find(|e| grit_lib::pack::pack_index_entry_matches_sha1_oid(e, &base_oid))
                {
                    let pb = fs::read(&idx.pack_path)?;
                    base_obj = Some(read_from_pack(&pb, e.offset, indexes)?);
                    break;
                }
            }
            let base = base_obj.ok_or_else(|| anyhow::anyhow!("ref-delta base not found"))?;
            let result = grit_lib::unpack_objects::apply_delta(&base.data, &delta)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            Ok(grit_lib::objects::Object::new(base.kind, result))
        }
        other => bail!("unknown pack type {other}"),
    }
}

fn build_pack_data(objects: &[(ObjectId, ObjectKind, Vec<u8>)]) -> Result<Vec<u8>> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;

    let mut buf = Vec::new();
    buf.extend_from_slice(b"PACK");
    buf.extend_from_slice(&2u32.to_be_bytes());
    buf.extend_from_slice(&(objects.len() as u32).to_be_bytes());

    for (_, kind, data) in objects {
        let type_code: u8 = match kind {
            ObjectKind::Commit => 1,
            ObjectKind::Tree => 2,
            ObjectKind::Blob => 3,
            ObjectKind::Tag => 4,
        };
        let mut size = data.len();
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

        let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
        enc.write_all(data)?;
        let compressed = enc.finish()?;
        buf.extend_from_slice(&compressed);
    }

    let mut hasher = Sha1::new();
    hasher.update(&buf);
    let digest = hasher.finalize();
    buf.extend_from_slice(digest.as_slice());

    Ok(buf)
}
