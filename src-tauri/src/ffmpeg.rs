// FFmpeg 模块（Step 2 完整实现）
//
// 职责：
//   1. detect_hardware —— 通过真实「测试编码」探测运行时可用的硬件编码器
//      （编译进 ffmpeg ≠ 运行时可用，必须实际初始化编码器才能确认 GPU 存在）
//   2. probe_file      —— 调用 ffprobe 获取视频元信息（JSON 输出，精确解析）
//   3. build_command   —— 根据编码目标 / 预设 / 编码器拼接 FFmpeg 参数
//
// 进程封装基于 tauri-plugin-shell 的 sidecar 机制：
//   app.shell().sidecar("ffmpeg") 自动解析到打包内/开发时的 ffmpeg 二进制。

use std::time::Duration;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::Command;

// ========================= 共享类型（前后端契约） =========================

/// 硬件加速检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub available_encoders: Vec<String>,
    pub recommended_h265: String,
    pub recommended_av1: String,
    pub gpu_vendor: String,
}

/// 视频文件元信息（ffprobe 解析结果）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub duration: f64,
    pub width: u32,
    pub height: u32,
    pub codec: String,
    pub fps: f64,
}

/// 构建命令的输入参数
#[derive(Debug, Clone, Deserialize)]
pub struct BuildCommandInput {
    pub input_path: String,
    pub output_path: String,
    pub target_codec: String,
    pub preset: String,
    pub encoder: String,
}

/// 构建命令的输出
#[derive(Debug, Clone, Serialize)]
pub struct BuiltCommand {
    pub args: Vec<String>,
    pub encoder: String,
    pub estimated_quality: String,
}

// ========================= ffprobe JSON 反序列化结构 =========================

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    #[serde(default)]
    streams: Vec<FfprobeStream>,
    #[serde(default)]
    format: Option<FfprobeFormat>,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    codec_name: Option<String>,
    codec_type: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    avg_frame_rate: Option<String>,
    r_frame_rate: Option<String>,
    duration: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    duration: Option<String>,
    size: Option<String>,
}

// ========================= sidecar 进程封装 =========================

/// 获取 ffmpeg sidecar 命令构造器
fn ffmpeg_sidecar(app: &AppHandle) -> Result<Command, String> {
    app.shell()
        .sidecar("ffmpeg")
        .map_err(|e| format!("无法定位 ffmpeg sidecar: {e}"))
}

/// 获取 ffprobe sidecar 命令构造器
fn ffprobe_sidecar(app: &AppHandle) -> Result<Command, String> {
    app.shell()
        .sidecar("ffprobe")
        .map_err(|e| format!("无法定位 ffprobe sidecar: {e}"))
}

/// 运行命令至完成，返回 (stdout, stderr, 是否成功退出)
async fn run_collect(cmd: Command, timeout_secs: u64) -> Result<(String, String, bool), String> {
    let out = tokio::time::timeout(Duration::from_secs(timeout_secs), cmd.output())
        .await
        .map_err(|_| format!("命令执行超时（{timeout_secs}s）"))?
        .map_err(|e| format!("命令执行失败: {e}"))?;
    Ok((
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.success(),
    ))
}

// ========================= 硬件检测 =========================

/// 硬件编码器候选（按优先级排序）
/// H.265: NVIDIA > Intel > AMD > MediaFoundation
const H265_HW_CANDIDATES: &[&str] = &["hevc_nvenc", "hevc_qsv", "hevc_amf", "hevc_mf"];
/// AV1:   NVIDIA > Intel > AMD > MediaFoundation
const AV1_HW_CANDIDATES: &[&str] = &["av1_nvenc", "av1_qsv", "av1_amf", "av1_mf"];

/// 用一段极短的合成视频测试某编码器是否能在运行时真正初始化
///
/// 命令：ffmpeg -f lavfi -i testsrc2=size=320x240:rate=1 -frames:v 1 -pix_fmt yuv420p
///       -c:v <enc> -f null -
///
/// 注意：尺寸不能太小。NVENC 等硬件编码器有最小帧尺寸要求（约 145px），
///       64x64 会触发 "Frame dimensions are less than the minimum supported value"。
/// 退出码 0 视为可用。
async fn test_encoder(app: &AppHandle, encoder: &str) -> bool {
    let cmd = match ffmpeg_sidecar(app) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let cmd = cmd.args([
        "-hide_banner", "-nostdin", "-loglevel", "error",
        "-f", "lavfi", "-i", "testsrc2=size=320x240:rate=1",
        "-frames:v", "1",
        "-pix_fmt", "yuv420p",
        "-c:v", encoder,
        "-f", "null", "-",
    ]);
    matches!(run_collect(cmd, 10).await, Ok((_, _, true)))
}

