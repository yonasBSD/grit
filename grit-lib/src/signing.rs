//! Commit/tag GPG (and gpgsm/ssh) signing and signature verification.
//!
//! This is a port of the parts of Git's `gpg-interface.c` and `commit.c`
//! signature handling that the grit CLI needs:
//!
//! * read `gpg.format`, `gpg.<fmt>.program` / `gpg.program`, `user.signingkey`
//!   and `gpg.minTrustLevel`,
//! * resolve the signing program (handling a leading `~`, absolute paths, and
//!   bare names looked up on `$PATH`),
//! * [`sign_buffer`] — spawn `<program> --status-fd=2 -bsau <key>` and capture
//!   the armored detached signature,
//! * [`add_header_signature`] — splice a `gpgsig` header into a serialized
//!   commit object (Git `commit.c:add_header_signature`),
//! * [`extract_signed_payload`] / [`verify_commit`] — strip the `gpgsig` header
//!   to rebuild the signed payload and run `<program> --verify` over it,
//!   parsing the `[GNUPG:]` status lines into a [`SignatureCheck`].
//!
//! Only the gpg-based formats (`openpgp` -> `gpg`, `x509` -> `gpgsm`) implement
//! signing/verification here; `ssh` is recognized for `gpg.format` validation
//! but its sign/verify paths are not exercised by the commit/verify-commit
//! tests and return an explanatory error.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::config::ConfigSet;
use crate::error::{Error, Result};

/// The hash header label for a sha1 repository.
pub const GPG_SIG_HEADER_SHA1: &str = "gpgsig";
/// The hash header label for a sha256 repository.
pub const GPG_SIG_HEADER_SHA256: &str = "gpgsig-sha256";

/// Signature trust level, mirroring Git's `enum signature_trust_level`.
///
/// The numeric ordering matters: `gpg.minTrustLevel` comparisons use it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TrustLevel {
    #[default]
    Undefined = 0,
    Never = 1,
    Marginal = 2,
    Fully = 3,
    Ultimate = 4,
}

impl TrustLevel {
    /// The `%GT` display string (`undefined`, `never`, ...).
    pub fn display_key(self) -> &'static str {
        match self {
            TrustLevel::Undefined => "undefined",
            TrustLevel::Never => "never",
            TrustLevel::Marginal => "marginal",
            TrustLevel::Fully => "fully",
            TrustLevel::Ultimate => "ultimate",
        }
    }

    /// Parse an uppercase GNUPG `TRUST_<LEVEL>` suffix.
    fn from_status(level: &str) -> Option<TrustLevel> {
        match level {
            "UNDEFINED" => Some(TrustLevel::Undefined),
            "NEVER" => Some(TrustLevel::Never),
            "MARGINAL" => Some(TrustLevel::Marginal),
            "FULLY" => Some(TrustLevel::Fully),
            "ULTIMATE" => Some(TrustLevel::Ultimate),
            _ => None,
        }
    }

    /// Parse a configured `gpg.minTrustLevel` value (case-insensitive).
    pub fn from_config(value: &str) -> Option<TrustLevel> {
        TrustLevel::from_status(&value.to_ascii_uppercase())
    }
}

/// The signature format selected via `gpg.format`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpgFormat {
    OpenPgp,
    X509,
    Ssh,
}

impl GpgFormat {
    /// Resolve a `gpg.format` value case-sensitively (Git `get_format_by_name`).
    ///
    /// `openpgp` is valid; `OpEnPgP` is not (matches subtest 24).
    pub fn from_name(name: &str) -> Option<GpgFormat> {
        match name {
            "openpgp" => Some(GpgFormat::OpenPgp),
            "x509" => Some(GpgFormat::X509),
            "ssh" => Some(GpgFormat::Ssh),
            _ => None,
        }
    }

