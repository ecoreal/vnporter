//! detect 阶段：根据记忆文件《fuyu-rhapsody-android-port》中的引擎特征做
//! 真实的文件系统检测，返回 [`Engine`] 枚举 + 置信度说明。
//!
//! 特征来源（每个特征一个独立函数）：
//!
//! - RPG Maker MV（记忆实录：《冬日狂想曲v1.02》= RPG Maker MV 1.6.1，
//!   游戏数据在 `www/` 下组织）：
//!   - [`feature_index_html`]：`www/index.html`（记忆实录：入口 HTML，
//!     「index.html 不含 fuyu_probe.js」即指此文件）
//!   - [`feature_rpg_core_js`]：`www/js/rpg_core.js`（记忆实录：
//!     `Graphics._loadingCount`、`ImageManager`、`AudioManager` 等 MV 核心
//!     管理器所在脚本系）
//!   - [`feature_plugins_dir`]：`www/js/plugins/`（记忆实录：`plugins/FuyuUnlock.js`）
//!   - [`feature_rpgmvo_audio`]：`.rpgmvo` 音频（记忆实录：「游戏只带
//!     .rpgmvo」——即 audioFileExt 移动端误报 .m4a 导致全程无声的对照）
//!   - [`feature_rpgmvp_image`]：`.rpgmvp` 图像（记忆实录：小写→实际路径表
//!     中的 `volcano.rpgmvp→Volcano.rpgmvp`）
//!   - [`detect_mv_version`]：从 rpg_core.js 解析 `RPGMAKER_VERSION`（记忆
//!     实录：MV 1.6.1），仅作置信度佐证，不参与判定
//! - Kirikiri（krkr 系）：[`feature_kirikiri_data_xp3`]：标志文件
//!   `data.xp3`。注意：该特征来自任务规格中的示例（data.xp3→krkr 系），
//!   记忆文件本身未涉及 Kirikiri 游戏。
//!
//! 红线：本模块只在测试中以合成 fixture（空标志文件）运行；绝不指向任何
//! 真实游戏目录。

use std::fmt;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::stages::{PipelineContext, Result, Stage, StageReport};

/// 递归统计文件数时的上限：真实 MV 游戏有数万条目（记忆实录：重建后 APK
/// 达 38150 条目），到上限即提前停止并注明「≥N」。
const WALK_COUNT_CAP: usize = 1024;

/// 支持的引擎。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Engine {
    /// RPG Maker MV（记忆实录：《冬日狂想曲v1.02》= RPG Maker MV 1.6.1）。
    RpgMakerMv,
    /// Kirikiri（krkr 系；标志文件 data.xp3，来自任务规格示例）。
    Kirikiri,
    /// 未识别。
    Unknown,
}

impl Engine {
    /// 引擎的稳定标识（与 serde snake_case 序列化一致）。
    pub fn as_str(&self) -> &'static str {
        match self {
            Engine::RpgMakerMv => "rpg_maker_mv",
            Engine::Kirikiri => "kirikiri",
            Engine::Unknown => "unknown",
        }
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 检测置信度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// 全部特征命中（MV 5/5）或命中 krkr 标志文件。
    High,
    /// 命中 2–3 个 MV 特征。
    Medium,
    /// 仅命中 1 个弱特征，或完全未识别。
    Low,
}

/// 单个引擎特征的检测结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Feature {
    /// 短标识，直接出现在置信度说明中（如 `"www/index.html"`）。
    pub name: String,
    /// 是否命中。
    pub hit: bool,
    /// 命中明细（如 "1 个 *.rpgmvo 音频"）。
    pub detail: String,
}

impl Feature {
    fn new(name: &str, hit: bool, detail: String) -> Self {
        Self {
            name: name.to_owned(),
            hit,
            detail,
        }
    }
}

/// detect 阶段的最终结论：引擎 + 置信度 + 人读的置信度说明。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Detection {
    /// 判定出的引擎（未知时为 [`Engine::Unknown`]）。
    pub engine: Engine,
    /// 置信度。
    pub confidence: Confidence,
    /// 置信度说明（列出命中/未命中的特征与版本佐证）。
    pub explanation: String,
}

// ---------------------------------------------------------------------------
// 特征函数（每个特征一个独立函数）
// ---------------------------------------------------------------------------

