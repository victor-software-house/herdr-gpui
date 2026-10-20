use super::*;

fn some(id: &str) -> Option<String> {
    Some(id.to_owned())
}

#[test]
fn a_moved_pane_opens_in_the_workspace_it_is_in_now() {
    // The agent's shell still has the variables of the workspace it started
    // in; the daemon resolves the old pane ID to the pane in its new place.
    let live = Lookup::Live {
        pane_id: "w86:p3".into(),
        workspace_id: "w86".into(),
    };
    assert_eq!(
        place(some("w85:p2"), some("w85"), None, live),
        (some("w86:p3"), some("w86"))
    );
}

#[test]
fn a_closed_pane_names_no_workspace_so_the_frontmost_window_opens_it() {
    assert_eq!(
        place(some("w7Y:p1"), some("w7Y"), None, Lookup::Gone),
        (None, None)
    );
}

#[test]
fn a_named_workspace_wins_over_the_daemon_and_the_variables() {
    let live = Lookup::Live {
        pane_id: "w86:p3".into(),
        workspace_id: "w86".into(),
    };
    assert_eq!(
        place(some("w85:p2"), some("w85"), some("w8E"), live),
        (some("w86:p3"), some("w8E"))
    );
    assert_eq!(
        place(some("w7Y:p1"), some("w7Y"), some("w8E"), Lookup::Gone),
        (None, some("w8E"))
    );
}

#[test]
fn without_an_answer_the_variables_still_place_the_tab() {
    assert_eq!(
        place(some("w1:p1"), some("w1"), None, Lookup::Unknown),
        (some("w1:p1"), some("w1"))
    );
}
