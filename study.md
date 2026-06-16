# GitHub Actions CI/CD 学习笔记

本文档基于本项目 `.github/` 目录下的实际配置，系统梳理 GitHub Actions 流水线相关知识。

---

## 目录结构

```
.github/
├── workflows/
│   └── ci.yml                    # 主流水线：测试、lint、构建、发布
├── dependabot.yml                # 自动依赖更新机器人配置
└── pull_request_template.md      # PR 模板
```

---

## 一、ci.yml — 主流水线详解

### 1.1 触发条件 (`on`)

```yaml
on:
  push:
    branches: [main]     # main 分支 push 时触发
    tags: ["v*"]         # 以 v 开头的 tag（如 v1.0.0）触发 release
  pull_request:
    branches: [main]     # 向 main 分支发起 PR 时触发
```

**要点**：
- `push` + `pull_request` 是最常见组合
- `tags: ["v*"]` 用于自动发布（只有 tag push 才触发 release job）
- 也可用 `workflow_dispatch` 实现手动触发

### 1.2 并发控制 (`concurrency`)

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

**作用**：同一工作流 + 同一分支/PR，新 push 会自动取消正在运行的旧任务。

**场景**：频繁推送修复时，避免排队等待旧构建完成浪费资源。

### 1.3 全局环境变量 (`env`)

```yaml
env:
  CARGO_TERM_COLOR: always    # cargo 输出带颜色（CI 日志可读性）
  RUST_BACKTRACE: 1           # 出错时打印 backtrace
```

定义在顶层的 `env` 对所有 job 生效。

### 1.4 Job 依赖关系

```
test ──┐
       ├──▶ build ──▶ release（仅 tag push）
lint ──┘
```

- `test` 和 `lint` 并行执行
- `build` 依赖两者都通过 (`needs: [test, lint]`)
- `release` 依赖 `build` 且仅在 tag push 时运行 (`if: startsWith(github.ref, 'refs/tags/v')`)

---

### 1.5 Test Job — 多平台测试

```yaml
jobs:
  test:
    strategy:
      fail-fast: false          # 某平台失败不影响其他平台继续跑
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}   # 每个 OS 启动一个独立 runner
```

**关键概念**：

| 概念 | 说明 |
|------|------|
| `matrix` | 生成多个并行 job 实例，每个实例使用不同的矩阵值 |
| `fail-fast: false` | 默认一个失败全部取消，设为 false 可看到所有平台的结果 |
| `runs-on` | 指定运行环境（GitHub 提供免费的 Linux/macOS/Windows runner） |

**缓存策略**：

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry     # crate 源码缓存
      ~/.cargo/git          # git 依赖缓存
      target                # 编译产物缓存
    key: ${{ runner.os }}-cargo-test-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: ${{ runner.os }}-cargo-test-
```

- `key`：精确匹配（Cargo.lock 变化则失效）
- `restore-keys`：前缀匹配回退（至少能用上旧缓存）

### 1.6 Lint Job — 代码质量检查

```yaml
lint:
  runs-on: ubuntu-latest        # 只需一个平台检查即可
  steps:
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy   # 安装额外组件
    - run: cargo fmt --all -- --check
    - run: cargo clippy --workspace -- -D warnings
```

- `cargo fmt -- --check`：只检查格式，不修改文件
- `-D warnings`：把 warning 提升为 error，强制零 warning

### 1.7 Build Job — 多平台多架构构建

**矩阵策略（include 写法）**：

```yaml
matrix:
  include:
    - platform: linux
      arch: x86_64
      runner: ubuntu-latest
      rust_target: x86_64-unknown-linux-gnu
      cli_artifact: envtools
    - platform: linux
      arch: aarch64
      runner: ubuntu-latest          # 仍用 x86 runner
      rust_target: aarch64-unknown-linux-gnu
      cross: true                    # 标记为交叉编译
    # ... macOS, Windows 类似
```

`include` 写法比 `matrix.os + matrix.arch` 笛卡尔积更灵活，可以为每个组合精确指定参数。

**条件执行 (`if`)**：

```yaml
- name: Install cross-compilation tools
  if: matrix.cross == true              # 只有交叉编译时执行

- name: Build Tauri GUI
  if: matrix.cross != true              # 交叉编译时跳过 GUI
```

**交叉编译要点**：

```yaml
- run: |
    sudo apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
    echo "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc" >> $GITHUB_ENV
```

- 安装目标架构的编译器
- 通过 `$GITHUB_ENV` 设置后续 step 可见的环境变量
- Cargo 通过 `CARGO_TARGET_<triple>_LINKER` 环境变量识别链接器

**构建产物上传**：

```yaml
- uses: actions/upload-artifact@v4
  with:
    name: envtools-${{ matrix.platform }}-${{ matrix.arch }}
    path: dist/envtools-${{ matrix.platform }}-${{ matrix.arch }}/
    retention-days: 30
```

Artifact 是 job 间传递文件的机制，保留 30 天后自动删除。

### 1.8 Release Job — 自动发布

```yaml
release:
  needs: [build]
  if: startsWith(github.ref, 'refs/tags/v')   # 只在 tag push 时执行
  permissions:
    contents: write                             # 需要写权限创建 Release
