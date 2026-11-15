//! repack 阶段（stub）：把适配后的产物重封为 APK。
//!
//! **MVP 骨架**：不实现任何真实重封逻辑，[`RepackStage::run`] 返回
//! [`PipelineError::NotImplemented`]，`plan` 字段为 [`PLAN`]——逐字引用记忆
//! 文件《fuyu-rhapsody-android-port》的 SOP 措辞。
//!
//! # 计划步骤（引自记忆文件）
//!
//! 记忆实录：APK 产物 `FuyuRhapsody.apk` 1.7GB、包名 com.fuyu.rhapsody、
//! 重建后 38150 条目；构建入口 `project/build.sh`（assets 缓存
//! assets-cache.zip，游戏文件改动加 --fresh-assets）。
//!
//! 1. **zip64 红线**：zip 条目 >65535 时 bt34/35/36 的 zipalign+apksigner 全
//!    不支持 zip64 → 必须按内容哈希去重重复帧文件并用
//!    `fuyu_assets_map.tsv` 别名表压缩条目数（记忆实录：去重 31516 个重复
//!    帧，重建后 38150 条目）。
//! 2. 经 `build.sh` 构建（assets 缓存 assets-cache.zip，游戏文件改动加
//!    `--fresh-assets`）。
//! 3. 产物核对：APK 条目数、包名（如 com.fuyu.rhapsody 模式）、体积。

use crate::stages::{PipelineContext, PipelineError, Result, Stage, StageReport};

/// repack 阶段的计划步骤（引自记忆文件《fuyu-rhapsody-android-port》）。
pub const PLAN: &str = "\
1. zip 条目 >65535 时 bt34/35/36 的 zipalign+apksigner 全不支持 zip64→按内容哈希去重重复帧文件（记忆实录：31516 个）+ fuyu_assets_map.tsv 别名表压缩条目数（重建后 38150 条目）；
2. 经 build.sh 构建（assets 缓存 assets-cache.zip，游戏文件改动加 --fresh-assets）；
3. 产物核对：条目数、包名（com.fuyu.rhapsody 模式）、体积（记忆实录：FuyuRhapsody.apk 1.7GB）。";

/// repack 阶段（stub）。
#[derive(Debug, Default, Clone, Copy)]
pub struct RepackStage;

impl Stage for RepackStage {
    fn name(&self) -> &str {
        "repack"
    }

    fn run(&self, _ctx: &mut PipelineContext) -> Result<StageReport> {
        Err(PipelineError::NotImplemented {
            stage: "repack",
            plan: PLAN,
        })
    }
}