    /// The format name used in `gpg.<fmt>.program`.
    fn name(self) -> &'static str {
        match self {
            GpgFormat::OpenPgp => "openpgp",
            GpgFormat::X509 => "x509",
            GpgFormat::Ssh => "ssh",
        }
    }

    /// Default program for this format.
    fn default_program(self) -> &'static str {
        match self {
            GpgFormat::OpenPgp => "gpg",
            GpgFormat::X509 => "gpgsm",
            GpgFormat::Ssh => "ssh-keygen",
        }
    }

    /// Extra arguments passed before `--verify` for this format.
    fn verify_args(self) -> &'static [&'static str] {
        match self {
            GpgFormat::OpenPgp => &["--keyid-format=long"],
            GpgFormat::X509 => &["--keyid-format=long"],
            GpgFormat::Ssh => &[],
        }
    }
}

/// Resolved signing/verification configuration.
#[derive(Debug, Clone)]
pub struct GpgConfig {
    /// The selected format.
    pub format: GpgFormat,
    /// The resolved program command (may be a bare name to look up on `$PATH`).
    pub program: String,
    /// `user.signingkey`, if set.
    pub signing_key: Option<String>,
    /// `gpg.minTrustLevel`, if set.
    pub min_trust_level: Option<TrustLevel>,
}

impl GpgConfig {
    /// Read the signing configuration from a [`ConfigSet`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::ConfigError`] when `gpg.format` holds an unrecognized
    /// value (Git rejects this case-sensitively).
    pub fn from_config(config: &ConfigSet) -> Result<GpgConfig> {
        let format = match config.get("gpg.format") {
            Some(raw) => GpgFormat::from_name(&raw).ok_or_else(|| {
                Error::ConfigError(format!("invalid value for 'gpg.format': '{raw}'"))
            })?,
            None => GpgFormat::OpenPgp,
        };

        // `gpg.<fmt>.program` takes precedence over `gpg.program`.
        let fmt_program_key = format!("gpg.{}.program", format.name());
        let program = config
            .get(&fmt_program_key)
            .or_else(|| config.get("gpg.program"))
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| format.default_program().to_owned());

        let signing_key = config.get("user.signingkey").filter(|k| !k.is_empty());

        let min_trust_level = config
            .get("gpg.mintrustlevel")
            .and_then(|v| TrustLevel::from_config(&v));

        Ok(GpgConfig {
            format,
            program,
            signing_key,
            min_trust_level,
        })
    }

    /// The signing key to use: the explicit `key_override`, else
    /// `user.signingkey`, else the supplied committer identity (Git passes
    /// `git_committer_info(IDENT_STRICT | IDENT_NO_DATE)`).
    pub fn resolve_signing_key(
        &self,
        key_override: Option<&str>,
        committer_default: &str,
    ) -> String {
        if let Some(k) = key_override {
            if !k.is_empty() {
                return k.to_owned();
            }
        }
        if let Some(k) = &self.signing_key {
            return k.clone();
        }
        committer_default.to_owned()
    }

    /// Resolve [`Self::program`] to an executable path.
    ///
    /// Mirrors Git's program resolution: a leading `~/` expands to `$HOME`, an
    /// absolute path is used verbatim, and a bare name is searched on `$PATH`.
    pub fn resolve_program_path(&self) -> Result<PathBuf> {
        resolve_program(&self.program)
    }
}

/// Resolve a program string to an executable path.
fn resolve_program(program: &str) -> Result<PathBuf> {
    // `~` / `~/...` expansion relative to $HOME.
    if program == "~" {
        if let Some(home) = home_dir() {
            return Ok(home);
        }
    }
    if let Some(rest) = program.strip_prefix("~/") {
        if let Some(home) = home_dir() {
            return Ok(home.join(rest));
        }
    }

    let path = Path::new(program);
    // Absolute path, or any relative path that contains a separator: use as-is.
    if path.is_absolute() || program.contains('/') {
        return Ok(path.to_path_buf());
    }

    // Bare name: search $PATH.
    if let Some(found) = search_path(program) {
        return Ok(found);
    }

    // Fall back to the bare name and let the OS resolve it (preserves Git's
    // behavior of handing the name straight to exec when not found on PATH).
    Ok(path.to_path_buf())
}

