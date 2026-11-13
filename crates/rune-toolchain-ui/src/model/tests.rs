use super::*;
use serde_json::json;

fn tree() -> Value {
    json!({"elements":[{"key":"archive","role":"button","inert":false,
        "opacity":1,"rect":[0,0,100,30],"text":"Archive"}]})
}

#[test]
fn rejects_missing_ambiguous_and_inert_targets() {
    assert!(action_target(&tree(), "archive").is_ok());
    assert!(action_target(&tree(), "missing").is_err());
    let mut t = tree();
    t["elements"][0]["inert"] = json!(true);
    assert!(action_target(&t, "archive").is_err());
    let mut t = tree();
    let extra = t["elements"][0].clone();
    t["elements"].as_array_mut().unwrap().push(extra);
    assert!(action_target(&t, "archive").is_err());
}

#[test]
fn wrong_postconditions_cannot_pass() {
    assert!(assert_tree(
        &tree(),
        &[Expectation::Text {
            key: "archive".into(),
            value: "wrong".into()
        }]
    )
    .is_err());
    assert!(assert_tree(
        &tree(),
        &[Expectation::Absent {
            key: "archive".into()
        }]
    )
    .is_err());
    assert!(assert_tree(&tree(), &[Expectation::Absent { key: "gone".into() }]).is_ok());
}

#[test]
fn bounds_actions_and_requires_assertions() {
    let mut s: Scenario = serde_json::from_value(json!({"schema":"double-rune.scenario/1",
        "name":"safe","initial":[{"kind":"present","key":"archive"}],
        "steps":[{"tap":"archive","expect":[{"kind":"absent","key":"gone"}]}]}))
    .unwrap();
    assert!(s.validate().is_ok());
    s.name = "../escape".into();
    assert!(s.validate().is_err());
    s.name = "safe".into();
    s.steps[0].expect.clear();
    assert!(s.validate().is_err());
}
