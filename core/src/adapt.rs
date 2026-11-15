//! adapt 阶段（stub）：把解包产物适配为安卓 WebView 工程（注入修正补丁、
//! 生成 Android 工程配置）。
//!
//! **MVP 骨架**：不实现任何真实适配逻辑，[`AdaptStage::run`] 返回
//! [`PipelineError::NotImplemented`]，`plan` 字段为 [`PLAN`]——逐字引用记忆
//! 文件《fuyu-rhapsody-android-port》的 SOP 措辞。
//!
//! # 计划步骤（引自记忆文件的「MV 移动 UA 引擎分支坑」与 Android 工程要点）
//!
//! 记忆实录的架构：WebView + 内嵌回环 AssetServer(47777) 直接流式读取 APK
//! 未压缩条目；存档走 localStorage（fuyu_preload.js 预载 19 个存档）。
//!
//! 1. 注入 `js/fuyu_fix.js` 修正 MV 移动 UA 引擎分支坑（用户反馈「卡加载/
//!    交互差/无声」后定位）：
//!    - `AudioManager.audioFileExt` 移动端返回 `.m4a` 而游戏只带 `.rpgmvo`
//!      → 全程无声；
//!    - `shouldUseCanvasRenderer` 移动端 true → Canvas2D 卡顿；
//!    - `_defaultStretchMode` 移动端 true → 画面拉伸变形；
//!    - 安卓 WebView 中被放弃的位图不触发 onload/onerror →
//!      `Graphics._loadingCount` 泄漏导致 "Now Loading" 永挂（看门狗清零
//!      计数器是安全的：场景就绪走 ImageManager 自己的状态）；
//!    - MV 原生触屏点角色只走过去不触发事件 → 包一层 triggerTouchAction，
//!      贴身面向并自动 start()。
//! 2. index.html 注入补丁 script 标签（教训：构建失败先查 index.html 里
//!    script 标签）。
//! 3. Android 工程要点：
//!    - 无 INTERNET 权限则 socket 创建即抛异常且静默 finish()，必须声明；
//!    - `getLoopbackAddress()` 返回 ::1 而 WebView 请求 127.0.0.1，必须
//!      显式绑 IPv4；
//!    - AssetManager 要求 assets/ 前缀；
//!    - MainActivity `setWebContentsDebuggingEnabled(true)`（记忆实录：
//!      MainActivity.java:24），DevTools 必可达。
//! 4. 存档链路：fuyu_preload.js 预载 localStorage 存档键。

use crate::stages::{PipelineContext, PipelineError, Result, Stage, StageReport};

/// adapt 阶段的计划步骤（引自记忆文件《fuyu-rhapsody-android-port》）。
pub const PLAN: &str = "\
1. 注入 js/fuyu_fix.js 修正 MV 移动 UA 分支：audioFileExt 移动端返回 .m4a 而游戏只带 .rpgmvo→全程无声；shouldUseCanvasRenderer 移动端 true→Canvas2D 卡顿；_defaultStretchMode 移动端 true→画面拉伸变形；被放弃的位图不触发 onload/onerror→Graphics._loadingCount 泄漏 \"Now Loading\" 永挂（看门狗清零计数器）；触屏点角色只走过去不触发事件→triggerTouchAction 包一层，贴身面向并自动 start()；
2. index.html 注入补丁 script 标签（构建失败先查 script 标签）；
3. Android 工程：声明 INTERNET 权限（无权限则 socket 创建即抛异常且静默 finish()）；回环 AssetServer 显式绑 IPv4（getLoopbackAddress() 返回 ::1 而 WebView 请求 127.0.0.1）；AssetManager 要求 assets/ 前缀；setWebContentsDebuggingEnabled(true) 保证 DevTools 可达；
4. 存档走 localStorage（fuyu_preload.js 预载存档）。";

/// adapt 阶段（stub）。
#[derive(Debug, Default, Clone, Copy)]
pub struct AdaptStage;

impl Stage for AdaptStage {
    fn name(&self) -> &str {
        "adapt"
    }

    fn run(&self, _ctx: &mut PipelineContext) -> Result<StageReport> {
        Err(PipelineError::NotImplemented {
            stage: "adapt",
            plan: PLAN,
        })
    }
}
