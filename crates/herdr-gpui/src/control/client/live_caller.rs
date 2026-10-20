//! The caller's pane and workspace as its daemon sees them now.
//!
//! Herdr sets `HERDR_PANE_ID` and `HERDR_WORKSPACE_ID` once, when it starts
//! a pane's shell. `pane move` to another workspace gives the pane a new ID
//! and leaves both variables naming the old place, and a pane that closed
//! leaves an agent it started naming a pane that is gone. A tab opened from
//! those variables lands in a workspace the user is not looking at. Herdr
//! resolves a moved pane's old ID to the pane itself, so asking the daemon
//! for the caller's pane gives the workspace it is in now.

use serde_json::Value;

/// What the daemon says about the caller's pane.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Lookup {
    /// The pane lives, now as `pane_id` in `workspace_id`.
    Live {
        pane_id: String,
        workspace_id: String,
    },
    /// The pane closed: neither variable names where the caller is.
    Gone,
    /// No answer, such as no `herdr` executable or no daemon: the variables
    /// are all there is.
    Unknown,
}

/// Asks the caller's daemon, through the `herdr` CLI Herdr gave the pane,
/// where `pane_id` is now. The CLI finds the daemon from the same pane
/// variables, as it does when an agent runs it.
pub(super) fn lookup(pane_id: &str) -> Lookup {
    let program = std::env::var_os("HERDR_BIN_PATH").unwrap_or_else(|| "herdr".into());
    let Ok(output) = herdr_client::new_command(program)
        .args(["pane", "get", pane_id])
        .output()
    else {
        return Lookup::Unknown;
    };
    // The CLI prints a result on stdout and a server error on stderr.
    parse(if output.status.success() {
        &output.stdout
    } else {
        &output.stderr
    })
}

/// The answer in a `herdr pane get` reply.
pub(super) fn parse(reply: &[u8]) -> Lookup {
    let Ok(reply) = serde_json::from_slice::<Value>(reply) else {
        return Lookup::Unknown;
    };
    if reply["error"]["code"] == "pane_not_found" {
        return Lookup::Gone;
    }
    let pane = &reply["result"]["pane"];
    match (pane["pane_id"].as_str(), pane["workspace_id"].as_str()) {
        (Some(pane_id), Some(workspace_id)) if !pane_id.is_empty() && !workspace_id.is_empty() => {
            Lookup::Live {
                pane_id: pane_id.to_owned(),
                workspace_id: workspace_id.to_owned(),
            }
        }
        _ => Lookup::Unknown,
    }
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
