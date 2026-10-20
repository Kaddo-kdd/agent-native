//! Compilation context — explicit WIT resolution root, no cwd dependency.
//!
//! Introduced by AN-016 to eliminate the `std::fs::read_to_string(p)`
//! `.or_else("../../{p}")` fallback chain in [`crate::codegen`]. The
//! context declares WHERE WIT files live, so the compiler produces the
//! same output regardless of the process's current working directory.
//!
//! `compile_ir_to_wasm` remains available as a backwards-compatible
//! shortcut that constructs a context anchored at the current working
//! directory. New call sites should prefer
//! [`crate::compile_ir_to_wasm_with_context`].

use std::io;
use std::path::{Path, PathBuf};

/// A compilation context carrying the roots the compiler needs to resolve
/// external references (currently only WIT files) and experimental version flags.
#[derive(Debug, Clone)]
pub struct CompileContext {
    wit_root: PathBuf,
    allow_experimental: bool,
}

impl CompileContext {
    /// Builds a context anchored at an explicit WIT root.
    ///
    /// `wit_root` is the directory relative paths in
    /// `wit_contract.wit_file` resolve against. Absolute paths take
    /// precedence over the root.
    pub fn new(wit_root: impl Into<PathBuf>) -> Self {
        Self {
            wit_root: wit_root.into(),
            allow_experimental: false,
        }
    }

    /// Enables or disables experimental Semantic IR v0.6 compilation (`--experimental`).
    #[must_use]
    pub fn with_experimental(mut self, allow_experimental: bool) -> Self {
        self.allow_experimental = allow_experimental;
        self
    }

    /// Returns `true` if experimental IR v0.6 compilation is enabled.
    #[must_use]
    pub const fn allow_experimental(&self) -> bool {
        self.allow_experimental
    }

    /// Builds a context anchored at the current working directory.
    ///
    /// This matches the pre-AN-016 behaviour of `compile_ir_to_wasm` and
    /// is used by that function to preserve byte-identical output for the
    /// existing test suites and fixtures.
    ///
    /// # Errors
    ///
    /// Returns the underlying `std::io::Error` if the cwd cannot be read.
    pub fn cwd() -> io::Result<Self> {
        Ok(Self {
            wit_root: std::env::current_dir()?,
            allow_experimental: false,
        })
    }

    /// Returns the configured WIT root.
    #[must_use]
    pub fn wit_root(&self) -> &Path {
        &self.wit_root
    }

    /// Resolves a reference to a WIT file. Absolute paths are returned
    /// as-is when they exist; relative paths are tried first as given
    /// (preserving caller intent) and then joined onto [`Self::wit_root`].
    /// Returns `None` when neither variant exists on disk, so the caller
    /// can emit a precise `E_WIT_NOT_FOUND` diagnostic with the paths
    /// probed.
    ///
    /// This method is deliberately strict: there are no
    /// `../../` fallbacks. If the configured root is wrong, compilation
    /// fails with a diagnostic rather than silently reaching for a
    /// sibling directory.
    #[must_use]
    pub fn resolve(&self, path_hint: &str) -> ResolveOutcome {
        let hint = Path::new(path_hint);
        if hint.is_absolute() {
            if hint.exists() {
                return ResolveOutcome::Found(hint.to_path_buf());
            }
            return ResolveOutcome::NotFound {
                tried: vec![hint.to_path_buf()],
            };
        }
        // Relative paths resolve against `wit_root` ONLY — never against
        // the process cwd. This is the invariant AN-016 establishes:
        // reproducible compilation regardless of where the build runs.
        let joined = self.wit_root.join(hint);
        if joined.exists() {
            return ResolveOutcome::Found(joined);
        }
        ResolveOutcome::NotFound {
            tried: vec![joined],
        }
    }
}

impl Default for CompileContext {
    /// Mirrors [`CompileContext::cwd`]; falls back to an empty root if the
    /// cwd cannot be read (never happens on supported platforms, but
    /// guarantees `Default` cannot panic).
    fn default() -> Self {
        Self::cwd().unwrap_or_else(|_| Self {
            wit_root: PathBuf::new(),
            allow_experimental: false,
        })
    }
}

/// Result of attempting to resolve a WIT file reference.
#[derive(Debug)]
pub enum ResolveOutcome {
    Found(PathBuf),
    NotFound { tried: Vec<PathBuf> },
}
