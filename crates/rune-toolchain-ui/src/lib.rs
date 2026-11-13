//! ruvnet Rune controls the approved Anomaly Rune artifact, not application state.
//! Each observation replays a growing input prefix on a fresh deterministic app.
//! This MVP is a bounded, offline UI pilot, not a live-session or LLM planner.
mod model;
mod process;

pub use model::{Expectation, Scenario, Step};
use rune_core::{file_digest, hex_sha256, tree_digest, Store};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Observation {
    pub index: usize,
    pub action: Option<String>,
    pub tree_sha256: String,
    pub png_sha256: String,
    pub assertions: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PilotReport {
    pub schema: String,
    pub status: String,
    pub plan_id: String,
    pub artifact_sha256: String,
    pub scenario_sha256: String,
    pub mode: String,
    pub model_calls: u32,
    pub observations: Vec<Observation>,
    pub evidence_dir: String,
    pub failure: Option<String>,
}

fn approved_artifact(workspace: &Path) -> Result<(String, String, PathBuf), String> {
    let store = Store::open(workspace);
    let plan = store.current().map_err(|e| e.to_string())?;
    let source = tree_digest(workspace).map_err(|e| e.to_string())?;
    let lock = rune_cargo::lock_digest(workspace).map_err(|e| e.to_string())?;
    let compiler = rune_cargo::toolchain_id().map_err(|e| e.to_string())?;
    plan.check_fresh(&source, &lock, &compiler)
        .map_err(|e| e.to_string())?;
    let id = plan.id();
    let receipt = store
        .receipt(&id)
        .map_err(|e| e.to_string())?
        .ok_or("no build receipt; run `rune build --freeze` first")?;
    if receipt.plan_id != id || !receipt.offline || receipt.model_calls != 0 {
        return Err("invalid or non-offline build receipt".into());
    }
    if receipt.artifact != "rune-inbox" && receipt.artifact != "rune-inbox.exe" {
        return Err(format!(
            "expected the UI product rune-inbox, got {}",
            receipt.artifact
        ));
    }
    let artifact = workspace
        .join("target/rune/release")
        .join(&receipt.artifact);
    if file_digest(&artifact).map_err(|e| e.to_string())? != receipt.artifact_digest {
        return Err("artifact digest differs from approved build receipt".into());
    }
    Ok((id, receipt.artifact_digest, artifact))
}

/// Drive the UI only after source, compiler, lockfile and binary match the frozen plan.
pub fn pilot(
    workspace: &Path,
    scenario: &Path,
    timeout_seconds: u64,
) -> Result<PilotReport, String> {
    if !(1..=60).contains(&timeout_seconds) {
        return Err("timeout must be in 1..=60 seconds".into());
    }
    let ws = fs::canonicalize(workspace).map_err(|e| e.to_string())?;
    let raw = fs::read(scenario).map_err(|e| e.to_string())?;
    if raw.len() > 64 * 1024 {
        return Err("scenario exceeds 64 KiB".into());
    }
    let scenario: Scenario = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
    scenario.validate()?;
    let (plan, digest, artifact) = approved_artifact(&ws)?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let relative = format!(".rune/ui/{}-{}-{nonce}", scenario.name, std::process::id());
    let output = ws.join(&relative);
    fs::create_dir_all(&output).map_err(|e| e.to_string())?;
    let mut report = PilotReport {
        schema: "double-rune.ui-proof/1".into(),
        status: "running".into(),
        plan_id: plan,
        artifact_sha256: digest.clone(),
        scenario_sha256: hex_sha256(&raw),
        mode: "headless-prefix-replay".into(),
        model_calls: 0,
        observations: vec![],
        evidence_dir: relative,
        failure: None,
    };
    fs::write(output.join("scenario.json"), raw).map_err(|e| e.to_string())?;
    let result: Result<(), String> = (|| {
        let mut events = Vec::<Value>::new();
        let mut tree = observe(&artifact, &digest, &output, 0, &events, timeout_seconds)?;
        model::assert_tree(&tree, &scenario.initial)?;
        record(&mut report, &output, 0, None, scenario.initial.len())?;
        for (index, step) in scenario.steps.iter().enumerate() {
            model::action_target(&tree, &step.tap)?;
            events.push(json!({"t": (index + 1) as f64 * 0.4,
                "type": "tap", "key": step.tap, "hold": 0.04}));
            tree = observe(
                &artifact,
                &digest,
                &output,
                index + 1,
                &events,
                timeout_seconds,
            )?;
            model::assert_tree(&tree, &step.expect)?;
            record(
                &mut report,
                &output,
                index + 1,
                Some(step.tap.clone()),
                step.expect.len(),
            )?;
        }
        // Catch accidental concurrent source changes rather than accepting stale evidence.
        let (current, current_digest, _) = approved_artifact(&ws)?;
        if current != report.plan_id || current_digest != report.artifact_sha256 {
            return Err("approval changed during pilot".into());
        }
        Ok(())
    })();
    match &result {
        Ok(()) => report.status = "passed".into(),
        Err(e) => {
            report.status = "rejected".into();
            report.failure = Some(e.clone());
        }
    }
    fs::write(
        output.join("proof.json"),
        serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    result.map_err(|e| format!("{e}; evidence: {}", report.evidence_dir))?;
    Ok(report)
}

fn observe(
    artifact: &Path,
    expected: &str,
    out: &Path,
    index: usize,
    events: &[Value],
    timeout: u64,
) -> Result<Value, String> {
    if file_digest(artifact).map_err(|e| e.to_string())? != expected {
        return Err("artifact changed during pilot".into());
    }
    let prefix = out.join(format!("{index:02}"));
    let script = prefix.with_extension("script.json");
    let tree = prefix.with_extension("tree.json");
    let png = prefix.with_extension("png");
    fs::write(
        &script,
        serde_json::to_vec(&json!({"events": events})).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let at = events.last().and_then(|v| v["t"].as_f64()).unwrap_or(0.0) + 0.3;
    let args = vec![
        "--script".into(),
        script.display().to_string(),
        "--tree".into(),
        tree.display().to_string(),
        "--snap".into(),
        png.display().to_string(),
        "--at".into(),
        at.to_string(),
        "--scale".into(),
        "1".into(),
    ];
    process::run(artifact, &args, &prefix, timeout)?;
    let bytes = fs::read(&png).map_err(|e| format!("missing screenshot: {e}"))?;
    if !bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Err("renderer did not produce PNG".into());
    }
    let tree: Value = serde_json::from_slice(&fs::read(&tree).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if tree["elements"].as_array().is_none() {
        return Err("missing element tree".into());
    }
    if tree["warnings"].as_array().is_some_and(|v| !v.is_empty()) {
        return Err("renderer reported duplicate/invalid element keys".into());
    }
    Ok(tree)
}

fn record(
    r: &mut PilotReport,
    out: &Path,
    index: usize,
    action: Option<String>,
    assertions: usize,
) -> Result<(), String> {
    r.observations.push(Observation {
        index,
        action,
        tree_sha256: file_digest(&out.join(format!("{index:02}.tree.json")))
            .map_err(|e| e.to_string())?,
        png_sha256: file_digest(&out.join(format!("{index:02}.png"))).map_err(|e| e.to_string())?,
        assertions,
    });
    Ok(())
}