/// 按优先级依次测试候选编码器，返回第一个可用的
async fn first_working(app: &AppHandle, candidates: &[&str]) -> Option<String> {
    for enc in candidates {
        if test_encoder(app, enc).await {
            return Some((*enc).to_string());
        }
    }
    None
}

/// 根据编码器名推断 GPU 厂商
fn vendor_of(encoder: &str) -> &'static str {
    if encoder.contains("nvenc") {
        "nvidia"
    } else if encoder.contains("qsv") {
        "intel"
    } else if encoder.contains("amf") {
        "amd"
    } else if encoder.contains("videotoolbox") {
        "apple"
    } else {
        "cpu"
    }
}

/// 探测系统硬件加速支持
///
/// 并行测试 H.265 与 AV1 两类编码器，各取优先级最高的可用项；
/// 全部不可用时降级为 CPU 软解（libx265 / libsvtav1）。
#[tauri::command]
pub async fn detect_hardware(app: AppHandle) -> Result<HardwareInfo, String> {
    // 并行探测两种编码目标，缩短启动耗时
    let (h265_hw, av1_hw) = tokio::join!(
        first_working(&app, H265_HW_CANDIDATES),
        first_working(&app, AV1_HW_CANDIDATES),
    );

    let recommended_h265 = h265_hw
        .clone()
        .unwrap_or_else(|| "libx265".to_string());
    let recommended_av1 = av1_hw
        .clone()
        .unwrap_or_else(|| "libsvtav1".to_string());

    // 汇总可用编码器列表（去重）
    let mut available: Vec<String> = Vec::new();
    for enc in [&h265_hw, &av1_hw] {
        if let Some(e) = enc {
            if !available.contains(e) {
                available.push(e.clone());
            }
        }
    }

    // 厂商标识以 H.265 的检测结果为准
    let gpu_vendor = h265_hw
        .as_deref()
        .map(vendor_of)
        .unwrap_or("cpu")
        .to_string();

    let summary = format!(
        "硬件检测完成: vendor={}, h265={}, av1={}, available={:?}",
        gpu_vendor, recommended_h265, recommended_av1, available
    );
    log::info!("{}", summary);
    eprintln!("[VCT] {}", summary);

    Ok(HardwareInfo {
        available_encoders: available,
        recommended_h265,
        recommended_av1,
        gpu_vendor,
    })
}

// ========================= 文件探测 =========================

/// 解析帧率字符串，如 "30/1" -> 30.0、"30000/1001" -> 29.97
fn parse_fps(rate: &str) -> f64 {
    let parts: Vec<&str> = rate.split('/').collect();
    match parts.len() {
        2 => {
            let n: f64 = parts[0].parse().unwrap_or(0.0);
            let d: f64 = parts[1].parse().unwrap_or(1.0);
            if d != 0.0 {
                n / d
            } else {
                0.0
            }
        }
        _ => rate.parse().unwrap_or(0.0),
    }
}

/// 探测视频文件元信息
#[tauri::command]
pub async fn probe_file(app: AppHandle, path: String) -> Result<FileInfo, String> {
    let cmd = ffprobe_sidecar(&app)?.args([
        "-v", "error",
        "-print_format", "json",
        "-show_format", "-show_streams",
        &path,
    ]);
    let (stdout, stderr, ok) = run_collect(cmd, 30).await?;
    if !ok {
        return Err(format!("ffprobe 执行失败: {}", stderr.trim()));
    }

    let probe: FfprobeOutput = serde_json::from_str(&stdout)
        .map_err(|e| format!("解析 ffprobe 输出失败: {e}"))?;

    let vstream = probe
        .streams
        .iter()
        .find(|s| s.codec_type.as_deref() == Some("video"))
        .ok_or_else(|| "未找到视频流".to_string())?;

    // 时长：优先取流级，回退到格式级
    let duration = vstream
        .duration
        .as_ref()
        .and_then(|d| d.parse::<f64>().ok())
        .or_else(|| {
            probe
                .format
                .as_ref()
                .and_then(|f| f.duration.as_ref().and_then(|d| d.parse::<f64>().ok()))
        })
        .unwrap_or(0.0);

    let size = probe
        .format
        .as_ref()
        .and_then(|f| f.size.as_ref().and_then(|s| s.parse::<u64>().ok()))
        .unwrap_or(0);

    let fps = vstream
        .avg_frame_rate
        .as_deref()
        .or(vstream.r_frame_rate.as_deref())
        .map(parse_fps)
        .unwrap_or(0.0);

    let name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Ok(FileInfo {
        path,
        name,
        size,
        duration,
        width: vstream.width.unwrap_or(0),
        height: vstream.height.unwrap_or(0),
        codec: vstream.codec_name.clone().unwrap_or_else(|| "unknown".into()),
        fps,
    })
}