```

**流程**：
1. `actions/download-artifact@v4` 下载所有 build artifact
2. 压缩打包（Linux/macOS 用 tar.gz，Windows 用 zip）
3. `softprops/action-gh-release@v2` 创建 GitHub Release 并上传文件

**触发方式**：

```bash
git tag v1.0.0
git push origin v1.0.0
```

---

## 二、dependabot.yml — 自动依赖更新

```yaml
version: 2
updates:
  - package-ecosystem: "cargo"       # Rust 依赖
    directory: "/"
    schedule:
      interval: "weekly"
      day: "monday"
    open-pull-requests-limit: 5      # 最多同时 5 个 PR
    versioning-strategy: "increase-if-necessary"
    groups:
      minor-and-patch:               # 小版本合并到一个 PR
        update-types: ["minor", "patch"]

  - package-ecosystem: "npm"         # Node.js 依赖
    directory: "/"
    schedule:
      interval: "weekly"
      day: "monday"
    open-pull-requests-limit: 3
```

**核心配置**：

| 配置 | 作用 |
|------|------|
| `interval` | 检查频率：daily / weekly / monthly |
| `open-pull-requests-limit` | 控制同时打开的 PR 数量，避免被 PR 淹没 |
| `versioning-strategy` | `increase-if-necessary` 只在必要时提升版本 |
| `groups` | 将多个小更新合并为一个 PR，减少噪音 |

**Dependabot 工作流程**：
1. 按计划检查依赖新版本
2. 自动创建 PR（包含版本升级）
3. PR 会触发 CI，确认升级不会破坏构建
4. 人工 review 后合并

---

## 三、pull_request_template.md — PR 模板

当创建 PR 时，GitHub 自动填充此模板内容：

```markdown
## 变更说明
## 变更类型（勾选）
## 关联 Issue
## 自查清单
```

**好处**：
- 统一 PR 格式，方便 review
- 自查清单提醒提交前检查质量
- 关联 Issue 便于追踪

---

## 四、关键概念速查

### 4.1 GitHub Actions 核心模型

```
Workflow (.yml 文件)
  └── Job (独立运行的任务单元，各有自己的 runner)
       └── Step (按顺序执行的步骤)
            ├── uses: xxx   (使用官方/社区 Action)
            └── run: xxx    (执行 shell 命令)
```

### 4.2 常用内置变量

| 变量 | 含义 |
|------|------|
| `github.ref` | 触发的 ref（如 `refs/heads/main`、`refs/tags/v1.0`） |
| `github.sha` | 触发的 commit SHA |
| `github.workflow` | 工作流名称 |
| `runner.os` | 运行器 OS（Linux/macOS/Windows） |
| `matrix.*` | 当前矩阵实例的值 |

### 4.3 常用 Actions

| Action | 用途 |
|--------|------|
| `actions/checkout@v4` | 拉取仓库代码 |
| `actions/cache@v4` | 缓存依赖（加速后续构建） |
| `actions/upload-artifact@v4` | 上传构建产物 |
| `actions/download-artifact@v4` | 下载构建产物 |
| `actions/setup-node@v4` | 安装 Node.js |
| `dtolnay/rust-toolchain@stable` | 安装 Rust 工具链 |
| `softprops/action-gh-release@v2` | 创建 GitHub Release |

### 4.4 费用

| 类型 | 免费额度（公开仓库） | 免费额度（私有仓库） |
|------|------|------|
| Linux runner | 无限 | 2000 分钟/月 |
| macOS runner | 无限 | 200 分钟/月（10x 计费系数） |
| Windows runner | 无限 | 2000 分钟/月（2x 计费系数） |
| Storage | 无限 | 500 MB |

> **公开仓库完全免费，私有仓库有限额。**

### 4.5 实用技巧

1. **调试失败的 CI**：点击失败 job → 展开 step → 查看日志
2. **本地模拟 CI**：使用 [act](https://github.com/nektos/act) 工具本地运行 workflow
3. **加速构建**：善用 `actions/cache`，key 设计要合理（hash lock 文件）
4. **矩阵裁剪**：用 `include`/`exclude` 精确控制，避免不必要的组合
5. **安全发布**：tag 触发 + `permissions: contents: write` 最小权限原则

---

## 五、本项目流水线流程图

```
┌─────── push to main / PR ───────┐
│                                   │
│  ┌──────────┐    ┌──────────┐    │
│  │   Test   │    │   Lint   │    │
│  │ (3 OS)   │    │ (Ubuntu) │    │
│  └────┬─────┘    └────┬─────┘    │
│       │                │          │
│       └───────┬────────┘          │
│               ▼                   │
│  ┌──────────────────────────┐    │
│  │         Build            │    │
│  │  (6 platform x arch)    │    │
│  │  CLI + GUI artifacts     │    │
│  └─────────────┬────────────┘    │
│                │                  │
└────────────────┼──────────────────┘
                 │
         ┌───────┴───────┐
         │ tag v* push?  │
         └───────┬───────┘
                 │ yes
                 ▼
        ┌────────────────┐
        │    Release     │
        │  GitHub Release│
        │  + 压缩包上传  │
        └────────────────┘
```

---

## 六、常见问题

### Q: 如何手动重跑失败的 job？
在 GitHub Actions 页面，点击失败的 workflow run → "Re-run failed jobs"。

### Q: 如何跳过 CI？
commit message 中加 `[skip ci]` 或 `[ci skip]`。

### Q: 如何添加 secrets？
仓库 Settings → Secrets and variables → Actions → New repository secret。
在 workflow 中通过 `${{ secrets.MY_SECRET }}` 引用。

### Q: matrix.include vs 普通 matrix 有什么区别？
- 普通 `matrix`: 自动生成所有变量的笛卡尔积
- `include`: 手动列举每个具体组合，可以附加额外变量（如 `cross: true`）

### Q: `needs` 和 `if` 的区别？
- `needs`: 声明 job 依赖（控制执行顺序）
- `if`: 条件判断（控制是否执行）
