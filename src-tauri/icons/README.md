# 应用图标

此目录存放 Tauri 打包所需的应用图标。当前为占位。

## 生成方式（需先安装 Rust + Tauri CLI）

准备一张 1024×1024 的 PNG 源图（建议透明背景），然后执行：

```bash
npm run tauri icon path/to/your-icon.png
```

该命令会自动生成 `32x32.png`、`128x128.png`、`128x128@2x.png`、`icon.icns`、`icon.ico` 等全部所需图标。

> 在 Rust 工具链就绪前，`tauri dev` / `tauri build` 会因缺少图标而报错，这不影响 Step 1 的纯前端 UI 预览（`npm run dev:web`）。
