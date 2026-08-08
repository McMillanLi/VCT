# VCT — 极简可视化视频转码工具

基于 **Tauri v2 + Vue 3 + TypeScript + Vite** 与内置 **FFmpeg sidecar** 的桌面端视频转码应用。
将 MP4 等视频高效转换为 **H.265 (HEVC)** 或 **AV1** 编码，支持硬件加速检测与可视化进度。

## 核心特性

- 🎯 **极简流程**：拖拽/选择 → 选编码与预设 → 开始 → 实时进度
- ⚡ **硬件加速**：自动检测 NVIDIA NVENC / AMD AMF / Intel QSV / VideoToolbox，智能推荐编码器
- 📊 **可视化进度**：实时百分比、转码速度倍率、ETA、平滑进度条
- 📁 **批处理**：多文件拖拽、顺序转码队列
- 🛡️ **健壮交互**：中途取消、清理临时文件、完成通知、一键打开输出目录

## 目录结构

```
VCT/
├── src/                      # 前端（Vue 3）
│   ├── api/backend.ts        # 后端通信封装（含 Web mock 回退）
│   ├── components/           # UI 组件
│   │   ├── TitleBar.vue      # 自定义标题栏 + 窗口控制
│   │   ├── DropZone.vue      # 拖拽区 + 文件选择
│   │   ├── ConfigPanel.vue   # 编码/预设/输出配置
│   │   ├── ProgressPanel.vue # 进度面板容器
│   │   └── TaskCard.vue      # 单个任务卡片
│   ├── stores/               # 响应式状态（app / tasks）
│   ├── styles/               # 设计令牌 + 全局样式
│   ├── types/index.ts        # 前后端共享类型
│   └── utils/format.ts       # 格式化工具
├── src-tauri/                # Rust 后端
│   ├── src/
│   │   ├── main.rs           # 入口
│   │   ├── lib.rs            # 应用构建 + 命令注册
│   │   ├── ffmpeg.rs         # FFmpeg 封装（Step 2）
│   │   └── tasks.rs          # 任务管理（Step 4）
│   ├── capabilities/         # Tauri v2 权限声明
│   ├── binaries/             # FFmpeg sidecar 二进制（需手动放置）
│   ├── icons/                # 应用图标（需 tauri icon 生成）
│   └── tauri.conf.json
├── package.json
└── vite.config.ts
```

## 开发

### 前置条件

| 依赖 | 用途 | 安装 |
|------|------|------|
| Node.js ≥ 18 | 前端构建 | https://nodejs.org |
| Rust (stable) | 后端编译 | https://rustup.rs |
| FFmpeg 二进制 | 转码引擎 | 见下方「Sidecar 配置」 |

### 安装依赖

```bash
npm install
```

### 纯前端预览（无需 Rust）

```bash
npm run dev:web
```

> 此模式使用 mock 数据模拟转码进度，便于独立验证 UI。
> 访问 http://localhost:1420

### 完整桌面开发（需 Rust + FFmpeg sidecar）

```bash
npm run dev
```

### 打包发布

```bash
npm run build:tauri
```

## Sidecar 配置（FFmpeg）

1. 下载静态编译的 FFmpeg（推荐 [gyan.dev](https://www.gyan.dev/ffmpeg/builds/) 的 `ffmpeg-release-full.7z`）
2. 将 `ffmpeg.exe` 重命名为带目标三元组的名称并放入 `src-tauri/binaries/`：
   ```
   src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe
   ```
3. （可选）若需文件探测，同样放置 `ffprobe-x86_64-pc-windows-msvc.exe` 并在 `tauri.conf.json` 的 `externalBin` 中声明

> 命名规则：Tauri 会自动为 `externalBin` 中的条目追加当前平台的目标三元组后缀。

## 开发阶段进度

- [x] **Step 1**：项目结构 + 现代 UI 框架与核心布局
- [ ] **Step 2**：FFmpeg 进程封装 + 命令生成 + 硬件检测
- [ ] **Step 3**：输出日志实时解析 + 进度提取 + UI 响应式
- [ ] **Step 4**：多任务队列 + 错误捕捉 + 完成通知
