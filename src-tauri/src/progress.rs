// FFmpeg -progress pipe:1 输出解析器（Step 3 核心）
//
// FFmpeg 的 -progress 选项将机器可读的 key=value 行写入指定输出（这里是 stdout）。
// 每组统计以 progress=continue 或 progress=end 结尾，形成一次完整的进度快照。
//
// 典型输出（每组）：
//   frame=100
//   fps=30.5
//   stream_0_0_q=23.0
//   bitrate=5000.0kbits/s
//   total_size=5000000
//   out_time_us=3333333          ← 已处理时长（微秒），首选字段
//   out_time_ms=3333333          ← 历史遗留 bug：实际也是微秒，仅作回退
//   out_time=00:00:03.333333     ← 人类可读，最终回退
//   dup_frames=0
//   drop_frames=0
//   speed=1.5x
//   progress=continue            ← 一组统计结束标志
//
// 解析策略：逐行喂入，累积字段；遇到 progress= 行时产出一次 ProgressSnapshot。

use serde::Serialize;

// ========================= 进度快照 =========================

/// 单次进度快照（解析一组 -progress 输出后产出）
#[derive(Debug, Clone, Serialize, Default)]
pub struct ProgressSnapshot {
    /// 已编码帧数
    pub frame: u64,
    /// 当前处理帧率
    pub fps: f64,
    /// 已处理时长（秒）
    pub processed_time: f64,
    /// 转码速度倍率，如 1.5 表示 1.5x 实时
    pub speed: f64,
    /// 已写出字节数
    pub total_size: u64,
    /// 当前比特率（kbits/s）
    pub bitrate_kbps: f64,
    /// 是否为最终统计（progress=end）
    pub is_final: bool,
}

// ========================= 解析器 =========================

/// 进度解析器：逐行喂入 -progress 输出，每组结束时产出 ProgressSnapshot
///
/// 用法：
/// ```ignore
/// let mut parser = ProgressParser::new();
/// for line in stdout_lines {
///     if let Some(snapshot) = parser.feed_line(&line) {
///         let metrics = calculate_metrics(&snapshot, total_duration);
///         // 推送进度到前端...
///     }
/// }
/// ```
#[derive(Default)]
pub struct ProgressParser {
    frame: u64,
    fps: f64,
    out_time_us: u64,
    speed: f64,
    total_size: u64,
    bitrate_kbps: f64,
}

impl ProgressParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// 喂入一行输出。若该行是 progress= 结束标志，返回累积的快照。
    pub fn feed_line(&mut self, line: &str) -> Option<ProgressSnapshot> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        // progress= 行标志着一组统计结束
        if let Some(value) = line.strip_prefix("progress=") {
            let is_final = value.trim() == "end";
            let snapshot = ProgressSnapshot {
                frame: self.frame,
                fps: self.fps,
                processed_time: self.out_time_us as f64 / 1_000_000.0,
                speed: self.speed,
                total_size: self.total_size,
                bitrate_kbps: self.bitrate_kbps,
                is_final,
            };
            return Some(snapshot);
        }

        // 解析 key=value
        let (key, value) = match line.split_once('=') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => return None,
        };

        match key {
            "frame" => self.frame = value.parse().unwrap_or(0),
            "fps" => self.fps = value.parse().unwrap_or(0.0),
            "out_time_us" => self.out_time_us = value.parse().unwrap_or(0),
            "out_time_ms" => {
                // FFmpeg 历史遗留 bug：out_time_ms 实际单位也是微秒。
                // 仅在 out_time_us 未更新时作为回退。
                if self.out_time_us == 0 {
                    self.out_time_us = value.parse().unwrap_or(0);
                }
            }
            "out_time" => {
                // HH:MM:SS.ffffff 格式，仅在前两者都缺失时解析
                if self.out_time_us == 0 {
                    self.out_time_us = parse_timecode(value);
                }
            }
            "speed" => {
                // "1.5x" -> 1.5
                self.speed = value
                    .trim_end_matches('x')
                    .trim_end_matches('X')
                    .parse()
                    .unwrap_or(0.0);
            }
            "total_size" => self.total_size = value.parse().unwrap_or(0),
            "bitrate" => {
                // "5000.0kbits/s" -> 5000.0
                self.bitrate_kbps = value
                    .trim_end_matches("kbits/s")
                    .parse()
                    .unwrap_or(0.0);
            }
            _ => {} // 忽略 stream_*_q、dup_frames、drop_frames 等字段
        }

        None
    }
}

