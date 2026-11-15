//! 阶段契约：[`Stage`] trait、[`PipelineContext`]、[`StageReport`] 与错误类型。
//!
//! 五阶段共享同一契约：拿到 `&mut PipelineContext`，产出 [`StageReport`]。
//! 中间产物统一写入 `ctx.work_dir`（MVP 的 stub 阶段不会实际创建它）。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::detect::Engine;

/// 流水线统一错误类型。
#[derive(Debug, Error)]
pub enum PipelineError {
    /// stub 阶段的占位错误：`plan` 携带引自记忆文件 SOP 的计划步骤。
    #[error("阶段 '{stage}' 尚未实现（MVP 骨架）。计划步骤：{plan}")]
    NotImplemented {
        /// 阶段名（如 "unpack"）。
        stage: &'static str,
        /// 引自记忆文件《fuyu-rhapsody-android-port》SOP 的计划步骤。
        plan: &'static str,
    },
    /// 游戏目录不存在或不是目录。
    #[error("游戏目录无效：{0}")]
    InvalidGameDir(String),
    /// 文件系统错误。
    #[error("I/O 错误：{0}")]
    Io(#[from] std::io::Error),
}

/// 各阶段统一使用的 `Result` 别名。
pub type Result<T> = std::result::Result<T, PipelineError>;

/// 阶段执行状态（serde 序列化为 snake_case，如 `"not_implemented"`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    /// 阶段成功完成。
    Succeeded,
    /// stub 阶段尚未实现；detail 为计划步骤（引自记忆文件 SOP）。
    NotImplemented,
    /// 阶段执行失败；detail 为错误说明。
    Failed,
    /// 因前置阶段失败而未执行。
    Pending,
}

/// 单个阶段的执行报告，最终汇入 [`PipelineReport`](crate::pipeline::PipelineReport)。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageReport {
    /// 阶段名。
    pub stage: String,
    /// 执行状态。
    pub status: StageStatus,
    /// 说明：成功时为结果摘要（如 detect 的置信度说明），
    /// NotImplemented 时为计划步骤，Failed/Pending 时为原因。
    pub detail: String,
}

impl StageReport {
    /// 构造成功报告。
    pub fn succeeded(stage: &str, detail: impl Into<String>) -> Self {
        Self {
            stage: stage.to_owned(),
            status: StageStatus::Succeeded,
            detail: detail.into(),
        }
    }

    /// 构造 stub 未实现报告。
    pub fn not_implemented(stage: &str, detail: impl Into<String>) -> Self {
        Self {
            stage: stage.to_owned(),
            status: StageStatus::NotImplemented,
            detail: detail.into(),
        }
    }

    /// 构造失败报告。
    pub fn failed(stage: &str, detail: impl Into<String>) -> Self {
        Self {
            stage: stage.to_owned(),
            status: StageStatus::Failed,
            detail: detail.into(),
        }
    }

    /// 构造因前置失败而跳过的报告。
    pub fn pending(stage: &str, detail: impl Into<String>) -> Self {
        Self {
            stage: stage.to_owned(),
            status: StageStatus::Pending,
            detail: detail.into(),
        }
    }
}

/// 流水线上下文：游戏目录、中间产物目录、引擎判定与各阶段报告。
#[derive(Debug, Clone)]
pub struct PipelineContext {
    /// 待移植的游戏目录。
    ///
    /// 红线：真实运行中只允许指向用户显式提供的游戏目录；测试一律使用
    /// 合成 fixture（空标志文件），绝不指向真实游戏目录。
    pub game_dir: PathBuf,
    /// 中间产物目录（解包/适配/重封阶段的工作区），默认为
    /// `<game_dir>/.vnporter-work`。
    pub work_dir: PathBuf,
    /// detect 阶段写入的引擎判定结果。
    pub engine: Option<Engine>,
    /// 各阶段报告（按执行顺序追加）。
    pub stage_reports: Vec<StageReport>,
}

impl PipelineContext {
    /// 以默认中间产物目录 `<game_dir>/.vnporter-work` 构造上下文。
    pub fn new(game_dir: impl Into<PathBuf>) -> Self {
        let game_dir = game_dir.into();
        let work_dir = game_dir.join(".vnporter-work");
        Self {
            game_dir,
            work_dir,
            engine: None,
            stage_reports: Vec::new(),
        }
    }

    /// 覆盖中间产物目录。
    pub fn with_work_dir(mut self, work_dir: impl Into<PathBuf>) -> Self {
        self.work_dir = work_dir.into();
        self
    }

    /// 追加一条阶段报告。
    pub fn push_report(&mut self, report: StageReport) {
        self.stage_reports.push(report);
    }

    /// 按阶段名查找报告。
    pub fn report_of(&self, stage: &str) -> Option<&StageReport> {
        self.stage_reports.iter().find(|r| r.stage == stage)
    }

    /// 校验游戏目录存在且为目录（detect 阶段入口检查）。
    pub fn ensure_game_dir(&self) -> Result<()> {
        if self.game_dir.is_dir() {
            Ok(())
        } else {
            Err(PipelineError::InvalidGameDir(
                self.game_dir.display().to_string(),
            ))
        }
    }

    /// 校验中间产物目录存在且为目录（供后续真实阶段使用）。
    pub fn ensure_work_dir(&self) -> Result<()> {
        if self.work_dir.is_dir() {
            Ok(())
        } else {
            Err(PipelineError::InvalidGameDir(
                self.work_dir.display().to_string(),
            ))
        }
    }
}

/// 流水线阶段契约。
///
/// `run` 拿到 `&mut PipelineContext`（可读取游戏目录、写入引擎判定等中间
/// 状态），返回该阶段的 [`StageReport`]；stub 阶段返回
/// [`PipelineError::NotImplemented`]，由 [`Pipeline`](crate::Pipeline) 汇总
/// 为 NotImplemented 状态的报告。
pub trait Stage {
    /// 阶段名（如 "detect"、"unpack"）。
    fn name(&self) -> &str;

    /// 执行阶段。
    fn run(&self, ctx: &mut PipelineContext) -> Result<StageReport>;
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn context_defaults_work_dir_under_game_dir() {
        let ctx = PipelineContext::new("/tmp/fake-game");
        assert_eq!(ctx.work_dir, Path::new("/tmp/fake-game/.vnporter-work"));
        assert!(ctx.engine.is_none());
        assert!(ctx.stage_reports.is_empty());
    }

    #[test]
    fn context_with_work_dir_overrides_default() {
        let ctx = PipelineContext::new("/tmp/fake-game").with_work_dir("/tmp/work");
        assert_eq!(ctx.work_dir, Path::new("/tmp/work"));
    }

    #[test]
    fn report_lookup_by_stage_name() {
        let mut ctx = PipelineContext::new("/tmp/fake-game");
        ctx.push_report(StageReport::succeeded("detect", "ok"));
        assert!(ctx.report_of("detect").is_some());
        assert!(ctx.report_of("unpack").is_none());
    }

    #[test]
    fn stage_status_serializes_snake_case() {
        assert_eq!(
            serde_json::to_string(&StageStatus::NotImplemented).unwrap(),
            "\"not_implemented\""
        );
    }
}
