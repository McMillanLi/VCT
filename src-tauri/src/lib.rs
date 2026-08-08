// VCT 后端核心库
//
// 开发阶段：
//   Step 1 -> 项目结构 + UI 布局 + 命令骨架（已完成）
//   Step 2 -> ffmpeg 模块：进程封装 / 命令生成 / 硬件检测（已完成）
//   Step 3 -> progress 模块：输出日志实时解析 / 进度事件推送（当前）
//   Step 4 -> 任务队列 / 错误处理 / 完成通知

use tauri::Manager;

mod ffmpeg;   // FFmpeg 进程封装、命令生成、硬件检测、文件探测
mod progress; // FFmpeg -progress 输出解析器与进度指标计算
mod tasks;    // 转码任务执行：sidecar 流式启动 + 进度推送 + 取消清理

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
        "stage": "step3"
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
        // 注册运行中任务的子进程注册表（供 start/cancel 共享）
        .manage(tasks::TaskRegistry::new())
        // 注册前端可调用的命令
        .invoke_handler(tauri::generate_handler![
            ping,
            app_info,
            ffmpeg::detect_hardware,
            ffmpeg::probe_file,
            ffmpeg::build_command,
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