// ========================= 命令拼接 =========================

/// 根据 预设 返回 (质量值, 预设名) —— 软编用
fn software_quality(preset: &str) -> (u32, &'static str) {
    match preset {
        "fast" => (28, "ultrafast"),
        "quality" => (20, "slow"),
        _ => (24, "medium"),
    }
}

/// 构建完整的 FFmpeg 参数列表
///
/// 设计要点：
/// - 全局选项前置：-nostats -loglevel error 抑制人类可读统计，stderr 仅保留错误
/// - -progress pipe:1 将机器可读进度写入 stdout，供 Step 3 解析
/// - 视频编码器分支：NVENC / QSV / AMF / MediaFoundation / 软解 各自参数不同
/// - 音频默认 -c:a copy，保留原音轨
pub fn build_args(input: &BuildCommandInput) -> Vec<String> {
    let preset = input.preset.as_str();
    let enc = input.encoder.as_str();
    let (crf, _sw_preset) = software_quality(preset);

    let mut args: Vec<String> = Vec::with_capacity(24);
    // ---- 全局选项 ----
    args.push("-hide_banner".into());
    args.push("-nostdin".into());
    args.push("-nostats".into());
    args.push("-loglevel".into());
    args.push("error".into());
    args.push("-y".into());
    // ---- 输入 ----
    args.push("-i".into());
    args.push(input.input_path.clone());

    // ---- 视频编码器参数 ----
    if enc.contains("nvenc") {
        // NVIDIA NVENC：VBR + CQ 控制质量
        let nv_preset = match preset {
            "fast" => "p4",
            "quality" => "p7",
            _ => "p5",
        };
        let cq: u32 = match preset {
            "fast" => 30,
            "quality" => 22,
            _ => 26,
        };
        args.extend([
            "-c:v".into(), enc.into(),
            "-preset".into(), nv_preset.into(),
            "-rc".into(), "vbr".into(),
            "-cq".into(), cq.to_string(),
            "-b:v".into(), "0".into(),
            "-spatial-aq".into(), "1".into(),
        ]);
    } else if enc.contains("qsv") {
        // Intel QSV：ICQ 全局质量
        let qsv_preset = match preset {
            "fast" => "very_fast",
            "quality" => "slow",
            _ => "medium",
        };
        args.extend([
            "-c:v".into(), enc.into(),
            "-preset".into(), qsv_preset.into(),
            "-global_quality".into(), crf.to_string(),
        ]);
    } else if enc.contains("amf") {
        // AMD AMF：CQP 固定 QP
        let amf_quality = match preset {
            "fast" => "speed",
            "quality" => "quality",
            _ => "balanced",
        };
        args.extend([
            "-c:v".into(), enc.into(),
            "-quality".into(), amf_quality.into(),
            "-rc".into(), "cqp".into(),
            "-qp_i".into(), crf.to_string(),
            "-qp_p".into(), crf.to_string(),
        ]);
    } else if enc.ends_with("_mf") {
        // MediaFoundation：质量由 bitrate 控制，这里用 -quality
        args.extend([
            "-c:v".into(), enc.into(),
            "-quality".into(), preset.into(),
        ]);
    } else if enc == "libx265" {
        // 软解 H.265：CRF + preset，抑制 x265 日志噪音
        let (c, p) = software_quality(preset);
        args.extend([
            "-c:v".into(), "libx265".into(),
            "-preset".into(), p.into(),
            "-crf".into(), c.to_string(),
            "-x265-params".into(), "log-level=error".into(),
        ]);
    } else if enc == "libsvtav1" {
        // 软解 AV1：SVT-AV1，CRF 0-63（值越小质量越高）
        let svt_preset = match preset {
            "fast" => "8",
            "quality" => "3",
            _ => "5",
        };
        let av1_crf: u32 = match preset {
            "fast" => 32,
            "quality" => 24,
            _ => 28,
        };
        args.extend([
            "-c:v".into(), "libsvtav1".into(),
            "-preset".into(), svt_preset.into(),
            "-crf".into(), av1_crf.to_string(),
        ]);
    } else {
        // 未知编码器：直接透传
        args.extend(["-c:v".into(), enc.into()]);
    }

    // ---- 音频：直接拷贝 ----
    args.push("-c:a".into());
    args.push("copy".into());

    // ---- 进度输出到 stdout（供 Step 3 progress 解析器实时解析）----
    // 放在输出文件之前：-progress 是输出级选项，FFmpeg 会将其归属到紧随其后的输出。
    args.push("-progress".into());
    args.push("pipe:1".into());

    // ---- 输出文件（必须是最后一个位置参数）----
    args.push(input.output_path.clone());

    args
}