/// Look up a bare program name on `$PATH`.
fn search_path(name: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let candidate = dir.join(name);
        if is_executable_file(&candidate) {
            return Some(candidate);
        }
    }
    None
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(path) {
        Ok(meta) => meta.is_file() && (meta.permissions().mode() & 0o111 != 0),
        Err(_) => false,
    }
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

/// The user's home directory (`$HOME`).
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

/// Sign `payload` with `signing_key` using the configured program.
///
/// Spawns `<program> --status-fd=2 -bsau <signing_key>`, writes `payload` to
/// stdin, and returns the armored detached signature from stdout.  Fails if the
/// child exits non-zero or does not emit a `[GNUPG:] SIG_CREATED` status line —
/// in either case the program's stderr is surfaced in the error (the
/// `LET_GPG_PROGRAM_FAIL`/`zOMG` path of subtest 28).
///
/// # Errors
///
/// Returns [`Error::Signing`] when the program cannot be spawned, exits
/// non-zero, or fails to produce a signature.
pub fn sign_buffer(cfg: &GpgConfig, payload: &[u8], signing_key: &str) -> Result<Vec<u8>> {
    if cfg.format == GpgFormat::Ssh {
        return Err(Error::Signing(
            "ssh signing is not supported by grit".to_owned(),
        ));
    }

    let program = cfg.resolve_program_path()?;

    let mut child = Command::new(&program)
        .arg("--status-fd=2")
        .arg("-bsau")
        .arg(signing_key)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            Error::Signing(format!(
                "could not run gpg program '{}': {e}",
                program.display()
            ))
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        // Ignore broken-pipe errors: a bad signing key can make gpg exit
        // before consuming all input (Git ignores SIGPIPE here too).
        let _ = stdin.write_all(payload);
        drop(stdin);
    }

    let output = child
        .wait_with_output()
        .map_err(|e| Error::Signing(format!("failed waiting for gpg program: {e}")))?;

    let status_text = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() || !has_sig_created(&status_text) {
        let detail = if status_text.trim().is_empty() {
            "(no gpg output)".to_owned()
        } else {
            status_text.into_owned()
        };
        return Err(Error::Signing(format!(
            "gpg failed to sign the data:\n{detail}"
        )));
    }

    Ok(output.stdout)
}

/// True when a `[GNUPG:] SIG_CREATED ` status line is present at the start of a
/// line (Git's `sign_buffer_gpg` SIG_CREATED scan).
fn has_sig_created(status: &str) -> bool {
    status
        .lines()
        .any(|line| line.starts_with("[GNUPG:] SIG_CREATED "))
}

/// Splice a signature into a serialized commit object as a `gpgsig` header.
///
/// Port of Git's `commit.c:add_header_signature`: find the first `\n\n`, insert
/// the header at the position right after that first `\n`, prefix the first
/// signature line with `<header> ` and every subsequent line with a single
/// space.  `header` is [`GPG_SIG_HEADER_SHA1`] or [`GPG_SIG_HEADER_SHA256`].
pub fn add_header_signature(buf: &[u8], sig: &[u8], header: &str) -> Vec<u8> {
    // Find end of header (first occurrence of "\n\n"); inspos is just past the
    // first '\n'. If absent, append at the end.
    let inspos = find_double_newline(buf).map(|p| p + 1).unwrap_or(buf.len());

    let mut out = Vec::with_capacity(buf.len() + sig.len() + header.len() + 16);
    out.extend_from_slice(&buf[..inspos]);

    let mut first = true;
    let mut copypos = 0usize;
    while copypos < sig.len() {
        let bol = copypos;
        // End of this line, including the trailing '\n' when present.
        let end = match memchr(sig, copypos, b'\n') {
            Some(idx) => idx + 1,
            None => sig.len(),
        };

        if first {
            out.extend_from_slice(header.as_bytes());
            first = false;
        }
        out.push(b' ');
        out.extend_from_slice(&sig[bol..end]);
        copypos = end;
    }

    out.extend_from_slice(&buf[inspos..]);
    out
}