/// 特征：RPG Maker MV 的入口 HTML `www/index.html`。
pub fn feature_index_html(game_dir: &Path) -> Feature {
    let path = game_dir.join("www").join("index.html");
    let hit = path.is_file();
    Feature::new(
        "www/index.html",
        hit,
        if hit {
            "入口 HTML 存在".to_owned()
        } else {
            "缺少 www/index.html".to_owned()
        },
    )
}

/// 特征：RPG Maker MV 核心脚本 `www/js/rpg_core.js`。
pub fn feature_rpg_core_js(game_dir: &Path) -> Feature {
    let path = game_dir.join("www").join("js").join("rpg_core.js");
    let hit = path.is_file();
    Feature::new(
        "www/js/rpg_core.js",
        hit,
        if hit {
            "MV 核心脚本存在".to_owned()
        } else {
            "缺少 www/js/rpg_core.js".to_owned()
        },
    )
}

/// 特征：RPG Maker MV 插件目录 `www/js/plugins/`（记忆实录：plugins/FuyuUnlock.js）。
pub fn feature_plugins_dir(game_dir: &Path) -> Feature {
    let path = game_dir.join("www").join("js").join("plugins");
    let hit = path.is_dir();
    Feature::new(
        "www/js/plugins/",
        hit,
        if hit {
            "插件目录存在".to_owned()
        } else {
            "缺少 www/js/plugins/ 目录".to_owned()
        },
    )
}

/// 特征：`.rpgmvo` 音频文件（记忆实录：「游戏只带 .rpgmvo」）。
///
/// 递归统计全目录（大小写不敏感），到 [`WALK_COUNT_CAP`] 提前停止。
pub fn feature_rpgmvo_audio(game_dir: &Path) -> Feature {
    count_feature(game_dir, "*.rpgmvo", "rpgmvo", ".rpgmvo 音频")
}

/// 特征：`.rpgmvp` 图像文件（记忆实录：volcano.rpgmvp→Volcano.rpgmvp 大小写别名）。
///
/// 递归统计全目录（大小写不敏感），到 [`WALK_COUNT_CAP`] 提前停止。
pub fn feature_rpgmvp_image(game_dir: &Path) -> Feature {
    count_feature(game_dir, "*.rpgmvp", "rpgmvp", ".rpgmvp 图像")
}

/// 特征：Kirikiri 标志文件 `data.xp3`（krkr 系；任务规格示例特征）。
pub fn feature_kirikiri_data_xp3(game_dir: &Path) -> Feature {
    let path = game_dir.join("data.xp3");
    let hit = path.is_file();
    Feature::new(
        "data.xp3",
        hit,
        if hit {
            "krkr 系标志包存在".to_owned()
        } else {
            "缺少 data.xp3".to_owned()
        },
    )
}

/// 佐证：从 `www/js/rpg_core.js` 解析 `RPGMAKER_VERSION = "x.y.z"`。
///
/// 记忆实录：《冬日狂想曲》为 MV 1.6.1。解析失败（缺文件/无标记/格式异常）
/// 返回 `None`，不影响引擎判定，只作为置信度说明的一部分。
pub fn detect_mv_version(game_dir: &Path) -> Option<String> {
    let js = fs::read_to_string(game_dir.join("www").join("js").join("rpg_core.js")).ok()?;
    let idx = js.find("RPGMAKER_VERSION")?;
    let rest = &js[idx..];
    let quote_pos = rest.find(['"', '\''])?;
    let quote = rest.as_bytes()[quote_pos] as char;
    let after = &rest[quote_pos + 1..];
    let end = after.find(quote)?;
    let version = &after[..end];
    if version.is_empty() || !version.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    Some(version.to_owned())
}

// ---------------------------------------------------------------------------
// 判定
// ---------------------------------------------------------------------------

/// RPG Maker MV 特征全集的检测入口（供 detect 与测试复用）。
pub fn mv_features(game_dir: &Path) -> Vec<Feature> {
    vec![
        feature_index_html(game_dir),
        feature_rpg_core_js(game_dir),
        feature_plugins_dir(game_dir),
        feature_rpgmvo_audio(game_dir),
        feature_rpgmvp_image(game_dir),
    ]
}

