//! A tab keeps the pane whose agent opened it, where its notes go and which
//! `browser reload` and `browser feedback` find it by. `pane move` gives a
//! pane a new ID and a closed pane has none, so an origin missing from the
//! daemon's panes is asked about: the daemon still resolves a moved pane's
//! earlier IDs, and the tab follows the pane; a closed pane's tab keeps no
//! agent, so its notes are copied rather than kept for a pane no agent can
//! ask for. Tabs of SSH hosts keep the origin they were opened with.

use super::{scope, store};
use crate::{
    HerdrWindow,
    browser::{Feedback, Store},
    pane_lookup::{self, Lookup},
};
use gpui::*;
use std::{collections::HashSet, sync::atomic::AtomicBool};

impl HerdrWindow {
    pub(super) fn follow_tab_origins(&mut self, cx: &mut Context<Self>) {
        let Some(snapshot) = self.live.snapshot.as_ref() else {
            return;
        };
        let target = &self.endpoints[self.selected_endpoint].connection.target;
        // Asking needs the host scripts that moving a pane uses, which need
        // a POSIX client.
        if target.is_remote() || crate::pane_move::refusal(target).is_some() {
            return;
        }
        let scope = scope(&self.endpoints[self.selected_endpoint]);
        let panes: HashSet<&str> = snapshot
            .panes
            .iter()
            .map(|pane| pane.pane_id.as_str())
            .collect();
        let Some(store) = store(cx) else {
            return;
        };
        // Each pane is asked about once per daemon boot: an origin with no
        // answer keeps today's behaviour rather than asking every frame.
        let boot = &snapshot.boot_id;
        let lost: Vec<String> = store
            .lost_origins(&scope, &panes)
            .into_iter()
            .filter(|pane| {
                self.browser
                    .origin_lookups
                    .insert((boot.clone(), pane.clone()))
            })
            .collect();
        if lost.is_empty() {
            return;
        }
        let Ok(host) = crate::pane_move::host_for(target) else {
            return;
        };
        // The CLI is a child process: never on this thread.
        let answers = cx.background_executor().spawn(async move {
            let cancelled = AtomicBool::new(false);
            lost.into_iter()
                .map(|pane| {
                    let lookup = pane_lookup::ask_host(&host, &pane, &cancelled);
                    (pane, lookup)
                })
                .collect::<Vec<_>>()
        });
        cx.spawn(async move |this, cx| {
            let answers = answers.await;
            let _ = this.update(cx, |_, cx| {
                for (pane, lookup) in &answers {
                    let to = match lookup {
                        Lookup::Live { pane_id, .. } => {
                            // Notes kept for the pane before it moved wait
                            // for it at its new ID.
                            cx.default_global::<Feedback>()
                                .follow(&scope, pane, pane_id);
                            Some(pane_id.as_str())
                        }
                        Lookup::Gone => None,
                        Lookup::Unknown => continue,
                    };
                    if Store::update(cx, |store| store.follow_origin(&scope, pane, to)) {
                        cx.notify();
                    }
                }
            });
        })
        .detach();
    }
}
