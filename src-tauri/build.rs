fn main() {
    // 生成 Tauri 上下文（读取 tauri.conf.json），并校验 sidecar 等资源
    tauri_build::build()
}
