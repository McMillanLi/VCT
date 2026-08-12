// 任务管理模块（Step 3 实现：实时转码 + 进度推送）
//
// 职责：
//   1. start_transcode —— 启动 ffmpeg sidecar 子进程，逐行读取 stdout（-progress 输出），
//      经 progress 解析器计算进度后通过 app.emit 推送前端；同时收集 stderr 错误信息。
//   2. cancel_transcode —— 终止指定任务子进程，清理未完成的输出文件，推送 canceled 状态。
//
// 子进程生命周期管理：
//   - 运行中的 CommandChild 存入 TaskRegistry（Tauri State），供 cancel_transcode 取出 kill。
//   - 自然结束时 start_transcode 从 registry 移除 child 并推送 completed/failed。
//   - 被取消时 cancel_transcode 先移除 child 再 kill，start_transcode 事件循环检测到
//     child 已不在 registry 中，判定为取消场景，清理临时文件后静默返回。

use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::{CommandChild, CommandEvent};

use crate::ffmpeg::{build_args, BuildCommandInput};
use crate::progress::{calculate_metrics, ProgressParser};

// ========================= 前后端契约类型 =========================

/// 转码任务请求
#[derive(Debug, Clone, Deserialize)]
pub struct TranscodeRequest {
    pub task_id: String,
    pub input_path: String,
    pub output_path: String,
    pub target_codec: String, // "h264" | "h265" | "av1"
    pub preset: String,       // "fast" | "balanced" | "quality" | "custom"
    pub encoder: String,
    #[serde(default)]
    pub custom_crf: Option<u32>,
    pub duration: f64, // 用于进度百分比计算
}

/// 转码任务结果
#[derive(Debug, Clone, Serialize)]
pub struct TranscodeResult {
    pub task_id: String,
    pub success: bool,
    pub output_path: String,
    pub message: String,
}

/// 推送给前端的进度事件 payload（对应前端 ProgressEvent）
#[derive(Debug, Clone, Serialize)]
struct ProgressPayload {
    task_id: String,
    progress: f64,
    speed: f64,
    fps: f64,
    processed_time: f64,
    eta: f64,
}

/// 推送给前端的状态变更事件 payload（对应前端 StatusEvent）
#[derive(Debug, Clone, Serialize)]
struct StatusPayload {
    task_id: String,
    status: String,
    message: Option<String>,
    output_path: Option<String>,
}

// ========================= 子进程注册表 =========================

/// 运行中任务的子进程注册表
///
/// key = task_id, value = CommandChild（kill 消费 self，故取出即移除）
pub struct TaskRegistry {
    children: Mutex<HashMap<String, CommandChild>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            children: Mutex::new(HashMap::new()),
        }
    }
}

// ========================= 命令实现 =========================

