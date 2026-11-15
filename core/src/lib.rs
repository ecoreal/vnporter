//! vnporter-core：PC 视觉小说 → 安卓 APK 移植流水线的纯 Rust 核心。
//!
//! 五阶段流水线（MVP：骨架 + 真实可跑的 detect 阶段）：
//! `detect → unpack → adapt → repack → sign`。
//!
//! 设计依据：项目记忆文件《fuyu-rhapsody-android-port》——《冬日狂想曲v1.02》
//! （RPG Maker MV 1.6.1）安卓移植的真实 SOP 与踩坑记录。其中的关键事实：
//!
//! - 引擎：RPG Maker MV 1.6.1；游戏数据在 `www/` 下组织（`index.html`、`js/`、
//!   `js/plugins/`、音频 `.rpgmvo`、图像 `.rpgmvp`）
//! - 目标架构：WebView + 内嵌回环 AssetServer(47777) 直接流式读取 APK 未压缩
//!   条目；存档走 localStorage
//! - 关键坑：zip 条目 >65535 时 bt34/35/36 的 zipalign+apksigner 全不支持
//!   zip64 → 按内容哈希去重 + 别名表压缩条目数；无 INTERNET 权限则 socket
//!   创建即抛异常且静默 finish()；`getLoopbackAddress()` 返回 ::1 而 WebView
//!   请求 127.0.0.1，必须显式绑 IPv4；AssetManager 要求 assets/ 前缀
//! - 签名：keystore.jks（storepass android）+ apksigner 验签
//!
//! 本 crate 不依赖 Tauri，可独立测试。MVP 中 unpack/adapt/repack/sign 仅为
//! stub：返回 [`PipelineError::NotImplemented`]（携带引自记忆文件 SOP 的计划
//! 步骤），不实现任何解包/重封的真实逻辑。

pub mod adapt;
pub mod detect;
pub mod pipeline;
pub mod repack;
pub mod sign;
pub mod stages;
pub mod unpack;

pub use adapt::AdaptStage;
pub use detect::{Confidence, DetectStage, Detection, Engine, Feature};
pub use pipeline::{Pipeline, PipelineReport};
pub use repack::RepackStage;
pub use sign::SignStage;
pub use stages::{PipelineContext, PipelineError, Stage, StageReport, StageStatus};
pub use unpack::UnpackStage;
