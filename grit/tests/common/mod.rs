//! Shared test helpers: composable command builder, cross-check assertions,
//! and a unified-diff macro.

use similar::TextDiff;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};

const GRIT_BIN: &str = env!("CARGO_BIN_EXE_grit");

// ---------------------------------------------------------------------------
// Command result
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct Output {
    pub status: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Output {
    pub fn ok(&self) -> bool {
        self.status == Some(0)
    }

    pub fn dump(&self, label: &str) -> String {
        format!(
            "{label}: exit={:?}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            self.status, self.stdout, self.stderr
        )
    }
}

impl fmt::Debug for Output {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Output {{ status: {:?}, .. }}", self.status)
    }
}

// ---------------------------------------------------------------------------
// Diff assertion
// ---------------------------------------------------------------------------

/// Assert `left == right`, printing a unified diff on failure.
#[macro_export]
macro_rules! assert_eq_nice {
    ($left:expr, $right:expr $(,)?) => {
        assert_eq_nice!($left, $right,)
    };
    ($left:expr, $right:expr, $($arg:tt)+) => {{
        use std::borrow::Borrow;
        let left: &str = $left.borrow();
        let right: &str = $right.borrow();
        if left != right {
            let diff: TextDiff<'_, '_, '_, str> = TextDiff::from_lines(right, left);
            let diff = diff
                .unified_diff()
                .context_radius(3)
                .to_string();
            panic!(
                "assertion failed: left != right\n\
                 --- right (expected)\n\
                 +++ left (actual)\n\
                 {diff}\n\
                 {}",
                format_args!($($arg)+),
            );
        }
    }};
}

// ---------------------------------------------------------------------------
// Temp directory
// ---------------------------------------------------------------------------

pub fn unique_tmp(prefix: &str, tag: &str) -> PathBuf {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut p = std::env::temp_dir();
    p.push(format!("grit-{prefix}-{tag}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).expect("create temp dir");
    p
}

// ---------------------------------------------------------------------------
// File helpers
// ---------------------------------------------------------------------------

pub fn write_file(dir: &Path, name: &str, contents: &str) {
    std::fs::write(dir.join(name), contents).unwrap_or_else(|e| panic!("write {name}: {e}"));
}

// ---------------------------------------------------------------------------
// Low-level runner
// ---------------------------------------------------------------------------

fn run(bin: &str, args: &[String], dir: &Path) -> Output {
    let out = Command::new(bin)
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .env("GIT_AUTHOR_DATE", "1700000000 +0000")
        .env("GIT_COMMITTER_DATE", "1700000000 +0000")
        .output()
        .unwrap_or_else(|e| panic!("spawn {bin} {args:?}: {e}"));
    Output {
        status: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

// ---------------------------------------------------------------------------
// Command builder
// ---------------------------------------------------------------------------

pub struct Cmd {
    bin: String,
    args: Vec<String>,
    dir: Option<PathBuf>,
}

impl Cmd {
    pub fn in_dir(mut self, dir: &Path) -> Self {
        self.dir = Some(dir.to_path_buf());
        self
    }

    fn exec(&self) -> Output {
        let dir = self.dir.as_deref().expect("Cmd: call .in_dir() first");
        let args: Vec<String> = self.args.clone();
        run(&self.bin, &args, dir)
    }

    /// Assert exit code is 0 and return `Output`.
    pub fn suc(&self) -> Output {
        let out = self.exec();
        assert!(out.ok(), "{}\n{}", self.bin, out.dump(&self.bin));
        out
    }

    /// Cross-check: run the same args against both grit and system `git`,
    /// asserting identical exit code, stdout, and stderr.
    pub fn check(&self) {
        let dir = self.dir.as_deref().expect("Cmd: call .in_dir() first");

        let g = run(GRIT_BIN, &self.args, dir);
        let r = run("git", &self.args, dir);

        let g_ok = g.status == Some(0);
        let r_ok = r.status == Some(0);

        assert_eq!(
            g.status, r.status,
            "exit status mismatch: grit={:?} git={:?}\n\
             args: {:?}  dir: {:?}\n\n\
             --- grit stdout ---\n{}\n\
             --- git  stdout ---\n{}\n",
            g.status, r.status, self.args, dir, g.stdout, r.stdout,
        );

        if g_ok && r_ok {
            assert_eq_nice!(
                g.stdout, r.stdout,
                "stdout mismatch (both exited 0)\n\
                 args: {:?}  dir: {:?}",
                self.args, dir,
            );
            if !g.stderr.is_empty() && !r.stderr.is_empty() {
                assert_eq_nice!(
                    g.stderr, r.stderr,
                    "stderr mismatch (both exited 0)\n\
                     args: {:?}  dir: {:?}",
                    self.args, dir,
                );
            }
        }

        if !g_ok && !r_ok {
            assert_eq_nice!(
                g.stderr, r.stderr,
                "stderr mismatch (both failed)\n\
                 args: {:?}  dir: {:?}",
                self.args, dir,
            );
        }
    }
}

pub fn grit_cmd(args: &[&str]) -> Cmd {
    Cmd {
        bin: GRIT_BIN.to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
        dir: None,
    }
}

pub fn git_cmd(args: &[&str]) -> Cmd {
    Cmd {
        bin: "git".to_string(),
        args: args.iter().map(|s| s.to_string()).collect(),
        dir: None,
    }
}