/// 启动转码任务
///
/// 流程：
/// 1. 构建 FFmpeg 参数 → 2. spawn sidecar → 3. 注册 child → 4. 推送 running
/// 5. 事件循环：stdout 解析进度并推送，stderr 收集错误
/// 6. 进程结束：判定取消/成功/失败，推送对应状态
#[tauri::command]
pub async fn start_transcode(
    app: AppHandle,
    registry: State<'_, TaskRegistry>,
    req: TranscodeRequest,
) -> Result<TranscodeResult, String> {
    let task_id = req.task_id.clone();
    let duration = req.duration;
    let output_path = req.output_path.clone();

    // 1. 构建 FFmpeg 参数
    let input = BuildCommandInput {
        input_path: req.input_path.clone(),
        output_path: req.output_path.clone(),
        target_codec: req.target_codec.clone(),
        preset: req.preset.clone(),
        encoder: req.encoder.clone(),
        custom_crf: req.custom_crf,
    };
    let args = build_args(&input);

    log::info!(
        "[{}] 启动转码: {} -> {} ({})",
        task_id,
        req.input_path,
        output_path,
        req.encoder
    );
    log::debug!("[{}] ffmpeg args: {:?}", task_id, args);

    // 2. 启动 sidecar 子进程
    let cmd = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("无法定位 ffmpeg sidecar: {e}"))?;
    let cmd = cmd.args(args);

    let (mut rx, child) = cmd
        .spawn()
        .map_err(|e| format!("启动 ffmpeg 失败: {e}"))?;

    // 3. 注册子进程（供 cancel_transcode 取出 kill）
    registry
        .children
        .lock()
        .unwrap()
        .insert(task_id.clone(), child);

    // 4. 通知前端：任务开始运行
    let _ = app.emit(
        "transcode://status",
        StatusPayload {
            task_id: task_id.clone(),
            status: "running".into(),
            message: None,
            output_path: None,
        },
    );

    // 5. 事件循环：逐行读取输出
    let mut parser = ProgressParser::new();
    let mut stderr_buf = String::new();
    let mut exit_code: Option<i32> = None;

    while let Some(event) = rx.recv().await {
        match event {
            CommandEvent::Stdout(line_bytes) => {
                let line = String::from_utf8_lossy(&line_bytes);
                if let Some(snapshot) = parser.feed_line(&line) {
                    let m = calculate_metrics(&snapshot, duration);
                    let _ = app.emit(
                        "transcode://progress",
                        ProgressPayload {
                            task_id: task_id.clone(),
                            progress: m.progress,
                            speed: m.speed,
                            fps: m.fps,
                            processed_time: m.processed_time,
                            eta: m.eta,
                        },
                    );
                }
            }
            CommandEvent::Stderr(line_bytes) => {
                let line = String::from_utf8_lossy(&line_bytes);
                stderr_buf.push_str(&line);
                if !stderr_buf.ends_with('\n') {
                    stderr_buf.push('\n');
                }
            }
            CommandEvent::Terminated(payload) => {
                exit_code = payload.code;
                // Terminated 是最后一条事件，循环将在下次 recv 返回 None 时退出
            }
            CommandEvent::Error(err) => {
                log::error!("[{}] ffmpeg 事件流错误: {}", task_id, err);
                stderr_buf.push_str(&format!("ffmpeg stream error: {err}\n"));
            }
            // CommandEvent 标记为 non_exhaustive，未来可能新增变体
            _ => {}
        }
    }

    // 6. 进程已结束。检查是自然结束还是被取消
    //    cancel_transcode 会先从 registry 移除 child 再 kill，
    //    所以若 child 已不在 registry 中 → 被取消
    let was_canceled = registry
        .children
        .lock()
        .unwrap()
        .remove(&task_id)
        .is_none();

    if was_canceled {
        // 清理未完成的输出文件
        let _ = std::fs::remove_file(&output_path);
        log::info!("[{}] 任务已取消，清理临时文件: {}", task_id, output_path);
        return Ok(TranscodeResult {
            task_id,
            success: false,
            output_path,
            message: "任务已取消".into(),
        });
    }

    // 7. 自然结束：根据退出码判定成功/失败
    let success = exit_code == Some(0);

    if success {
        log::info!("[{}] 转码完成: {}", task_id, output_path);
        // 推送最终 100% 进度
        let _ = app.emit(
            "transcode://progress",
            ProgressPayload {
                task_id: task_id.clone(),
                progress: 100.0,
                speed: 0.0,
                fps: 0.0,
                processed_time: duration,
                eta: 0.0,
            },
        );
        let _ = app.emit(
            "transcode://status",
            StatusPayload {
                task_id: task_id.clone(),
                status: "completed".into(),
                message: Some("转码完成".into()),
                output_path: Some(output_path.clone()),
            },
        );
        Ok(TranscodeResult {
            task_id,
            success: true,
            output_path,
            message: "转码完成".into(),
        })
    } else {
        // 提取最后几行 stderr 作为错误信息
        let msg = if stderr_buf.trim().is_empty() {
            format!("ffmpeg 异常退出 (code={:?})", exit_code)
        } else {
            stderr_buf
                .lines()
                .rev()
                .filter(|l| !l.trim().is_empty())
                .take(5)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n")
        };
        log::error!("[{}] 转码失败: {}", task_id, msg);
        // 清理不完整的输出文件
        let _ = std::fs::remove_file(&output_path);
        let _ = app.emit(
            "transcode://status",
            StatusPayload {
                task_id: task_id.clone(),
                status: "failed".into(),
                message: Some(msg.clone()),
                output_path: None,
            },
        );
        Ok(TranscodeResult {
            task_id,
            success: false,
            output_path,
            message: msg,
        })
    }
}

/// 取消转码任务
///
/// 从注册表中取出 child 并 kill，推送 canceled 状态。
/// start_transcode 的事件循环会检测到 child 已被移除，自动清理临时文件。
#[tauri::command]
pub async fn cancel_transcode(
    app: AppHandle,
    registry: State<'_, TaskRegistry>,
    task_id: String,
) -> Result<bool, String> {
    log::info!("[{}] 取消转码", task_id);

    // 先从注册表移除 child（这样 start_transcode 能判定为取消场景），再 kill
    let child = registry.children.lock().unwrap().remove(&task_id);
    if let Some(child) = child {
        let _ = child.kill();
    } else {
        log::warn!("[{}] 未找到运行中的子进程（可能已结束）", task_id);
    }

    // 推送 canceled 状态
    let _ = app.emit(
        "transcode://status",
        StatusPayload {
            task_id: task_id.clone(),
            status: "canceled".into(),
            message: Some("任务已取消".into()),
            output_path: None,
        },
    );

    Ok(true)
}
