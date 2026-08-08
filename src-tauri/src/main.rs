// VCT 应用入口 —— Tauri v2 采用 lib + main 双文件结构，实际逻辑在 lib.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    vct_lib::run()
}
