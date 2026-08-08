// 任务管理模块（Step 4 将完整实现）
//
// 本文件在 Step 1 阶段提供类型定义与桩实现。
// Step 4 将填充：
//   - start_transcode: 启动 sidecar 子进程，通过事件实时推送进度
//   - cancel_transcode: 终止指定任务子进程并清理临时文件
//   - 多任务队列调度

use serde::{Deserialize, Serialize};

/// 转码任务请求
#[derive(Debug, Clone, Deserialize)]
pub struct TranscodeRequest {
    pub task_id: String,
    pub input_path: String,
    pub output_path: String,
    pub target_codec: String, // "h265" | "av1"
    pub preset: String,       // "fast" | "balanced" | "quality"
    pub encoder: String,
    pub duration: f64,        // 用于进度百分比计算
}

/// 转码任务结果
#[derive(Debug, Clone, Serialize)]
pub struct TranscodeResult {
    pub task_id: String,
    pub success: bool,
    pub output_path: String,
    pub message: String,
}

/// 启动转码任务
///
/// Step 4 实现：通过 tauri-plugin-shell 执行 ffmpeg sidecar，
/// 逐行读取 stderr，经 Step 3 的解析器计算进度后用 `app.emit` 推送前端。
#[tauri::command]
pub async fn start_transcode(_req: TranscodeRequest) -> Result<TranscodeResult, String> {
    // TODO(Step 4): 启动子进程 + 进度事件推送
    log::warn!("start_transcode: Step 4 之前的桩实现");
    Err("转码功能将在 Step 4 实现".to_string())
}

/// 取消转码任务
///
/// Step 4 实现：终止子进程并删除未完成的输出文件。
#[tauri::command]
pub async fn cancel_transcode(task_id: String) -> Result<bool, String> {
    // TODO(Step 4): 终止子进程 + 清理临时文件
    log::warn!("cancel_transcode: Step 4 之前的桩实现, task_id = {}", task_id);
    Ok(true)
}