/// Find the first `\n\n` in `buf`, returning the index of the first `\n`.
fn find_double_newline(buf: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + 1 < buf.len() {
        if buf[i] == b'\n' && buf[i + 1] == b'\n' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Find the byte `needle` in `buf` starting at `from`.
fn memchr(buf: &[u8], from: usize, needle: u8) -> Option<usize> {
    buf.get(from..)
        .and_then(|s| s.iter().position(|&b| b == needle))
        .map(|p| p + from)
}

/// A parsed signature check result, mirroring Git's `struct signature_check`.
#[derive(Debug, Clone, Default)]
pub struct SignatureCheck {
    /// The detached armored signature extracted from the object.
    pub signature: Vec<u8>,
    /// The signed payload (object with `gpgsig` header removed).
    pub payload: Vec<u8>,
    /// `%G?` result: `G` good, `B` bad, `U` good+untrusted, `E` error,
    /// `N` no signature, `X`/`Y`/`R` expired/expired-key/revoked.
    pub result: char,
    /// `%GT` trust level.
    pub trust_level: TrustLevel,
    /// `%GK` key id.
    pub key: Option<String>,
    /// `%GS` signer (uid).
    pub signer: Option<String>,
    /// `%GF` signing key fingerprint.
    pub fingerprint: Option<String>,
    /// `%GP` primary key fingerprint.
    pub primary_key_fingerprint: Option<String>,
    /// Human-readable gpg output (stderr); shown by `--show-signature`.
    pub output: String,
    /// Raw `[GNUPG:]` status lines; shown by `verify-commit --raw`.
    pub gpg_status: String,
}

impl SignatureCheck {
    /// Construct the "no signature" result (`%G?` -> `N`).
    pub fn default_none() -> SignatureCheck {
        SignatureCheck {
            result: 'N',
            trust_level: TrustLevel::Undefined,
            ..Default::default()
        }
    }

    /// True when the signature verified as good (`G`) or good-but-expired-key
    /// (`Y`) — Git's success criterion in `check_signature`.
    pub fn is_good(&self) -> bool {
        self.result == 'G' || self.result == 'Y'
    }

    /// Overall verification result honoring `min_trust_level`: `Ok(())` when the
    /// signature is good and meets the configured minimum trust level.
    pub fn verify_status(&self, min_trust_level: Option<TrustLevel>) -> bool {
        if !self.is_good() {
            return false;
        }
        if let Some(min) = min_trust_level {
            if self.trust_level < min {
                return false;
            }
        }
        true
    }
}

/// Extract the `gpgsig` (or `gpgsig-sha256`) header value and the signed
/// payload from a raw commit object.
///
/// Returns `(payload, signature)` where `payload` is the commit object with the
/// `gpgsig` header removed (the bytes that were actually signed), and
/// `signature` is the de-indented armored signature.  Returns `None` when the
/// object carries no signature header.
pub fn extract_signed_payload(raw_commit: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    // Header region ends at first "\n\n".
    let header_end = find_double_newline(raw_commit)?;
    let header = &raw_commit[..=header_end]; // includes trailing first '\n'
    let body = &raw_commit[header_end + 1..]; // remaining (starts with '\n')

    let mut payload = Vec::with_capacity(raw_commit.len());
    let mut signature = Vec::new();
    let mut found = false;

    let mut idx = 0;
    while idx < header.len() {
        let line_end = memchr(header, idx, b'\n')
            .map(|p| p + 1)
            .unwrap_or(header.len());
        let line = &header[idx..line_end];

        let is_sig_header = line.starts_with(GPG_SIG_HEADER_SHA1.as_bytes())
            && line
                .get(GPG_SIG_HEADER_SHA1.len())
                .map(|&b| b == b' ')
                .unwrap_or(false);

        if is_sig_header && !found {
            found = true;
            // First signature line: text after "gpgsig ".
            let prefix_len = GPG_SIG_HEADER_SHA1.len() + 1;
            signature.extend_from_slice(&line[prefix_len..]);
            idx = line_end;
            // Subsequent continuation lines (leading space) belong to the sig.
            while idx < header.len() {
                let cont_end = memchr(header, idx, b'\n')
                    .map(|p| p + 1)
                    .unwrap_or(header.len());
                let cont = &header[idx..cont_end];
                if cont.first() == Some(&b' ') {
                    signature.extend_from_slice(&cont[1..]);
                    idx = cont_end;
                } else {
                    break;
                }
            }
            continue;
        }

        payload.extend_from_slice(line);
        idx = line_end;
    }

    if !found {
        return None;
    }

    payload.extend_from_slice(body);
    Some((payload, signature))
}

/// Verify a raw commit object's embedded signature.
///
/// Extracts the payload + signature, then (for gpg-based formats) writes the
/// signature to a temp file and runs `<program> --status-fd=1 <verify_args>
/// --verify <sigfile> -`, feeding the payload on stdin, and parses the
/// `[GNUPG:]` status lines.
pub fn verify_commit(cfg: &GpgConfig, raw_commit: &[u8]) -> Result<SignatureCheck> {
    let (payload, signature) = match extract_signed_payload(raw_commit) {
        Some(parts) => parts,
        None => return Ok(SignatureCheck::default_none()),
    };

    if cfg.format == GpgFormat::Ssh {
        return Err(Error::Signing(
            "ssh signature verification is not supported by grit".to_owned(),
        ));
    }

    let program = cfg.resolve_program_path()?;

    // Write the detached signature to a temp file.
    let sig_path = write_temp_file(&signature)?;

    let mut cmd = Command::new(&program);
    cmd.arg("--status-fd=1");
    for a in cfg.format.verify_args() {
        cmd.arg(a);
    }
    cmd.arg("--verify")
        .arg(&sig_path)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        let _ = std::fs::remove_file(&sig_path);
        Error::Signing(format!(
            "could not run gpg program '{}': {e}",
            program.display()
        ))
    })?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(&payload);
        drop(stdin);
    }

    let output = child.wait_with_output();
    let _ = std::fs::remove_file(&sig_path);
    let output =
        output.map_err(|e| Error::Signing(format!("failed waiting for gpg program: {e}")))?;

    // status-fd=1 routes GNUPG status to stdout; human-readable goes to stderr.
    let gpg_status = String::from_utf8_lossy(&output.stdout).into_owned();
    let human = String::from_utf8_lossy(&output.stderr).into_owned();

    let mut sigc = SignatureCheck {
        signature,
        payload,
        result: 'N',
        trust_level: TrustLevel::Undefined,
        gpg_status: gpg_status.clone(),
        output: human,
        ..Default::default()
    };

    parse_gpg_output(&mut sigc, &gpg_status);

    Ok(sigc)
}

