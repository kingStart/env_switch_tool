# GitHub Actions 自动化工作流

本项目配置了一套完整的 GitHub 自动化体系，涵盖 CI、PR 审查、安全审计、依赖管理和发布全流程。

---

## 工作流总览

```
代码推送 / PR
  ├── CI          → 格式检查 + Lint + 三平台测试
  ├── PR Review   → Clippy 行内审查 + 格式建议 + PR 体量标签
  └── Security    → cargo-audit 漏洞扫描 (依赖变更时)

定时任务
  ├── Security    → 每周一扫描依赖漏洞
  ├── Dependabot  → 每周一检测 Cargo/Actions 依赖更新 → 自动发起 PR
  └── Stale       → 每周日清理过期 Issue/PR

Tag 推送 (v*)
  └── Release     → 6 架构并行构建 → 打包 → 创建 GitHub Release
```

---

## 1. CI 工作流

**文件**: `.github/workflows/ci.yml`  
**触发**: 向 `main` 分支推送代码或发起 PR

| Job | 运行环境 | 执行内容 |
|-----|---------|---------|
| **check** | ubuntu-latest | `cargo fmt --check` 格式检查 + `cargo clippy -D warnings` 静态分析 |
| **test** | ubuntu / macOS / Windows | `cargo test --workspace` 全工作空间测试 |

`test` 依赖 `check` 通过后才会执行，三个平台并行运行（`fail-fast: false`）。

---

## 2. PR 自动审查

**文件**: `.github/workflows/pr-review.yml`  
**触发**: PR 被创建、同步或重新打开时

| Job | 功能 |
|-----|------|
| **clippy-review** | 使用 [clippy-action](https://github.com/giraffate/clippy-action) 将 Clippy 警告作为 PR Review 评论直接标注在代码行上 |
| **fmt-review** | 使用 [reviewdog](https://github.com/reviewdog/action-suggester) 将格式化修正作为 PR 建议提交 |
| **size-label** | 根据 PR 改动行数自动打标签（XS/S/M/L/XL），帮助审查者快速评估工作量 |

**效果**: 每个 PR 会自动收到 Clippy 和格式化的行内审查评论，无需人工触发。

---

## 3. 安全审计

**文件**: `.github/workflows/security-audit.yml`  
**触发**:
- `Cargo.toml` 或 `Cargo.lock` 变更时（push 到 main）
- 每周一 08:00 UTC 定时扫描

使用 [rustsec/audit-check](https://github.com/rustsec/audit-check) 检测依赖中的已知漏洞，发现问题会自动创建 Issue。

---

## 4. Dependabot 依赖管理

**文件**: `.github/dependabot.yml`

| 生态系统 | 频率 | 策略 |
|---------|------|------|
| **Cargo** (Rust 依赖) | 每周一 | minor/patch 合并为一个 PR，最多 5 个开放 PR |
| **GitHub Actions** | 每周一 | 独立 PR，最多 3 个开放 PR |

Dependabot 会自动检测依赖更新并发起 PR，PR 标签为 `dependencies` + `automated`（Cargo）或 `ci` + `automated`（Actions），**等待你审核后合入**。

---

## 5. Stale 管理

**文件**: `.github/workflows/stale.yml`  
**触发**: 每周日定时运行

| 类型 | 无活动标记时间 | 自动关闭时间 | 豁免标签 |
|------|-------------|------------|---------|
| Issue | 60 天 | +14 天 | `pinned`, `in-progress` |
| PR | 30 天 | +14 天 | `pinned`, `in-progress` |

被标记 `stale` 的 Issue/PR 会收到提醒消息，如有回复或移除标签则重置计时。

---

## 6. Release 多架构构建

**文件**: `.github/workflows/release.yml`  
**触发**: 推送 `v*` 格式的 Git 标签

### 构建矩阵

| 目标 (target) | 操作系统 | 架构 | 打包格式 | 构建方式 |
|--------------|---------|------|---------|---------|
| `x86_64-unknown-linux-gnu` | Linux | x64 | `.tar.gz` | cargo |
| `aarch64-unknown-linux-gnu` | Linux | ARM64 | `.tar.gz` | cross |
| `x86_64-apple-darwin` | macOS | x64 (Intel) | `.tar.gz` | cargo |
| `aarch64-apple-darwin` | macOS | ARM64 (Apple Silicon) | `.tar.gz` | cargo |
| `x86_64-pc-windows-msvc` | Windows | x64 | `.zip` | cargo |
| `aarch64-pc-windows-msvc` | Windows | ARM64 | `.zip` | cargo |

### 产出物

```
envtools-x86_64-unknown-linux-gnu.tar.gz
envtools-aarch64-unknown-linux-gnu.tar.gz
envtools-x86_64-apple-darwin.tar.gz
envtools-aarch64-apple-darwin.tar.gz
envtools-x86_64-pc-windows-msvc.zip
envtools-aarch64-pc-windows-msvc.zip
sha256sums.txt
```

---

## Issue / PR 模板

| 模板 | 文件 | 用途 |
|------|------|------|
| Bug Report | `.github/ISSUE_TEMPLATE/bug_report.yml` | 结构化 Bug 报告（含系统、版本、复现步骤） |
| Feature Request | `.github/ISSUE_TEMPLATE/feature_request.yml` | 功能建议（含动机、方案、备选） |
| PR Template | `.github/pull_request_template.md` | PR 提交规范（变更类型、关联 Issue、自查清单） |

---

## 使用方式

### 日常开发

```bash
git push origin main          # 触发 CI
```

### 发版

```bash
git tag v0.1.0
git push origin v0.1.0        # 触发 6 架构构建 + GitHub Release
```

### 用户安装

**Linux / macOS:**

```bash
tar xzf envtools-<target>.tar.gz
sudo mv envtools /usr/local/bin/
```

**Windows:** 解压 `.zip` 后将 `envtools.exe` 所在目录添加到系统 `PATH`。

---

## 依赖的 GitHub Actions

| Action | 用途 |
|--------|-----|
| [actions/checkout@v4](https://github.com/actions/checkout) | 拉取代码 |
| [dtolnay/rust-toolchain@stable](https://github.com/dtolnay/rust-toolchain) | 安装 Rust 工具链 |
| [Swatinem/rust-cache@v2](https://github.com/Swatinem/rust-cache) | 缓存编译产物 |
| [giraffate/clippy-action@v1](https://github.com/giraffate/clippy-action) | Clippy PR 行内审查 |
| [reviewdog/action-suggester@v1](https://github.com/reviewdog/action-suggester) | 格式化修正建议 |
| [CodelyTV/pr-size-labeler@v1](https://github.com/CodelyTV/pr-size-labeler) | PR 体量标签 |
| [rustsec/audit-check@v2](https://github.com/rustsec/audit-check) | 安全漏洞审计 |
| [actions/stale@v9](https://github.com/actions/stale) | 过期 Issue/PR 管理 |
| [softprops/action-gh-release@v2](https://github.com/softprops/action-gh-release) | 创建 GitHub Release |
| [actions/upload-artifact@v4](https://github.com/actions/upload-artifact) | 上传构建产物 |
| [actions/download-artifact@v4](https://github.com/actions/download-artifact) | 下载构建产物 |