/// 构建转码命令（前端可调用预览实际参数）
#[tauri::command]
pub async fn build_command(input: BuildCommandInput) -> Result<BuiltCommand, String> {
    let encoder = input.encoder.clone();
    let estimated_quality = input.preset.clone();
    let args = build_args(&input);
    Ok(BuiltCommand {
        args,
        encoder,
        estimated_quality,
    })
}

// ========================= 单元测试 =========================

#[cfg(test)]
mod tests {
    use super::*;

    fn input(encoder: &str, preset: &str) -> BuildCommandInput {
        BuildCommandInput {
            input_path: "in.mp4".into(),
            output_path: "out.mp4".into(),
            target_codec: if encoder.contains("av1") { "av1" } else { "h265" }.into(),
            preset: preset.into(),
            encoder: encoder.into(),
        }
    }

    /// 辅助：断言 args 中连续包含某段参数
    fn assert_has(args: &[String], kv: &[&str]) {
        for i in 0..=args.len().saturating_sub(kv.len()) {
            let window = &args[i..i + kv.len()];
            if window.iter().zip(kv.iter()).all(|(a, k)| a.as_str() == *k) {
                return;
            }
        }
        panic!("参数中未找到 {:?}，实际: {:?}", kv, args);
    }

    #[test]
    fn build_args_global_options() {
        let args = build_args(&input("libx265", "balanced"));
        assert!(args.contains(&"-hide_banner".into()));
        assert!(args.contains(&"-nostdin".into()));
        assert!(args.contains(&"-nostats".into()));
        assert!(args.contains(&"-y".into()));
        assert!(args.contains(&"-i".into()));
        assert!(args.contains(&"-c:a".into()));
        assert!(args.contains(&"copy".into()));
        assert!(args.contains(&"-progress".into()));
        assert!(args.contains(&"pipe:1".into()));
    }

    #[test]
    fn build_args_nvenc() {
        let args = build_args(&input("hevc_nvenc", "balanced"));
        assert_has(&args, &["-c:v", "hevc_nvenc"]);
        assert_has(&args, &["-preset", "p5"]);
        assert_has(&args, &["-rc", "vbr"]);
        assert_has(&args, &["-cq", "26"]);
        assert_has(&args, &["-b:v", "0"]);
    }

    #[test]
    fn build_args_qsv() {
        let args = build_args(&input("hevc_qsv", "quality"));
        assert_has(&args, &["-c:v", "hevc_qsv"]);
        assert_has(&args, &["-preset", "slow"]);
        assert_has(&args, &["-global_quality", "20"]);
    }

    #[test]
    fn build_args_amf() {
        let args = build_args(&input("av1_amf", "fast"));
        assert_has(&args, &["-c:v", "av1_amf"]);
        assert_has(&args, &["-quality", "speed"]);
        assert_has(&args, &["-rc", "cqp"]);
    }

    #[test]
    fn build_args_libx265() {
        let args = build_args(&input("libx265", "quality"));
        assert_has(&args, &["-c:v", "libx265"]);
        assert_has(&args, &["-preset", "slow"]);
        assert_has(&args, &["-crf", "20"]);
        assert!(args.contains(&"-x265-params".into()));
    }

    #[test]
    fn build_args_libsvtav1() {
        let args = build_args(&input("libsvtav1", "fast"));
        assert_has(&args, &["-c:v", "libsvtav1"]);
        assert_has(&args, &["-preset", "8"]);
        assert_has(&args, &["-crf", "32"]);
    }

    #[test]
    fn parse_fps_handles_fractions() {
        assert_eq!(parse_fps("30/1"), 30.0);
        assert!((parse_fps("30000/1001") - 29.970029).abs() < 1e-6);
        assert_eq!(parse_fps("24"), 24.0);
        assert_eq!(parse_fps("0/0"), 0.0);
    }

    #[test]
    fn vendor_detection() {
        assert_eq!(vendor_of("hevc_nvenc"), "nvidia");
        assert_eq!(vendor_of("av1_qsv"), "intel");
        assert_eq!(vendor_of("hevc_amf"), "amd");
        assert_eq!(vendor_of("libx265"), "cpu");
    }
}
