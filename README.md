# Sinter

**下一代高性能 Web 内容编译器 (The Next-Gen Web Content Compiler)**

> 🚧 **状态**: Alpha 开发预览版
>
> Sinter 是一个基于 **Rust** 和 **WebAssembly** 构建的全栈式静态站点生成系统。它不是传统的 SSG，而是一个**内容编译器**，将 Markdown 和资源编译为极其高效的单页应用 (SPA)。

---

## 🔗 在线文档

* 官网文档: https://shaogme.github.io/sinter/

## 🌟 核心特性 (Key Features)

### 1. 🚀 极致性能 (Extreme Performance)
*   **自研 Sinter UI 引擎**: 摒弃通用框架，基于 **No VDOM** 和 **细粒度响应式 (Fine-Grained Reactivity)** 构建。直接编译为 DOM 操作指令，运行时几乎零开销。
*   **WASM 驱动**: 核心逻辑编译为 WebAssembly，体积极致压缩（Full Runtime ~100KB gzipped），执行效率接近原生。

### 2. ⚡️ 智能分片 (Smart Sharding)
*   **App Shell 架构**: 首屏仅加载骨架和元数据。
*   **按需加载**: 文章详情 (`posts/*.json`) 和列表数据 (`page_*.json`) 仅在用户访问时异步获取。
*   **O(1) 部署**: 无论站点有 10 篇还是 10,000 篇文章，核心 JS/WASM 包大小恒定不变。

### 3. 🖥️ 原生级体验 (Native-like Experience)
*   **无刷新路由**: 内置基于 History API 的轻量级路由，页面切换如原生应用般流畅，无白屏、无闪烁。
*   **交互能力**: 支持复杂的交互组件，不局限于静态 HTML 的表达能力。

---

## 🏗️ 快速开始 (Getting Started)

### 环境准备 (Prerequisites)

请确保你的开发环境已安装以下工具：

1.  **Rust**: (Stable Toolchain) 用于编译 Core, CLI 和 WASM。
2.  **Node.js**: 用于运行 TailwindCSS CLI 进行样式构建。
3.  **Trunk**: Rust WASM 构建打包工具 (`cargo install trunk`)。

### 构建与运行 (Build & Run)

Sinter 的运行分为两步：首先使用 CLI 编译内容数据，然后启动 WASM 前端服务。

#### 1. 编译内容数据 (Compile Content)

运行后端 CLI 工具，扫描 `posts/` 目录并生成 JSON 分片数据到前端目录。

```bash
# 在项目根目录下运行
cargo run -p sinter_cli -- build
```

#### 2. 启动前端服务 (Serve Web App)

编译并启动前端 WASM 应用。

```bash
cd sinter_web
trunk serve --release
```

启动后，访问终端提示的地址（通常是 `http://127.0.0.1:8080`）即可预览站点。

---

## 📂 项目结构 (Project Structure)

Sinter 采用 Monorepo 结构管理核心组件：

| 目录 | 说明 |
| :--- | :--- |
| **`sinter_cli`** | **编译器 CLI**。负责 Markdown 解析 (AST)、并行构建、数据分片生成。 |
| **`sinter_core`** | **核心契约**。定义前后端通用的数据结构 (Schema) 和类型定义。 |
| **`sinter_ui`** | **UI 引擎**。自研的 Rust 响应式 Web 框架，提供 Signal, Effect, DOM 绑定等原语。 |
| **`sinter_web`** | **前端宿主程序**。WASM 应用入口，负责路由、主题加载和全局状态管理。 |
| **`sinter_themes`** | **主题包**。包含官方主题实现，完全解耦的视图层。 |
| **`theme_sdk`** | **主题开发 SDK**。定义了 `Theme` Trait，为主题开发者提供稳定的接口。 |

---

## 📚 文档 (Documentation)

更详细的架构设计与开发指南请参考 `docs/` 目录：

*   [系统架构 (Architecture)](docs/src/architecture.md)
*   [Sinter UI 响应式原理](docs/src/sinter_ui/reactivity.md)
*   [CLI 编译器设计](docs/src/sinter_cli/README.md)

你也可以在本地运行文档服务器查看：

```bash
mdbook serve docs
```

---

## 📝 许可证 (License)

MIT License.
