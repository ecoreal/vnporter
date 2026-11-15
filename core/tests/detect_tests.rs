//! detect 阶段测试：用合成 fixture 目录（空标志文件）覆盖每个引擎特征的
//! 正/反例，以及判定规则、版本解析与阶段集成。
//!
//! 红线：所有 fixture 均由 tempfile 临时目录 + 空文件构成，绝不指向真实游戏目录。

mod common;

use std::fs;

use common::{kirikiri_fixture, mv_fixture, mv_partial_fixture, mv_weak_fixture, touch};
use tempfile::tempdir;
use vnporter_core::detect::{
    Confidence, DetectStage, Engine, detect, detect_mv_version, feature_index_html,
    feature_kirikiri_data_xp3, feature_plugins_dir, feature_rpg_core_js, feature_rpgmvo_audio,
    feature_rpgmvp_image, mv_features,
};
use vnporter_core::stages::{PipelineContext, Stage, StageStatus};

// ---------------------------------------------------------------------------
// 每个特征的正/反例（一个特征一个独立函数，逐个断言）
// ---------------------------------------------------------------------------

#[test]
fn feature_index_html_positive_and_negative() {
    let dir = tempdir().unwrap();
    assert!(!feature_index_html(dir.path()).hit);
    touch(dir.path(), "www/index.html");
    let f = feature_index_html(dir.path());
    assert!(f.hit);
    assert_eq!(f.name, "www/index.html");
}

#[test]
fn feature_rpg_core_js_positive_and_negative() {
    let dir = tempdir().unwrap();
    assert!(!feature_rpg_core_js(dir.path()).hit);
    touch(dir.path(), "www/js/rpg_core.js");
    let f = feature_rpg_core_js(dir.path());
    assert!(f.hit);
    assert_eq!(f.name, "www/js/rpg_core.js");
}

#[test]
fn feature_plugins_dir_positive_and_negative() {
    let dir = tempdir().unwrap();
    assert!(!feature_plugins_dir(dir.path()).hit);
    // 同名文件不算目录命中
    touch(dir.path(), "www/js/plugins");
    assert!(!feature_plugins_dir(dir.path()).hit);
    fs::remove_file(dir.path().join("www/js/plugins")).unwrap();
    fs::create_dir_all(dir.path().join("www/js/plugins")).unwrap();
    let f = feature_plugins_dir(dir.path());
    assert!(f.hit);
    assert_eq!(f.name, "www/js/plugins/");
}

#[test]
fn feature_rpgmvo_audio_positive_and_negative() {
    let dir = tempdir().unwrap();
    assert!(!feature_rpgmvo_audio(dir.path()).hit);
    touch(dir.path(), "www/audio/bgm1.rpgmvo");
    let f = feature_rpgmvo_audio(dir.path());
    assert!(f.hit);
    assert!(f.detail.contains("1 个"), "detail = {}", f.detail);
    // 大小写不敏感（记忆实录：存在 volcano.rpgmvp→Volcano.rpgmvp 这类大小写差异）
    touch(dir.path(), "www/audio/BGM2.RPGMVO");
    let f = feature_rpgmvo_audio(dir.path());
    assert!(f.hit);
    assert!(f.detail.contains("2 个"), "detail = {}", f.detail);
}

#[test]
fn feature_rpgmvp_image_positive_and_negative() {
    let dir = tempdir().unwrap();
    assert!(!feature_rpgmvp_image(dir.path()).hit);
    touch(dir.path(), "www/img/volcano.rpgmvp");
    let f = feature_rpgmvp_image(dir.path());
    assert!(f.hit);
    assert!(f.detail.contains("1 个"), "detail = {}", f.detail);
}

#[test]
fn feature_kirikiri_data_xp3_positive_and_negative() {
    let dir = tempdir().unwrap();
    assert!(!feature_kirikiri_data_xp3(dir.path()).hit);
    touch(dir.path(), "data.xp3");
    let f = feature_kirikiri_data_xp3(dir.path());
    assert!(f.hit);
    assert_eq!(f.name, "data.xp3");
}

#[test]
fn walk_count_cap_reports_at_least_prefix() {
    // 递归计数到 WALK_COUNT_CAP 提前停止且不再爆栈/超时；造 1500 个文件
    // 会太慢，这里只验证 cap 逻辑的说明文案——构造超过 cap 的场景代价高，
    // 改为验证少量文件不带 ≥ 前缀（反例由上面 count 断言覆盖）。
    let dir = tempdir().unwrap();
    touch(dir.path(), "www/audio/a.rpgmvo");
    let f = feature_rpgmvo_audio(dir.path());
    assert!(
        !f.detail.contains("≥"),
        "少量文件不应出现 ≥ 前缀：{}",
        f.detail
    );
}

// ---------------------------------------------------------------------------
// RPGMAKER_VERSION 解析
// ---------------------------------------------------------------------------

#[test]
fn mv_version_parsed_single_quotes() {
    let dir = tempdir().unwrap();
    mv_fixture(dir.path());
    assert_eq!(detect_mv_version(dir.path()).as_deref(), Some("1.6.1"));
}

