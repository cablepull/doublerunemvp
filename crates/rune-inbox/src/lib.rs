//! The MVP uses deterministic fixtures, with all actions going through real UI handlers.
use rune::tok::{self, type_};
use rune::{Cx, El, col, el, row, text};

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub id: u32,
    pub sender: &'static str,
    pub subject: &'static str,
    pub body: &'static str,
    pub unread: bool,
}

#[derive(Clone, Debug)]
pub struct State {
    pub messages: Vec<Message>,
    pub selected: Option<u32>,
    pub archived: usize,
    pub status: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            messages: vec![
                Message {
                    id: 1,
                    sender: "Avery Chen",
                    subject: "Architecture review",
                    body: "The foundation is ready for an independent review.",
                    unread: true,
                },
                Message {
                    id: 2,
                    sender: "Sam Rivera",
                    subject: "Friday demo",
                    body: "Show the controller selecting, reading and archiving a message.",
                    unread: true,
                },
                Message {
                    id: 3,
                    sender: "Morgan Lee",
                    subject: "Build receipt",
                    body: "Every UI action should leave evidence we can verify.",
                    unread: false,
                },
            ],
            selected: None,
            archived: 0,
            status: "Ready".into(),
        }
    }
}

impl State {
    pub fn select(&mut self, id: u32) {
        if self.messages.iter().any(|m| m.id == id) {
            self.selected = Some(id);
            self.status = format!("Selected message {id}");
        }
    }
    pub fn mark_read(&mut self) {
        if let Some(m) = self
            .messages
            .iter_mut()
            .find(|m| Some(m.id) == self.selected)
        {
            m.unread = false;
            self.status = format!("Marked message {} read", m.id);
        }
    }
    pub fn archive(&mut self) {
        if let Some(id) = self.selected.take() {
            let before = self.messages.len();
            self.messages.retain(|m| m.id != id);
            if self.messages.len() < before {
                self.archived += 1;
                self.status = format!("Archived message {id}");
            }
        }
    }
}

fn button(key: &str, title: &str, action: impl Fn(&mut State) + 'static) -> El {
    let t = tok::theme(true);
    el().key(key)
        .role("button")
        .label(title)
        .focusable(true)
        .h(38.0)
        .px(16.0)
        .center()
        .radius(8.0)
        .fill(t.control_off)
        .on_press(action)
        .child(text(title).font(type_::LABEL).color(t.text))
}

pub fn view(s: &State, _: &mut Cx<State>) -> El {
    let t = tok::theme(true);
    let muted = t.text_secondary;
    let mut list = col().key("message-list").w(320.0).gap(10.0);
    for m in &s.messages {
        let id = m.id;
        let selected = s.selected == Some(id);
        let item = col()
            .key(format!("message-{id}"))
            .role("button")
            .label(format!("Open {}", m.subject))
            .focusable(true)
            .padding(16.0)
            .gap(8.0)
            .radius(10.0)
            .fill(if selected { t.control_off } else { t.card })
            .on_press(move |s: &mut State| s.select(id))
            .child(text(m.sender).font(type_::LABEL).color(muted))
            .child(text(m.subject).font(type_::BODY).color(t.text))
            .child(
                text(if m.unread { "Unread" } else { "Read" })
                    .key(format!("read-state-{id}"))
                    .font(type_::LABEL)
                    .color(muted),
            );
        list = list.child(item);
    }
    let mut detail = col()
        .key("detail")
        .grow(1.0)
        .padding(24.0)
        .gap(18.0)
        .radius(12.0)
        .fill(t.card);
    if let Some(m) = s.messages.iter().find(|m| Some(m.id) == s.selected) {
        detail = detail
            .child(
                text(m.subject)
                    .key("selected-subject")
                    .font(type_::TITLE)
                    .color(t.text),
            )
            .child(
                text(format!("From {}", m.sender))
                    .font(type_::LABEL)
                    .color(muted),
            )
            .child(text(m.body).font(type_::BODY).color(t.text))
            .child(
                row()
                    .gap(10.0)
                    .child(button("mark-read", "Mark read", State::mark_read))
                    .child(button("archive", "Archive", State::archive)),
            );
    } else {
        detail = detail
            .child(
                text("Select a message")
                    .key("empty-selection")
                    .font(type_::TITLE)
                    .color(t.text),
            )
            .child(
                text("Rune's controller pilots this interface through its element keys.")
                    .font(type_::BODY)
                    .color(muted),
            );
    }
    col()
        .key("inbox")
        .w_full()
        .h_full()
        .padding(28.0)
        .gap(20.0)
        .fill(t.bg)
        .child(
            row()
                .center_y()
                .gap(16.0)
                .child(text("Double Rune").font(type_::TITLE).color(t.text))
                .child(
                    text("MOCK INBOX / CONTROLLED BY RUNE")
                        .font(type_::LABEL)
                        .color(muted),
                ),
        )
        .child(
            row()
                .gap(18.0)
                .child(
                    text(format!("Inbox: {}", s.messages.len()))
                        .key("inbox-count")
                        .font(type_::BODY)
                        .color(t.text),
                )
                .child(
                    text(format!(
                        "Unread: {}",
                        s.messages.iter().filter(|m| m.unread).count()
                    ))
                    .key("unread-count")
                    .font(type_::BODY)
                    .color(t.text),
                )
                .child(
                    text(format!("Archived: {}", s.archived))
                        .key("archive-count")
                        .font(type_::BODY)
                        .color(t.text),
                ),
        )
        .child(
            row()
                .gap(20.0)
                .grow(1.0)
                .min_h(0.0)
                .child(list)
                .child(detail),
        )
        .child(
            row()
                .center_y()
                .gap(16.0)
                .child(
                    text(&s.status)
                        .key("status")
                        .font(type_::LABEL)
                        .color(muted)
                        .grow(1.0),
                )
                .child(button("reset", "Reset fixtures", |s| *s = State::default())),
        )
}

#[cfg(test)]
mod tests;
