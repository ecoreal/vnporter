//! 测试共用 fixture：全部为合成文件（空文件或一行合成文本），
//! 绝不触碰任何真实游戏目录（红线）。
//!
//! 目录布局依据记忆文件《fuyu-rhapsody-android-port》：《冬日狂想曲v1.02》
//! 为 RPG Maker MV 1.6.1，游戏数据在 `www/` 下组织。

// 共享 helper 会被部分测试文件使用，其余文件引入时按 dead_code 报警。
#![allow(dead_code)]

use std::fs;
use std::path::Path;

/// 合成 RPG Maker MV fixture：完整命中全部 5 个特征。
///
/// 记忆实录的元素：入口 index.html、核心脚本 rpg_core.js（含
/// RPGMAKER_VERSION = 1.6.1 标记）、插件 plugins/FuyuUnlock.js、
/// 音频 .rpgmvo、图像 .rpgmvp（文件名用记忆里的大小写别名示例 volcano）。
pub fn mv_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("www/js/plugins")).unwrap();
    fs::create_dir_all(dir.join("www/audio")).unwrap();
    fs::create_dir_all(dir.join("www/img")).unwrap();
    fs::write(dir.join("www/index.html"), "").unwrap();
    fs::write(
        dir.join("www/js/rpg_core.js"),
        "Utils.RPGMAKER_VERSION = '1.6.1';\n",
    )
    .unwrap();
    fs::write(dir.join("www/js/plugins/FuyuUnlock.js"), "").unwrap();
    fs::write(dir.join("www/audio/bgm1.rpgmvo"), "").unwrap();
    fs::write(dir.join("www/img/volcano.rpgmvp"), "").unwrap();
}

/// 部分命中：只留弱特征（index.html + plugins 目录），应为 Medium 置信度。
pub fn mv_partial_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("www/js/plugins")).unwrap();
    fs::write(dir.join("www/index.html"), "").unwrap();
}

/// 单一弱特征：只有 index.html，应为 Low 置信度（疑似）。
pub fn mv_weak_fixture(dir: &Path) {
    fs::create_dir_all(dir.join("www")).unwrap();
    fs::write(dir.join("www/index.html"), "").unwrap();
}

/// 合成 Kirikiri fixture：只有 krkr 系标志文件 data.xp3（任务规格示例特征）。
pub fn kirikiri_fixture(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("data.xp3"), "").unwrap();
}

/// 在 fixture 目录下创建空文件（自动建父目录）。
pub fn touch(dir: &Path, rel: &str) {
    let path = dir.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, "").unwrap();
}
