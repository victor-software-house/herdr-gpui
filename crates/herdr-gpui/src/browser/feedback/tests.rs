use super::*;

fn key(daemon: &str, pane: &str) -> FeedbackKey {
    FeedbackKey {
        scope: Scope::local(std::path::Path::new(daemon)),
        pane_id: pane.into(),
    }
}

fn batch(pane: &str, text: &str) -> Batch {
    Batch {
        target: key("/default.sock", pane),
        text: text.into(),
    }
}

#[test]
fn batches_are_taken_once_per_pane_and_bounded() {
    let mut feedback = Feedback::default();
    let p1 = key("/default.sock", "p1");
    let p2 = key("/default.sock", "p2");
    assert!(feedback.take(&p1).is_none());
    feedback.keep(batch("p1", "one"), None);
    feedback.keep(batch("p2", "other"), None);
    feedback.keep(batch("p1", "two"), None);
    assert!(feedback.has(&p1));
    assert_eq!(feedback.take(&p1).as_deref(), Some("one\ntwo"));
    assert!(feedback.take(&p1).is_none());
    assert_eq!(feedback.take(&p2).as_deref(), Some("other"));
    for index in 0..MAX_KEPT + 2 {
        feedback.keep(batch("p", &index.to_string()), None);
    }
    assert!(
        feedback
            .take(&key("/default.sock", "p"))
            .is_some_and(|text| text.starts_with("2\n"))
    );
}

#[test]
fn kept_notes_follow_a_moved_pane() {
    let mut feedback = Feedback::default();
    feedback.keep(batch("w1:p2", "kept before the move"), None);
    feedback.keep(batch("w1:p1", "for the pane that stayed"), None);
    let scope = Scope::local(std::path::Path::new("/default.sock"));
    feedback.follow(&scope, "w1:p2", "w2:p2");
    // `browser feedback` from the moved pane asks with its new ID.
    assert_eq!(
        feedback.take(&key("/default.sock", "w2:p2")).as_deref(),
        Some("kept before the move")
    );
    assert!(feedback.take(&key("/default.sock", "w1:p2")).is_none());
    assert!(feedback.has(&key("/default.sock", "w1:p1")));
    // Another daemon's pane with the same ID is left alone.
    feedback.keep(
        Batch {
            target: key("/other.sock", "w1:p2"),
            text: "other daemon".into(),
        },
        None,
    );
    feedback.follow(&scope, "w1:p2", "w2:p2");
    assert!(feedback.has(&key("/other.sock", "w1:p2")));
}

mod session_isolation;
mod take_budget;
