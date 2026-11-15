//! pipeline：按序执行 `detect → unpack → adapt → repack → sign`，
//! 把各阶段报告汇总为 serde 可序列化的 [`PipelineReport`]。
//!
//! 执行语义：
//! - 阶段按加入顺序逐个运行；[`Stage::run`] 返回的报告原样汇入
//!   [`PipelineContext::stage_reports`]。
//! - stub 阶段返回 [`PipelineError::NotImplemented`]：记录为
//!   [`StageStatus::NotImplemented`] 报告（detail 即计划步骤），并终止后续
//!   执行——剩余阶段记录为 [`StageStatus::Pending`]。
//! - 其他错误（如 [`PipelineError::InvalidGameDir`]）：记录为
//!   [`StageStatus::Failed`]，同样终止并标记剩余阶段 Pending。

use serde::{Deserialize, Serialize};

use crate::adapt::AdaptStage;
use crate::detect::{DetectStage, Engine};
use crate::repack::RepackStage;
use crate::sign::SignStage;
use crate::stages::{PipelineContext, PipelineError, Stage, StageReport, StageStatus};
use crate::unpack::UnpackStage;

/// 五阶段移植流水线。
pub struct Pipeline {
    stages: Vec<Box<dyn Stage>>,
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline {
    /// 默认流水线：`detect → unpack → adapt → repack → sign`。
    pub fn new() -> Self {
        Self::from_stages(vec![
            Box::new(DetectStage),
            Box::new(UnpackStage),
            Box::new(AdaptStage),
            Box::new(RepackStage),
            Box::new(SignStage),
        ])
    }

    /// 以自定义阶段序列构造（测试与未来扩展用）。
    pub fn from_stages(stages: Vec<Box<dyn Stage>>) -> Self {
        Self { stages }
    }

    /// 阶段名列表（按执行顺序）。
    pub fn stage_names(&self) -> Vec<&str> {
        self.stages.iter().map(|s| s.name()).collect()
    }

    /// 执行流水线：把每份报告追加进 `ctx`，并返回汇总的 [`PipelineReport`]。
    pub fn run(&self, ctx: &mut PipelineContext) -> PipelineReport {
        let mut aborted_at: Option<String> = None;
        for stage in &self.stages {
            if let Some(failed) = &aborted_at {
                ctx.push_report(StageReport::pending(
                    stage.name(),
                    format!("因 '{failed}' 阶段失败而未执行"),
                ));
                continue;
            }
            match stage.run(ctx) {
                Ok(report) => ctx.push_report(report),
                Err(PipelineError::NotImplemented { stage, plan }) => {
                    ctx.push_report(StageReport::not_implemented(stage, plan));
                    aborted_at = Some(stage.to_owned());
                }
                Err(err) => {
                    ctx.push_report(StageReport::failed(stage.name(), err.to_string()));
                    aborted_at = Some(stage.name().to_owned());
                }
            }
        }
        PipelineReport::from_context(ctx)
    }
}

/// 流水线汇总报告（serde 可序列化，供 Tauri 前端与 CLI 输出）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PipelineReport {
    /// detect 阶段判定的引擎（未执行 detect 时为 None）。
    pub engine: Option<Engine>,
    /// 各阶段报告（按执行顺序）。
    pub stages: Vec<StageReport>,
    /// 全部阶段均为 [`StageStatus::Succeeded`] 时为 true。
    pub success: bool,
}

impl PipelineReport {
    /// 从上下文汇总：引擎取 `ctx.engine`，报告取 `ctx.stage_reports`。
    pub fn from_context(ctx: &PipelineContext) -> Self {
        let success = !ctx.stage_reports.is_empty()
            && ctx
                .stage_reports
                .iter()
                .all(|r| r.status == StageStatus::Succeeded);
        Self {
            engine: ctx.engine,
            stages: ctx.stage_reports.clone(),
            success,
        }
    }

    /// 序列化为 JSON（供 Tauri 前端与 CLI `--json` 输出）。
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}
