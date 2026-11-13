use super::*;
use rune::{Env, Runtime};

#[test]
fn actual_ui_handlers_select_read_and_archive_only_the_selected_message() {
    let mut rt = Runtime::new(
        State::default(),
        view,
        (1040.0, 620.0),
        Env {
            system_dark: true,
            scale: 1.0,
        },
    );
    rt.frame(0.0);
    for key in ["message-2", "mark-read", "archive"] {
        let (x, y) = rt.find(key).unwrap_or_else(|| panic!("missing {key}"));
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(1.0 / 60.0);
    }
    assert_eq!(
        rt.state.messages.iter().map(|m| m.id).collect::<Vec<_>>(),
        [1, 3]
    );
    assert_eq!(rt.state.archived, 1);
    assert_eq!(rt.state.selected, None);
    assert_eq!(rt.state.status, "Archived message 2");
    assert!(
        rt.state.messages[0].unread,
        "unrelated message must remain unread"
    );
}

#[test]
fn actions_without_selection_do_not_change_the_inbox() {
    let mut s = State::default();
    s.archive();
    s.mark_read();
    s.select(999);
    assert_eq!(s.messages.len(), 3);
    assert_eq!(s.archived, 0);
    assert_eq!(s.selected, None);
}
