# VCT (Video Converter)— 极简可视化视频转码工具

让小白也能上手操作的极简可视化视频转码工具。基于 **Tauri v2 + Vue 3 + TypeScript + Vite** 与内置 **FFmpeg sidecar** 的桌面端视频转码应用。
支持在 **H.264 / H.265 (HEVC) / AV1** 三种编码之间灵活互转，智能硬件加速检测、可视化进度、自定义画质参数，一键完成视频转码。

![tech](https://img.shields.io/badge/Tauri-v2-orange) ![tech](https://img.shields.io/badge/Vue-3-brightgreen) ![tech](https://img.shields.io/badge/Rust-stable-dea584) ![tech](https://img.shields.io/badge/FFmpeg-sidecar-blueviolet)

## 核心特性

- 🎯 **极简流程**：拖拽/选择 → 选编码与预设 → 开始 → 实时进度
- 🔄 **三编码互转**：支持 H.264 / H.265 (HEVC) / AV1 之间灵活转换，兼顾兼容性与压缩率
- ⚡ **硬件加速**：自动检测 NVIDIA NVENC / AMD AMF / Intel QSV，智能推荐编码器
- 🎚️ **多档画质 + 自定义**：极速 / 均衡 / 高质量三档预设，进阶用户可手动输入 CRF / CQ 值
- 📊 **可视化进度**：实时百分比、转码速度倍率、fps、ETA、平滑进度条
- 🛡️ **防误操作确认**：检测到源编码与目标编码一致时弹窗确认，避免无效转码
- 📁 **批量队列**：多文件拖拽、顺序转码、暂停/恢复队列、任务排序
- 🔔 **完成通知**：转码完成/失败时推送系统桌面通知，批量完成汇总
- 🔁 **失败重试**：一键重试失败任务，可展开查看完整错误日志
- 🧹 **健壮交互**：中途取消、自动清理临时文件、一键打开输出目录

## 截图

<!-- 截图占位：发布后可替换为实际应用截图 -->
```
┌─────────────────────────────────────────────┐
│  VCT 视频转码                          ─ □ ✕ │
├─────────────────────────────────────────────┤
│  ┌─────────────────────────────────────┐    │
│  │     拖拽视频文件到此处                │    │
│  │     或 点击选择 · 支持批量添加        │    │
│  └─────────────────────────────────────┘    │
│                                             │
│  目标编码  [H.264] [H.265] [AV1]            │
│  硬件加速  NVIDIA NVENC ✅                   │
│                                             │
│  画质预设  [极速] [均衡] [高质量] [自定义]   │
│            CRF/CQ: [ 23  ]                  │
│                                             │
│  输出位置  [同目录] [自定义目录]             │
│  完成通知  [● ON]                            │
│                                             │
│  转码列表  共 3 个 · 已完成 1                │
│  ┌─────────────────────────────────────┐    │
│  │ video_01.mp4  → H.264  ✅ 已完成     │    │
│  │ ████████████████████████████ 100%   │    │
│  ├─────────────────────────────────────┤    │
│  │ video_02.mp4  → H.265  🔄 转码中     │    │
│  │ ██████████░░░░░░░░░░░░░░░░ 42%      │    │
│  │ 42%  2.15x  65fps  已转 1:20  剩 1m │    │
│  ├─────────────────────────────────────┤    │
│  │ video_03.mp4  → AV1    ⏳ 排队中     │    │
│  └─────────────────────────────────────┘    │
│                                             │
│  ⚠️ 编码一致确认弹窗:                        │
│  ┌─────────────────────────────────────┐    │
│  │ 当前视频编码格式与目标编码格式一致。   │    │
│  │ 确定仍要转换吗？                      │    │
│  │                    [取消]  [确定]    │    │
│  └─────────────────────────────────────┘    │
├─────────────────────────────────────────────┤
│  2 个任务待转码            [清空] [开始转码] │
└─────────────────────────────────────────────┘
```

## 目录结构

```
VCT/
├── src/                          # 前端（Vue 3 + TypeScript）
│   ├── api/backend.ts            # 后端通信封装（含 Web mock 回退）
│   ├── components/               # UI 组件
│   │   ├── TitleBar.vue          # 自定义标题栏 + 窗口控制
│   │   ├── DropZone.vue          # 拖拽区 + 文件选择
│   │   ├── ConfigPanel.vue       # 编码/预设/输出/通知配置
│   │   ├── ProgressPanel.vue     # 进度面板 + 队列控制
│   │   └── TaskCard.vue          # 任务卡片（进度/重试/排序/错误展开）
│   ├── stores/                   # 响应式状态（composable 单例）
│   │   ├── app.ts                # 硬件信息/编码配置/通知开关
│   │   └── tasks.ts             # 任务队列/进度/重试/暂停
│   ├── styles/                   # 设计令牌 + 全局样式
│   ├── types/index.ts            # 前后端共享类型定义
│   └── utils/
│       ├── format.ts             # 格式化工具（时长/体积/速度/ETA）
│       └── notify.ts             # 系统通知工具
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── main.rs               # 入口
│   │   ├── lib.rs                # 应用构建 + 命令注册 + 状态管理
│   │   ├── ffmpeg.rs             # FFmpeg 封装：硬件检测/文件探测/命令拼接
│   │   ├── progress.rs           # -progress 输出解析器 + 进度指标计算
│   │   └── tasks.rs              # 转码执行：流式进度推送/取消/清理
│   ├── capabilities/             # Tauri v2 权限声明
│   ├── binaries/                 # FFmpeg sidecar 二进制（需手动放置）
│   └── tauri.conf.json
├── package.json
└── vite.config.ts
```

## 技术架构

### 前后端通信

```
┌──────────────────────────────────────────────────────────┐
│  Frontend (Vue 3)                                        │
│                                                          │
│  DropZone ──► probeFile() ──► Task[]                     │
│  ConfigPanel ──► AppState (codec/preset/output)         │
│                                                          │
│  startAll() ──► invoke("start_transcode") ──┐            │
│                                              │            │
│  listen("transcode://progress") ◄────────── │ ──────────┐│
│  listen("transcode://status")   ◄────────── │ ────┐    ││
│                                              │     │    ││
└──────────────────────────────────────────────┼─────┼────┼┘
                                               │     │    │
┌──────────────────────────────────────────────┼─────┼────┼┘
│  Backend (Rust)                              │     │    │
│                                              ▼     ▼    │
│  start_transcode()                           emit emit  │
│    ├─ build_args() ──► FFmpeg CLI args               │  │
│    ├─ sidecar("ffmpeg").spawn()                       │  │
│    │    └─ stdout ──► ProgressParser ──► emit ───────┼──┘
│    │    └─ stderr ──► error buffer                    │
│    └─ Terminated ──► emit("completed"/"failed") ─────┘
│                                                        │
│  cancel_transcode()                                    │
│    ├─ registry.remove(child) ──► child.kill()          │
│    └─ emit("canceled")                                 │
└────────────────────────────────────────────────────────┘
```

### FFmpeg 进度解析

FFmpeg 的 `-progress pipe:1` 选项将机器可读的 `key=value` 行输出到 stdout，每组统计以 `progress=continue`/`progress=end` 结尾。`ProgressParser` 逐行喂入、累积字段，在遇到 `progress=` 行时产出一次快照：

| 字段 | 说明 | 解析方式 |
|------|------|----------|
| `out_time_us` | 已处理时长（微秒） | `÷ 1e6` → 秒 |
| `fps` | 当前帧率 | 直接取值 |
| `speed` | 速度倍率 | 去除 `x` 后缀 |
| `frame` | 已编码帧数 | 直接取值 |
| `total_size` | 已写出字节 | 直接取值 |
| `bitrate` | 当前比特率 | 去除 `kbits/s` 后缀 |

进度百分比 = `out_time_us ÷ 1e6 ÷ duration × 100`，ETA = `(duration - processed) ÷ speed`。

### 硬件加速检测

通过「真实测试编码」探测运行时可用的硬件编码器（编译支持 ≠ 运行时可用）：

```
优先级：NVENC > QSV > AMF > MediaFoundation > CPU 软解
```

每种编码器用 `ffmpeg -f lavfi -i testsrc2=size=320x240 -frames:v 1 -c:v <enc> -f null -` 测试，退出码 0 视为可用。

### 任务队列

- **顺序执行**：同一时间只运行一个转码任务（避免 GPU 资源争用）
- **自动续接**：任务完成/失败/取消后自动启动下一个 pending 任务
- **暂停/恢复**：暂停后当前任务继续运行，完成后不再续接；恢复后自动续接
- **取消**：`cancel_transcode` 先从注册表移除 child 再 kill，`start_transcode` 事件循环检测到 child 已不在注册表中，判定为取消场景，清理临时文件

## Sidecar 配置（FFmpeg）

1. 下载静态编译的 FFmpeg（推荐 [gyan.dev](https://www.gyan.dev/ffmpeg/builds/) 的 `ffmpeg-release-full.7z`）
2. 将 `ffmpeg.exe` 和 `ffprobe.exe` 重命名为带目标三元组的名称并放入 `src-tauri/binaries/`：
   ```
   src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe
   src-tauri/binaries/ffprobe-x86_64-pc-windows-msvc.exe
   ```
3. `tauri.conf.json` 中已声明 `externalBin: ["binaries/ffmpeg", "binaries/ffprobe"]`

> 命名规则：Tauri 会自动为 `externalBin` 中的条目追加当前平台的目标三元组后缀。

## 支持的编码器

| 编码目标 | 硬件加速 | 软解（回退） |
|---------|---------|-------------|
| H.264/AVC | `h264_nvenc` (NVIDIA) · `h264_qsv` (Intel) · `h264_amf` (AMD) | `libx264` |
| H.265/HEVC | `hevc_nvenc` (NVIDIA) · `hevc_qsv` (Intel) · `hevc_amf` (AMD) | `libx265` |
| AV1 | `av1_nvenc` (NVIDIA) · `av1_qsv` (Intel) · `av1_amf` (AMD) | `libsvtav1` |

> **编码选择建议**
> - **H.264**：兼容性最佳，几乎所有设备都支持，适合需要在老设备/浏览器上播放的场景
> - **H.265**：压缩率约为 H.264 的 2 倍，画质相同体积更小，中高端设备普遍支持
> - **AV1**：最新一代编码，压缩率优于 H.265，适合追求极致压缩比且硬件支持的用户



## License

本项目基于 [Apache License 2.0](./LICENSE) 开源。

Copyright 2026 McMillanLi

> 注意：本项目内置并调用 FFmpeg（基于 GPLv2+/LGPL 协议）作为 sidecar 二进制。FFmpeg 二进制本身遵循其各自协议，使用者需自行遵守 FFmpeg 的许可条款。
