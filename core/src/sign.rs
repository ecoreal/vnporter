//! sign 阶段（stub）：签名 APK 并做主机侧预装校验。
//!
//! **MVP 骨架**：不实现任何真实签名逻辑，[`SignStage::run`] 返回
//! [`PipelineError::NotImplemented`]，`plan` 字段为 [`PLAN`]——逐字引用记忆
//! 文件《fuyu-rhapsody-android-port》的 SOP 措辞。
//!
//! # 计划步骤（引自记忆文件）
//!
//! 1. 签名：keystore.jks 置于项目根目录，storepass android；apksigner 签名
//!    （记忆实录：apksigner 验签 OK）。
//! 2. 主机侧预装校验：补丁文件（fuyu_fix.js/fuyu_touch.js/fuyu_preload.js/
//!    plugins/FuyuUnlock.js）与游戏源 `www` 目录逐字节一致。
//! 3. 安装门禁：HyperOS 用 `settings put global adb_install_need_confirm 0`
//!    + 重试（流式路径可过）；adb 在 `tools/platform-tools/`，不在系统 PATH；
//!    签名不兼容时自动卸载重装。
//! 4. 验证纪律（会话教训）：验证一律走 DevTools 无头 JS 求值或 HTTP 探针，
//!    **不渲染游戏画面**（浏览器里打开 CG 画廊截图曾触发内容过滤中断）。

use crate::stages::{PipelineContext, PipelineError, Result, Stage, StageReport};

/// sign 阶段的计划步骤（引自记忆文件《fuyu-rhapsody-android-port》）。
pub const PLAN: &str = "\
1. keystore.jks 置于项目根目录，storepass android，apksigner 签名并验签；
2. 主机侧预装校验：补丁文件与游戏源 www 目录逐字节一致；
3. HyperOS 安装门禁：settings put global adb_install_need_confirm 0 + 重试（流式路径可过）；adb 在 tools/platform-tools/（不在系统 PATH）；签名不兼容自动卸载重装；
4. 验证一律走 DevTools 无头 JS 求值或 HTTP 探针，不渲染游戏画面。";

/// sign 阶段（stub）。
#[derive(Debug, Default, Clone, Copy)]
pub struct SignStage;

impl Stage for SignStage {
    fn name(&self) -> &str {
        "sign"
    }

    fn run(&self, _ctx: &mut PipelineContext) -> Result<StageReport> {
        Err(PipelineError::NotImplemented {
            stage: "sign",
            plan: PLAN,
        })
    }
}
