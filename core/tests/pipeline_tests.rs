//! pipeline 骨架测试：detect 真实执行，stub 阶段报 NotImplemented，
//! 报告汇总 serde 可序列化。

mod common;

use common::mv_fixture;
use tempfile::tempdir;
use vnporter_core::detect::{DetectStage, Engine};
use vnporter_core::pipeline::{Pipeline, PipelineReport};
use vnporter_core::stages::{PipelineContext, StageStatus};

fn detect_only_pipeline() -> Pipeline {
    Pipeline::from_stages(vec![Box::new(DetectStage)])
}

#[test]
fn default_pipeline_has_five_stages_in_order() {
    let pipeline = Pipeline::new();
    assert_eq!(
        pipeline.stage_names(),
        vec!["detect", "unpack", "adapt", "repack", "sign"]
    );
}

#[test]
fn pipeline_runs_detect_then_stubs_report_not_implemented() {
    let dir = tempdir().unwrap();
    mv_fixture(dir.path());
    let mut ctx = PipelineContext::new(dir.path());

    let report = Pipeline::new().run(&mut ctx);

    // detect 已真实执行
    assert_eq!(ctx.engine, Some(Engine::RpgMakerMv));
    let detect_report = ctx.report_of("detect").unwrap();
    assert_eq!(detect_report.status, StageStatus::Succeeded);
    assert!(detect_report.detail.contains("RPG Maker MV"));

    // 首个 stub 阶段报 NotImplemented，detail 携带记忆文件 SOP 的计划步骤
    let unpack = ctx.report_of("unpack").unwrap();
    assert_eq!(unpack.status, StageStatus::NotImplemented);
    assert!(
        unpack.detail.contains("fuyu_case_index.tsv"),
        "detail = {}",
        unpack.detail
    );

    // fail-fast：unpack 未实现即终止，其余 stub 阶段标记 Pending
    for stage in ["adapt", "repack", "sign"] {
        let r = ctx.report_of(stage).unwrap();
        assert_eq!(r.status, StageStatus::Pending, "stage = {stage}");
        assert!(
            r.detail.contains("因 'unpack' 阶段失败而未执行"),
            "stage = {stage}"
        );
    }
    // 报告齐全且按序
    let statuses: Vec<StageStatus> = report.stages.iter().map(|r| r.status).collect();
    assert_eq!(
        statuses,
        vec![
            StageStatus::Succeeded,
            StageStatus::NotImplemented,
            StageStatus::Pending,
            StageStatus::Pending,
            StageStatus::Pending,
        ]
    );
    // stub 未实现 → 整体不算成功
    assert!(!report.success);
    assert_eq!(report.engine, Some(Engine::RpgMakerMv));
}

#[test]
fn stub_stages_do_not_touch_work_dir() {
    let dir = tempdir().unwrap();
    mv_fixture(dir.path());
    let mut ctx = PipelineContext::new(dir.path());
    Pipeline::new().run(&mut ctx);
    // MVP 骨架不产生中间产物
    assert!(!ctx.work_dir.exists());
    assert_eq!(ctx.work_dir, dir.path().join(".vnporter-work"));
}

#[test]
fn detect_only_pipeline_succeeds() {
    let dir = tempdir().unwrap();
    mv_fixture(dir.path());
    let mut ctx = PipelineContext::new(dir.path());

    let report = detect_only_pipeline().run(&mut ctx);

    assert_eq!(report.stages.len(), 1);
    assert!(report.success);
    assert_eq!(report.engine, Some(Engine::RpgMakerMv));
}

#[test]
fn invalid_game_dir_fails_fast_and_marks_rest_pending() {
    let mut ctx = PipelineContext::new("/nonexistent/vnporter-game");

    let report = Pipeline::new().run(&mut ctx);

    assert_eq!(report.stages.len(), 5);
    assert_eq!(report.stages[0].status, StageStatus::Failed);
    assert!(report.stages[0].detail.contains("游戏目录无效"));
    for r in &report.stages[1..] {
        assert_eq!(r.status, StageStatus::Pending, "stage = {}", r.stage);
        assert!(r.detail.contains("因 'detect' 阶段失败而未执行"));
    }
    assert!(!report.success);
    assert_eq!(report.engine, None, "detect 失败不应写入引擎");
}

#[test]
fn empty_game_dir_detects_unknown_but_pipeline_reports_not_implemented() {
    let dir = tempdir().unwrap();
    let mut ctx = PipelineContext::new(dir.path());

    let report = Pipeline::new().run(&mut ctx);

    assert_eq!(ctx.engine, Some(Engine::Unknown));
    assert_eq!(report.stages[0].status, StageStatus::Succeeded);
    assert!(report.stages[0].detail.contains("未识别引擎"));
    assert!(!report.success);
}

#[test]
fn pipeline_report_is_serde_serializable() {
    let dir = tempdir().unwrap();
    mv_fixture(dir.path());
    let mut ctx = PipelineContext::new(dir.path());
    let report = Pipeline::new().run(&mut ctx);

    let json = report.to_json().unwrap();
    assert!(json.contains("\"rpg_maker_mv\""), "json = {json}");
    assert!(json.contains("\"not_implemented\""), "json = {json}");
    assert!(json.contains("\"succeeded\""), "json = {json}");

    let round_trip: PipelineReport = serde_json::from_str(&json).unwrap();
    assert_eq!(round_trip, report);
}