/// 执行引擎检测。
///
/// 判定规则：
/// 1. 命中任意 MV 特征 → [`Engine::RpgMakerMv`]；置信度按命中数：
///    4–5 → High，2–3 → Medium，1 → Low。
/// 2. 否则命中 `data.xp3` → [`Engine::Kirikiri`]（High）。
/// 3. 否则 → [`Engine::Unknown`]（Low）。
///
/// MV 优先于 Kirikiri：MV 特征（www/ 结构 + .rpgmvo/.rpgmvp）比单一的
/// data.xp3 更具体。
pub fn detect(game_dir: &Path) -> Detection {
    let features = mv_features(game_dir);
    let hits: Vec<&Feature> = features.iter().filter(|f| f.hit).collect();
    let version = detect_mv_version(game_dir);
    let kirikiri = feature_kirikiri_data_xp3(game_dir);

    if !hits.is_empty() {
        let score = hits.len();
        let confidence = match score {
            4..=5 => Confidence::High,
            2..=3 => Confidence::Medium,
            _ => Confidence::Low,
        };
        let hit_names: Vec<String> = hits
            .iter()
            .map(|f| format!("{}({})", f.name, f.detail))
            .collect();
        let miss_names: Vec<String> = features
            .iter()
            .filter(|f| !f.hit)
            .map(|f| f.name.clone())
            .collect();
        let mut explanation = format!(
            "RPG Maker MV：命中 {score}/5 特征 [{}]，未命中 [{}]",
            hit_names.join(", "),
            miss_names.join(", "),
        );
        match &version {
            Some(v) => explanation.push_str(&format!(
                "；rpg_core.js 标记 RPGMAKER_VERSION = \"{v}\"（记忆实录：MV 1.6.1）"
            )),
            None => explanation.push_str("；未解析到 RPGMAKER_VERSION 标记"),
        }
        return Detection {
            engine: Engine::RpgMakerMv,
            confidence,
            explanation,
        };
    }

    if kirikiri.hit {
        return Detection {
            engine: Engine::Kirikiri,
            confidence: Confidence::High,
            explanation: format!(
                "Kirikiri：命中 krkr 系标志文件 {}({})（特征来自任务规格示例；记忆文件未涉及 krkr）",
                kirikiri.name, kirikiri.detail
            ),
        };
    }

    Detection {
        engine: Engine::Unknown,
        confidence: Confidence::Low,
        explanation: format!(
            "未识别引擎：未命中任何已知特征（RPG Maker MV 0/5、Kirikiri data.xp3 缺失）"
        ),
    }
}

// ---------------------------------------------------------------------------
// detect 阶段实现
// ---------------------------------------------------------------------------

/// detect 阶段：真实执行引擎检测，把结果写入 `ctx.engine` 并产出报告。
#[derive(Debug, Default, Clone, Copy)]
pub struct DetectStage;

impl Stage for DetectStage {
    fn name(&self) -> &str {
        "detect"
    }

    fn run(&self, ctx: &mut PipelineContext) -> Result<StageReport> {
        ctx.ensure_game_dir()?;
        let detection = detect(&ctx.game_dir);
        ctx.engine = Some(detection.engine);
        Ok(StageReport::succeeded(self.name(), detection.explanation))
    }
}

// ---------------------------------------------------------------------------
// 内部工具
// ---------------------------------------------------------------------------

fn count_feature(game_dir: &Path, glob_name: &str, ext: &str, label: &str) -> Feature {
    let count = count_files_with_ext(game_dir, ext);
    let detail = if count == 0 {
        format!("未发现 {glob_name} {label}")
    } else if count >= WALK_COUNT_CAP {
        format!("≥{WALK_COUNT_CAP} 个 {glob_name} {label}")
    } else {
        format!("{count} 个 {glob_name} {label}")
    };
    Feature::new(glob_name, count > 0, detail)
}

/// 递归统计扩展名为 `ext`（大小写不敏感）的文件数，达到 [`WALK_COUNT_CAP`]
/// 提前返回。读目录失败一律静默跳过（检测不应因权限问题崩溃）。
fn count_files_with_ext(dir: &Path, ext: &str) -> usize {
    let mut count = 0usize;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case(ext))
            {
                count += 1;
                if count >= WALK_COUNT_CAP {
                    return count;
                }
            }
        }
    }
    count
}