// ========================= 辅助解析 =========================

/// 解析 FFmpeg 时间码 "HH:MM:SS.ffffff" -> 微秒
fn parse_timecode(s: &str) -> u64 {
    let parts: Vec<&str> = s.split(':').collect();
    let (h, m, sec) = match parts.len() {
        3 => (parts[0], parts[1], parts[2]),
        2 => ("0", parts[0], parts[1]),
        1 => ("0", "0", parts[0]),
        _ => return 0,
    };
    let h: u64 = h.parse().unwrap_or(0);
    let m: u64 = m.parse().unwrap_or(0);
    let sec: f64 = sec.parse().unwrap_or(0.0);
    (h * 3_600 + m * 60) * 1_000_000 + (sec * 1_000_000.0) as u64
}

// ========================= 进度指标计算 =========================

/// 根据进度快照和视频总时长，计算前端需要的进度指标
#[derive(Debug, Clone)]
pub struct ProgressMetrics {
    /// 进度百分比 0-100
    pub progress: f64,
    /// 转码速度倍率
    pub speed: f64,
    /// 当前 fps
    pub fps: f64,
    /// 已转码时长（秒）
    pub processed_time: f64,
    /// 预计剩余时间（秒）
    pub eta: f64,
}

/// 由快照 + 总时长计算进度百分比与 ETA
pub fn calculate_metrics(snapshot: &ProgressSnapshot, duration: f64) -> ProgressMetrics {
    let processed = snapshot.processed_time;
    let progress = if duration > 0.0 {
        (processed / duration * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };
    let eta = if duration > 0.0 && snapshot.speed > 0.0 {
        ((duration - processed) / snapshot.speed).max(0.0)
    } else {
        0.0
    };
    ProgressMetrics {
        progress,
        speed: snapshot.speed,
        fps: snapshot.fps,
        processed_time: processed,
        eta,
    }
}

// ========================= 单元测试 =========================

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一组完整的 -progress 输出并逐行喂入解析器
    fn parse_block(lines: &[&str]) -> Vec<ProgressSnapshot> {
        let mut parser = ProgressParser::new();
        let mut snapshots = Vec::new();
        for line in lines {
            if let Some(s) = parser.feed_line(line) {
                snapshots.push(s);
            }
        }
        snapshots
    }

    #[test]
    fn parse_single_progress_block() {
        let lines = [
            "frame=150",
            "fps=30.0",
            "stream_0_0_q=23.0",
            "bitrate=2000.0kbits/s",
            "total_size=5000000",
            "out_time_us=5000000",
            "out_time_ms=5000000",
            "out_time=00:00:05.000000",
            "dup_frames=0",
            "drop_frames=0",
            "speed=1.0x",
            "progress=continue",
        ];
        let snapshots = parse_block(&lines);
        assert_eq!(snapshots.len(), 1);
        let s = &snapshots[0];
        assert_eq!(s.frame, 150);
        assert!((s.fps - 30.0).abs() < 1e-6);
        assert!((s.processed_time - 5.0).abs() < 1e-6);
        assert!((s.speed - 1.0).abs() < 1e-6);
        assert_eq!(s.total_size, 5_000_000);
        assert!((s.bitrate_kbps - 2000.0).abs() < 1e-6);
        assert!(!s.is_final);
    }

    #[test]
    fn parse_multiple_blocks() {
        let lines = [
            "frame=100", "fps=30.0", "out_time_us=3333333", "speed=1.0x", "progress=continue",
            "frame=200", "fps=31.0", "out_time_us=6666666", "speed=1.1x", "progress=continue",
            "frame=300", "fps=30.5", "out_time_us=10000000", "speed=1.05x", "progress=end",
        ];
        let snapshots = parse_block(&lines);
        assert_eq!(snapshots.len(), 3);
        assert!((snapshots[0].processed_time - 3.333333).abs() < 1e-3);
        assert!((snapshots[1].processed_time - 6.666666).abs() < 1e-3);
        assert!((snapshots[2].processed_time - 10.0).abs() < 1e-6);
        assert!(snapshots[2].is_final);
    }

    #[test]
    fn parse_ignores_unknown_keys() {
        let lines = [
            "frame=10", "fps=30.0", "out_time_us=333333", "speed=0.5x",
            "stream_0_0_q=20.0", "dup_frames=2", "drop_frames=1", "nonsense_field=abc",
            "progress=continue",
        ];
        let snapshots = parse_block(&lines);
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].frame, 10);
    }

    #[test]
    fn parse_empty_and_malformed_lines() {
        let mut parser = ProgressParser::new();
        assert!(parser.feed_line("").is_none());
        assert!(parser.feed_line("   ").is_none());
        assert!(parser.feed_line("no_equals_sign").is_none());
        assert!(parser.feed_line("frame=abc").is_none()); // 非数字，应被忽略
        assert!(parser.feed_line("progress=continue").is_some()); // 产出空快照
    }

    #[test]
    fn calculate_metrics_normal() {
        let snapshot = ProgressSnapshot {
            frame: 300,
            fps: 30.0,
            processed_time: 10.0,
            speed: 2.0,
            total_size: 0,
            bitrate_kbps: 0.0,
            is_final: false,
        };
        let m = calculate_metrics(&snapshot, 100.0);
        assert!((m.progress - 10.0).abs() < 1e-6);
        assert!((m.speed - 2.0).abs() < 1e-6);
        assert!((m.eta - 45.0).abs() < 1e-6); // (100-10)/2 = 45
    }

    #[test]
    fn calculate_metrics_clamps_progress() {
        let snapshot = ProgressSnapshot {
            processed_time: 150.0,
            speed: 1.0,
            ..Default::default()
        };
        let m = calculate_metrics(&snapshot, 100.0);
        assert!((m.progress - 100.0).abs() < 1e-6); // 不超过 100
    }

    #[test]
    fn calculate_metrics_zero_duration() {
        let snapshot = ProgressSnapshot {
            processed_time: 10.0,
            speed: 2.0,
            ..Default::default()
        };
        let m = calculate_metrics(&snapshot, 0.0);
        assert!((m.progress - 0.0).abs() < 1e-6);
        assert!((m.eta - 0.0).abs() < 1e-6);
    }

    #[test]
    fn calculate_metrics_zero_speed() {
        let snapshot = ProgressSnapshot {
            processed_time: 10.0,
            speed: 0.0,
            ..Default::default()
        };
        let m = calculate_metrics(&snapshot, 100.0);
        assert!((m.progress - 10.0).abs() < 1e-6);
        assert!((m.eta - 0.0).abs() < 1e-6); // speed=0 时 ETA=0
    }

    #[test]
    fn parse_timecode_various() {
        assert_eq!(parse_timecode("00:00:05.000000"), 5_000_000);
        assert_eq!(parse_timecode("00:01:30.500000"), 90_500_000);
        assert_eq!(parse_timecode("01:00:00.000000"), 3_600_000_000);
        assert_eq!(parse_timecode("3.500000"), 3_500_000);
        assert_eq!(parse_timecode("invalid"), 0);
    }

    #[test]
    fn out_time_us_preferred_over_ms() {
        let mut parser = ProgressParser::new();
        parser.feed_line("out_time_us=5000000");
        parser.feed_line("out_time_ms=9999999"); // 不应覆盖
        let s = parser.feed_line("progress=continue").unwrap();
        assert!((s.processed_time - 5.0).abs() < 1e-6);
    }

    #[test]
    fn out_time_ms_fallback() {
        let mut parser = ProgressParser::new();
        parser.feed_line("out_time_ms=3000000"); // 无 out_time_us，用作回退
        let s = parser.feed_line("progress=continue").unwrap();
        assert!((s.processed_time - 3.0).abs() < 1e-6);
    }
}