/// Parse `[GNUPG:]` status lines into `sigc` (port of `parse_gpg_output`).
fn parse_gpg_output(sigc: &mut SignatureCheck, status: &str) {
    // (result-char, prefix, exclusive, keyid, uid, fingerprint, trust)
    struct Entry {
        result: Option<char>,
        check: &'static str,
        exclusive: bool,
        keyid: bool,
        uid: bool,
        fingerprint: bool,
        trust: bool,
    }
    const TABLE: &[Entry] = &[
        Entry {
            result: Some('G'),
            check: "GOODSIG ",
            exclusive: true,
            keyid: true,
            uid: true,
            fingerprint: false,
            trust: false,
        },
        Entry {
            result: Some('B'),
            check: "BADSIG ",
            exclusive: true,
            keyid: true,
            uid: true,
            fingerprint: false,
            trust: false,
        },
        Entry {
            result: Some('E'),
            check: "ERRSIG ",
            exclusive: true,
            keyid: true,
            uid: false,
            fingerprint: false,
            trust: false,
        },
        Entry {
            result: Some('X'),
            check: "EXPSIG ",
            exclusive: true,
            keyid: true,
            uid: true,
            fingerprint: false,
            trust: false,
        },
        Entry {
            result: Some('Y'),
            check: "EXPKEYSIG ",
            exclusive: true,
            keyid: true,
            uid: true,
            fingerprint: false,
            trust: false,
        },
        Entry {
            result: Some('R'),
            check: "REVKEYSIG ",
            exclusive: true,
            keyid: true,
            uid: true,
            fingerprint: false,
            trust: false,
        },
        Entry {
            result: None,
            check: "VALIDSIG ",
            exclusive: false,
            keyid: false,
            uid: false,
            fingerprint: true,
            trust: false,
        },
        Entry {
            result: None,
            check: "TRUST_",
            exclusive: false,
            keyid: false,
            uid: false,
            fingerprint: false,
            trust: true,
        },
    ];

    let mut seen_exclusive = false;

    for raw_line in status.lines() {
        let line = match raw_line.strip_prefix("[GNUPG:] ") {
            Some(l) => l,
            None => continue,
        };

        for entry in TABLE {
            let rest = match line.strip_prefix(entry.check) {
                Some(r) => r,
                None => continue,
            };

            if entry.exclusive {
                if seen_exclusive {
                    // Multiple exclusive statuses => multiple signatures: reject.
                    error_reset(sigc);
                    return;
                }
                seen_exclusive = true;
            }

            if let Some(r) = entry.result {
                sigc.result = r;
            }

            let mut cursor = rest;

            if entry.keyid {
                let (key, after) = split_at_space(cursor);
                sigc.key = Some(key.to_owned());
                if entry.uid && !after.is_empty() {
                    // signer is the rest of the line.
                    let signer = after.split('\n').next().unwrap_or("");
                    sigc.signer = Some(signer.to_owned());
                }
            }

            if entry.trust {
                let level: String = cursor
                    .chars()
                    .take_while(|&c| c != ' ' && c != '\n')
                    .collect();
                match TrustLevel::from_status(&level) {
                    Some(t) => sigc.trust_level = t,
                    None => {
                        error_reset(sigc);
                        return;
                    }
                }
            }

            if entry.fingerprint {
                // VALIDSIG <fingerprint> ... <primary-fingerprint>
                let (fpr, mut after) = split_at_space(cursor);
                sigc.fingerprint = Some(fpr.to_owned());
                // Skip 9 interim fields to reach the primary fingerprint.
                cursor = after;
                let mut remaining = 9;
                while remaining > 0 && !cursor.is_empty() {
                    let (_, next) = split_at_space(cursor);
                    after = next;
                    if after.is_empty() {
                        break;
                    }
                    cursor = after;
                    remaining -= 1;
                }
                if remaining == 0 {
                    let primary = cursor.split('\n').next().unwrap_or("");
                    sigc.primary_key_fingerprint = Some(primary.to_owned());
                }
            }

            break;
        }
    }
}

