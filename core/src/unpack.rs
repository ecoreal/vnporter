//! unpack 阶段（stub）：把 PC 版游戏数据解包到中间产物目录。
//!
//! **MVP 骨架**：不实现任何真实解包逻辑，[`UnpackStage::run`] 返回
//! [`PipelineError::NotImplemented`]，`plan` 字段为 [`PLAN`]——逐字引用记忆
//! 文件《fuyu-rhapsody-android-port》的 SOP 措辞。
//!
//! # 计划步骤（引自记忆文件）
//!
//! 记忆实录的游戏组织：《冬日狂想曲v1.02》为 RPG Maker MV 1.6.1，游戏数据
//! 在 `www/` 下（index.html、js/、js/plugins/、音频 .rpgmvo、图像 .rpgmvp），
//! 解包产物曾落在 `game-src/extracted/5.冬日狂想曲v1.02/www`。
//!
//! 1. 校验 detect 结果：引擎为 [`Engine::RpgMakerMv`](crate::detect::Engine)
//!    时确认 `www/` 结构完整（index.html、js/rpg_core.js）。
//! 2. 把游戏文件拷入 `ctx.work_dir`，并建立小写→实际路径索引
//!    `fuyu_case_index.tsv`（记忆实录：37672 条非恒等映射，如
//!    `volcano.rpgmvp→Volcano.rpgmvp`，供 AssetServer 大小写解析）。
//! 3. 按内容哈希去重重复帧文件（记忆实录：31516 个重复帧），生成
//!    `fuyu_assets_map.tsv` 别名表（AssetServer 加载）。
//! 4. 生成 assets 缓存 `assets-cache.zip`；游戏文件改动后需加
//!    `--fresh-assets` 重建（记忆实录：构建失败发生在 javac 时 assets 缓存
//!    可能没重建，先查 index.html 里 script 标签验证改动是否生效）。

use crate::stages::{PipelineContext, PipelineError, Result, Stage, StageReport};

/// unpack 阶段的计划步骤（引自记忆文件《fuyu-rhapsody-android-port》）。
pub const PLAN: &str = "\
1. 校验 www/ 结构（index.html、js/rpg_core.js，RPG Maker MV 1.6.1）；
2. 游戏文件拷入中间产物目录，建立小写→实际路径索引 fuyu_case_index.tsv（记忆实录：37672 条非恒等映射，如 volcano.rpgmvp→Volcano.rpgmvp，供 AssetServer 大小写解析）；
3. 按内容哈希去重重复帧文件（记忆实录：31516 个重复帧），生成 fuyu_assets_map.tsv 别名表（AssetServer 加载）；
4. 生成 assets 缓存 assets-cache.zip（游戏文件改动加 --fresh-assets 重建）。";

/// unpack 阶段（stub）。
#[derive(Debug, Default, Clone, Copy)]
pub struct UnpackStage;

impl Stage for UnpackStage {
    fn name(&self) -> &str {
        "unpack"
    }

    fn run(&self, _ctx: &mut PipelineContext) -> Result<StageReport> {
        Err(PipelineError::NotImplemented {
            stage: "unpack",
            plan: PLAN,
        })
    }
}
