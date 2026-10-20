//! The caller's pane and workspace as its daemon sees them now.
//!
//! Herdr sets `HERDR_PANE_ID` and `HERDR_WORKSPACE_ID` once, when it starts
//! a pane's shell. `pane move` to another workspace gives the pane a new ID
//! and leaves both variables naming the old place, and a pane that closed
//! leaves an agent it started naming a pane that is gone. A tab opened from
//! those variables lands in a workspace the user is not looking at. Herdr
//! resolves a moved pane's old ID to the pane itself, so asking the daemon
//! for the caller's pane gives the workspace it is in now.

pub(super) use crate::pane_lookup::Lookup;

/// Asks the caller's daemon, through the `herdr` CLI Herdr gave the pane,
/// where `pane_id` is now. The CLI finds the daemon from the same pane
/// variables, as it does when an agent runs it.
pub(super) fn lookup(pane_id: &str) -> Lookup {
    let program = std::env::var_os("HERDR_BIN_PATH").unwrap_or_else(|| "herdr".into());
    crate::pane_lookup::ask(herdr_client::new_command(program), pane_id)
}

/// The pane and workspace a request carries. A `--workspace` the caller
/// named wins. Otherwise the daemon's answer replaces the variables, and a
/// pane that closed leaves neither, so the request acts as one from outside
/// Herdr does: in the frontmost window.
pub(super) fn place(
    pane_var: Option<String>,
    workspace_var: Option<String>,
    named: Option<String>,
    lookup: Lookup,
) -> (Option<String>, Option<String>) {
    let (pane_id, workspace_id) = match lookup {
        Lookup::Live {
            pane_id,
            workspace_id,
        } => (Some(pane_id), Some(workspace_id)),
        Lookup::Gone => (None, None),
        Lookup::Unknown => (pane_var, workspace_var),
    };
    (pane_id, named.or(workspace_id))
}

#[cfg(test)]
mod tests;