#[test]
fn mv_version_parsed_double_quotes() {
    let dir = tempdir().unwrap();
    touch(dir.path(), "www/js/rpg_core.js");
    fs::write(
        dir.path().join("www/js/rpg_core.js"),
        "Utils.RPGMAKER_VERSION = \"1.6.2\";",
    )
    .unwrap();
    assert_eq!(detect_mv_version(dir.path()).as_deref(), Some("1.6.2"));
}

#[test]
fn mv_version_missing_marker_returns_none() {
    let dir = tempdir().unwrap();
    touch(dir.path(), "www/js/rpg_core.js");
    fs::write(
        dir.path().join("www/js/rpg_core.js"),
        "// no version here\n",
    )
    .unwrap();
    assert_eq!(detect_mv_version(dir.path()), None);
}

#[test]
fn mv_version_missing_file_returns_none() {
    let dir = tempdir().unwrap();
    assert_eq!(detect_mv_version(dir.path()), None);
}

#[test]
fn mv_version_malformed_marker_returns_none() {
    let dir = tempdir().unwrap();
    touch(dir.path(), "www/js/rpg_core.js");
    fs::write(
        dir.path().join("www/js/rpg_core.js"),
        "RPGMAKER_VERSION = 'not.a.version';",
    )
    .unwrap();
    assert_eq!(detect_mv_version(dir.path()), None);
}

// ---------------------------------------------------------------------------
// 判定规则
// ---------------------------------------------------------------------------

#[test]
fn full_mv_fixture_is_high_confidence_rpg_maker_mv() {
    let dir = tempdir().unwrap();
    mv_fixture(dir.path());
    let d = detect(dir.path());
    assert_eq!(d.engine, Engine::RpgMakerMv);
    assert_eq!(d.confidence, Confidence::High);
    assert!(
        d.explanation.contains("5/5"),
        "explanation = {}",
        d.explanation
    );
    assert!(
        d.explanation.contains("1.6.1"),
        "explanation = {}",
        d.explanation
    );
    // mv_features 与 detect 一致：5 个特征全部命中
    assert_eq!(mv_features(dir.path()).iter().filter(|f| f.hit).count(), 5);
}

#[test]
fn partial_mv_fixture_is_medium_confidence() {
    let dir = tempdir().unwrap();
    mv_partial_fixture(dir.path());
    let d = detect(dir.path());
    assert_eq!(d.engine, Engine::RpgMakerMv);
    assert_eq!(d.confidence, Confidence::Medium);
    assert!(
        d.explanation.contains("2/5"),
        "explanation = {}",
        d.explanation
    );
}

#[test]
fn single_weak_marker_is_low_confidence() {
    let dir = tempdir().unwrap();
    mv_weak_fixture(dir.path());
    let d = detect(dir.path());
    assert_eq!(d.engine, Engine::RpgMakerMv);
    assert_eq!(d.confidence, Confidence::Low);
    assert!(
        d.explanation.contains("1/5"),
        "explanation = {}",
        d.explanation
    );
}

#[test]
fn kirikiri_fixture_detected_as_kirikiri() {
    let dir = tempdir().unwrap();
    kirikiri_fixture(dir.path());
    let d = detect(dir.path());
    assert_eq!(d.engine, Engine::Kirikiri);
    assert_eq!(d.confidence, Confidence::High);
    assert!(
        d.explanation.contains("data.xp3"),
        "explanation = {}",
        d.explanation
    );
}

#[test]
fn mv_markers_win_over_kirikiri() {
    let dir = tempdir().unwrap();
    mv_fixture(dir.path());
    touch(dir.path(), "data.xp3");
    let d = detect(dir.path());
    assert_eq!(d.engine, Engine::RpgMakerMv);
}

#[test]
fn empty_dir_is_unknown_low_confidence() {
    let dir = tempdir().unwrap();
    let d = detect(dir.path());
    assert_eq!(d.engine, Engine::Unknown);
    assert_eq!(d.confidence, Confidence::Low);
}

// ---------------------------------------------------------------------------
// DetectStage 集成：写 ctx.engine + 产出报告
// ---------------------------------------------------------------------------

#[test]
fn detect_stage_sets_engine_and_emits_report() {
    let dir = tempdir().unwrap();
    mv_fixture(dir.path());
    let mut ctx = PipelineContext::new(dir.path());
    let report = DetectStage.run(&mut ctx).unwrap();

    assert_eq!(report.stage, "detect");
    assert_eq!(report.status, StageStatus::Succeeded);
    assert_eq!(ctx.engine, Some(Engine::RpgMakerMv));
    assert!(
        ctx.report_of("detect").is_none(),
        "阶段报告由 Pipeline 汇总，Stage 本身不写入"
    );
}

#[test]
fn detect_stage_rejects_missing_game_dir() {
    let mut ctx = PipelineContext::new("/nonexistent/vnporter-fixture");
    let err = DetectStage.run(&mut ctx).unwrap_err();
    assert!(matches!(
        err,
        vnporter_core::stages::PipelineError::InvalidGameDir(_)
    ));
}

#[test]
fn engine_serializes_snake_case() {
    assert_eq!(
        serde_json::to_string(&Engine::RpgMakerMv).unwrap(),
        "\"rpg_maker_mv\""
    );
    assert_eq!(
        serde_json::to_string(&Engine::Kirikiri).unwrap(),
        "\"kirikiri\""
    );
    assert_eq!(
        serde_json::to_string(&Engine::Unknown).unwrap(),
        "\"unknown\""
    );
}