/// Reset `sigc` to the error state, clearing partial fields.
fn error_reset(sigc: &mut SignatureCheck) {
    sigc.result = 'E';
    sigc.primary_key_fingerprint = None;
    sigc.fingerprint = None;
    sigc.signer = None;
    sigc.key = None;
}

/// Split `s` at the first space, returning `(before, after_space)`.
fn split_at_space(s: &str) -> (&str, &str) {
    match s.find(' ') {
        Some(i) => (&s[..i], &s[i + 1..]),
        None => (s, ""),
    }
}

/// Write `data` to a fresh temp file and return its path.
fn write_temp_file(data: &[u8]) -> Result<PathBuf> {
    let dir = std::env::temp_dir();
    // Build a reasonably unique name without external crates.
    let unique = format!("git_vtag_{}_{}", std::process::id(), next_temp_counter());
    let path = dir.join(unique);
    let mut f = std::fs::File::create(&path)
        .map_err(|e| Error::Signing(format!("could not create temporary file: {e}")))?;
    f.write_all(data)
        .map_err(|e| Error::Signing(format!("failed writing detached signature: {e}")))?;
    Ok(path)
}

/// Monotonic counter used to disambiguate temp file names within a process.
fn next_temp_counter() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    now ^ COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Build the committer-info default signing key (Git's
/// `git_committer_info(IDENT_STRICT | IDENT_NO_DATE)` — "Name <email>").
///
/// `committer_ident` is a full ident line ("Name <email> <ts> <tz>"); this
/// trims the trailing timestamp/timezone.
pub fn committer_signing_default(committer_ident: &str) -> String {
    if let Some(angle_end) = committer_ident.find('>') {
        committer_ident[..=angle_end].to_owned()
    } else {
        committer_ident.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_name_is_case_sensitive() {
        assert_eq!(GpgFormat::from_name("openpgp"), Some(GpgFormat::OpenPgp));
        assert_eq!(GpgFormat::from_name("x509"), Some(GpgFormat::X509));
        assert_eq!(GpgFormat::from_name("ssh"), Some(GpgFormat::Ssh));
        assert_eq!(GpgFormat::from_name("OpEnPgP"), None);
        assert_eq!(GpgFormat::from_name("OPENPGP"), None);
    }

    #[test]
    fn add_header_signature_splices_gpgsig() {
        let commit = b"tree 0123\nparent 4567\nauthor a\ncommitter c\n\nmessage\n";
        let sig = b"-----BEGIN PGP SIGNATURE-----\nABC\n-----END PGP SIGNATURE-----\n";
        let out = add_header_signature(commit, sig, GPG_SIG_HEADER_SHA1);
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("\ncommitter c\ngpgsig -----BEGIN PGP SIGNATURE-----\n ABC\n -----END PGP SIGNATURE-----\n\nmessage\n"));
    }

    #[test]
    fn extract_round_trips_signature() {
        let commit = b"tree 0123\nparent 4567\nauthor a\ncommitter c\n\nmessage\n";
        let sig = b"-----BEGIN PGP SIGNATURE-----\nABC\n-----END PGP SIGNATURE-----\n";
        let signed = add_header_signature(commit, sig, GPG_SIG_HEADER_SHA1);
        let (payload, extracted) = extract_signed_payload(&signed).unwrap();
        assert_eq!(payload, commit);
        assert_eq!(extracted, sig);
    }

    #[test]
    fn extract_none_when_unsigned() {
        let commit = b"tree 0123\ncommitter c\n\nmsg\n";
        assert!(extract_signed_payload(commit).is_none());
    }

    #[test]
    fn parse_goodsig_and_trust() {
        let status = "\
[GNUPG:] NEWSIG\n\
[GNUPG:] GOODSIG 73D758744BE721698EC54E8713D758744BE7216 C O Mitter <committer@example.com>\n\
[GNUPG:] VALIDSIG FINGERPRINT 2010-04-01 1270074988 0 4 0 17 2 00 PRIMARYFPR\n\
[GNUPG:] TRUST_ULTIMATE 0 pgp\n";
        let mut sigc = SignatureCheck::default_none();
        parse_gpg_output(&mut sigc, status);
        assert_eq!(sigc.result, 'G');
        assert_eq!(sigc.trust_level, TrustLevel::Ultimate);
        assert_eq!(
            sigc.signer.as_deref(),
            Some("C O Mitter <committer@example.com>")
        );
        assert_eq!(
            sigc.key.as_deref(),
            Some("73D758744BE721698EC54E8713D758744BE7216")
        );
        assert!(sigc.verify_status(None));
        assert!(sigc.verify_status(Some(TrustLevel::Ultimate)));
        assert!(sigc.verify_status(Some(TrustLevel::Marginal)));
    }

    #[test]
    fn parse_badsig() {
        let status = "[GNUPG:] BADSIG KEYID Some Signer <s@example.com>\n";
        let mut sigc = SignatureCheck::default_none();
        parse_gpg_output(&mut sigc, status);
        assert_eq!(sigc.result, 'B');
        assert!(!sigc.is_good());
    }

    #[test]
    fn double_exclusive_status_is_error() {
        let status = "[GNUPG:] GOODSIG K1 A <a@x>\n[GNUPG:] BADSIG K2 B <b@x>\n";
        let mut sigc = SignatureCheck::default_none();
        parse_gpg_output(&mut sigc, status);
        assert_eq!(sigc.result, 'E');
    }

    #[test]
    fn min_trust_level_from_config_is_case_insensitive() {
        assert_eq!(
            TrustLevel::from_config("marginal"),
            Some(TrustLevel::Marginal)
        );
        assert_eq!(TrustLevel::from_config("FULLY"), Some(TrustLevel::Fully));
        assert_eq!(TrustLevel::from_config("bogus"), None);
    }
}
