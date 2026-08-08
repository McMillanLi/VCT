// VCT 后端核心库
//
// 当前为 Step 1 阶段：完成插件注册、窗口与命令骨架搭建。
// 后续步骤将逐步填充：
//   Step 2 -> ffmpeg 模块（进程封装 / 命令生成 / 硬件检测）
//   Step 3 -> 输出日志实时解析 / 进度事件推送
//   Step 4 -> 任务队列 / 错误处理 / 完成通知

use tauri::Manager;

mod ffmpeg; // Step 2 将填充：进程封装、命令生成、硬件检测
mod tasks;  // Step 4 将填充：任务队列与生命周期管理

/// 连通性自检命令（前端启动时调用，验证 invoke 通道是否可用）
#[tauri::command]
fn ping() -> String {
    "VCT backend ready".to_string()
}

/// 返回应用基本信息
#[tauri::command]
fn app_info() -> serde_json::Value {
    serde_json::json!({
        "name": "VCT",
        "version": env!("CARGO_PKG_VERSION"),
        "stage": "step1"
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        // 注册前端可调用的命令
        .invoke_handler(tauri::generate_handler![
            ping,
            app_info,
            // Step 2 预留接口（当前为桩实现）
            ffmpeg::detect_hardware,
            ffmpeg::probe_file,
            ffmpeg::build_command,
            // Step 4 预留接口（当前为桩实现）
            tasks::start_transcode,
            tasks::cancel_transcode,
        ])
        .setup(|app| {
            log::info!("VCT 启动完成，窗口: {:?}", app.get_webview_window("main"));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("运行 VCT 时发生错误");
}
