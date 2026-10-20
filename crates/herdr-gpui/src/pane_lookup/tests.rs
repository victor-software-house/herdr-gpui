use super::*;

#[test]
fn a_moved_pane_answers_where_it_is_now() {
    // `herdr pane get w1:p2` after `pane move w1:p2 --tab w2:t1`.
    let reply = br#"{"id":"cli:pane:get","result":{"type":"pane_info","pane":{"pane_id":"w2:p2","workspace_id":"w2","tab_id":"w2:t1"}}}"#;
    assert_eq!(
        parse(reply),
        Lookup::Live {
            pane_id: "w2:p2".into(),
            workspace_id: "w2".into(),
        }
    );
}

#[test]
fn a_closed_pane_is_gone() {
    let reply = br#"{"error":{"code":"pane_not_found","message":"pane w7Y:p1 not found"},"id":"cli:pane:get"}"#;
    assert_eq!(parse(reply), Lookup::Gone);
}

#[test]
fn anything_else_is_no_answer() {
    for reply in [
        &b""[..],
        b"not json",
        br#"{"error":{"code":"server_unavailable"}}"#,
        br#"{"result":{"pane":{}}}"#,
    ] {
        assert_eq!(
            parse(reply),
            Lookup::Unknown,
            "{}",
            String::from_utf8_lossy(reply)
        );
    }
}
