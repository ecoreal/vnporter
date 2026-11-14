# vnporter

Tauri 2 桌面应用模板：React 19 + TypeScript 6 + Tailwind v4 + shadcn/ui + vitest，内置 CI（双平台测试）与三平台发布流水线。

## 技术栈

- **桌面**：Tauri 2（Rust 1.100+）
- **前端**：React 19、Vite 8、TypeScript 6（strict）、Tailwind CSS v4、shadcn/ui（radix-nova，neutral）
- **测试**：vitest（jsdom）+ cargo test
- **CI**：GitHub Actions — frontend 矩阵（ubuntu/windows）跑类型检查+测试+构建；rust job 跑 cargo test
- **发布**：`v*` tag 触发，校验 tag 版本 == `src-tauri/tauri.conf.json` version，tauri-action 三平台出包（deb/AppImage/rpm、NSIS/msi、dmg）

## 用这个模板新建项目

1. `gh repo create <name> --template` 或直接复制本目录，`git init` 后改名
2. 需要改的位（模板内已用占位值）：
   - `package.json` → `name`
   - `src-tauri/Cargo.toml` → `name` 与 `lib.name`
   - `src-tauri/tauri.conf.json` → `productName` / `identifier` / 窗口 `title`
   - 两个 workflow 无需改动（无仓库地址引用）
3. 按需加 Tauri 插件（托盘/通知/全局热键/store 等）

## 命令

| 命令 | 说明 |
|---|---|
| `npm run tauri dev` | 开发窗口 |
| `npm run tauri build` | 打包 |
| `npm test` / `npm run test:watch` | vitest |
| `npx tsc -b` | 类型检查 |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Rust 测试 |

## Linux 构建依赖（Debian/Ubuntu）

```bash
sudo apt-get install -y build-essential pkg-config libx11-dev libasound2-dev libudev-dev \
  libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf
```

## License

MIT
