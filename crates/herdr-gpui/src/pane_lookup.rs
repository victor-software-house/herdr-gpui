//! What a daemon says about a pane ID, asked through `herdr pane get`.
//!
//! `pane move` to another workspace gives a pane a new ID. Herdr still
//! resolves each earlier ID to the pane where it is now, through any number
//! of moves, and answers `pane_not_found` once the pane closed.

use serde_json::Value;
use std::{process::Command, sync::atomic::AtomicBool};

/// What the daemon says about a pane.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Lookup {
    /// The pane lives, now as `pane_id` in `workspace_id`.
    Live {
        pane_id: String,
        workspace_id: String,
    },
    /// The pane closed.
    Gone,
    /// No answer, such as no `herdr` executable or no daemon.
    Unknown,
}

/// Runs `command`, a `herdr` CLI aimed at the daemon, as `pane get
/// pane_id`.
pub(crate) fn ask(mut command: Command, pane_id: &str) -> Lookup {
    let Ok(output) = command.args(["pane", "get", pane_id]).output() else {
        return Lookup::Unknown;
    };
    // The CLI prints a result on stdout and a server error on stderr.
    parse(if output.status.success() {
        &output.stdout
    } else {
        &output.stderr
    })
}

/// Asks the daemon `host` scripts reach, as moving a pane does. Blocking:
/// call only on a background worker.
pub(crate) fn ask_host(
    host: &crate::teleport::Host,
    pane_id: &str,
    cancelled: &AtomicBool,
) -> Lookup {
    // The CLI would read a leading dash as an option.
    if pane_id.starts_with('-') {
        return Lookup::Unknown;
    }
    let line = crate::teleport::Host::herdr_line(&["pane", "get", pane_id]);
    match host.capture(&line, cancelled) {
        Ok(stdout) => parse(&stdout),
        // The CLI prints its error envelope as the last stderr line.
        Err(herdr_client::Error::ScriptExit { stderr, .. }) => stderr
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map_or(Lookup::Unknown, |line| parse(line.trim().as_bytes())),
        Err(_) => Lookup::Unknown,
    }
}

/// The answer in a `herdr pane get` reply.
pub(crate) fn parse(reply: &[u8]) -> Lookup {
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

#[cfg(test)]
mod tests;
