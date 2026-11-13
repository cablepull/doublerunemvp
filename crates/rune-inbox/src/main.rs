fn main() {
    rune::run(rune::AppSpec::new(
        "Double Rune — Inbox MVP",
        (1040.0, 620.0),
        rune_inbox::State::default,
        rune_inbox::view,
    ));
}
