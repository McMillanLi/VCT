// 任务状态管理：任务队列、进度更新、生命周期控制
//
// 采用模块级单例：tasks（reactive 数组）与所有 computed 均在模块作用域定义，
// useTasks() 返回对这些单例的引用，多处调用共享同一状态。
import { reactive, computed, readonly } from "vue";
import {
  startTranscode,
  cancelTranscode,
  onProgress,
  onStatus,
} from "@/api/backend";
import { useApp } from "@/stores/app";
import { defaultOutputPath, genTaskId } from "@/utils/format";
import type {
  TranscodeTask,
  FileInfo,
  ProgressEvent,
  StatusEvent,
} from "@/types";

const tasks = reactive<TranscodeTask[]>([]);

// ---- 模块级 computed（单例） ----
export const hasTasks = computed(() => tasks.length > 0);
export const pendingTasks = computed(() => tasks.filter((t) => t.status === "pending"));
export const activeTask = computed(() => tasks.find((t) => t.status === "running") || null);
export const isProcessing = computed(() => !!activeTask.value);
export const completedCount = computed(
  () => tasks.filter((t) => t.status === "completed").length,
);
export const failedCount = computed(
  () => tasks.filter((t) => t.status === "failed" || t.status === "canceled").length,
);
export const overallProgress = computed(() => {
  if (tasks.length === 0) return 0;
  const sum = tasks.reduce((acc, t) => acc + t.progress, 0);
  return Math.round(sum / tasks.length);
});

/** 计算输出路径：同目录 or 自定义目录 */
function computeOutputPath(file: FileInfo): string {
  const { state } = useApp();
  if (state.outputMode === "custom" && state.customOutputDir) {
    const base = file.name.replace(/\.[^.]+$/, "");
    const suffix = state.targetCodec === "av1" ? "_AV1" : "_H265";
    const dir = state.customOutputDir.replace(/[\\/]+$/, "");
    return `${dir}/${base}${suffix}.mp4`;
  }
  return defaultOutputPath(file.path, state.targetCodec);
}

/** 添加文件为待转码任务 */
function addFiles(files: FileInfo[]) {
  const { state, recommendedEncoder } = useApp();
  for (const file of files) {
    tasks.push({
      id: genTaskId(),
      file,
      output_path: computeOutputPath(file),
      target_codec: state.targetCodec,
      preset: state.preset,
      encoder: recommendedEncoder.value,
      status: "pending",
      progress: 0,
      speed: 0,
      fps: 0,
      processed_time: 0,
      eta: 0,
      error: "",
      created_at: Date.now(),
      started_at: null,
      finished_at: null,
    });
  }
}

/** 移除单个任务（运行中的会先取消） */
async function removeTask(id: string) {
  const idx = tasks.findIndex((t) => t.id === id);
  if (idx < 0) return;
  if (tasks[idx].status === "running") await cancelTranscode(id);
  tasks.splice(idx, 1);
}

/** 清空全部（运行中的会先取消） */
async function clearAll() {
  const running = tasks.filter((t) => t.status === "running");
  await Promise.all(running.map((t) => cancelTranscode(t.id)));
  tasks.splice(0, tasks.length);
}

/** 清空已完成/已取消/已失败的任务 */
function clearFinished() {
  for (let i = tasks.length - 1; i >= 0; i--) {
    const s = tasks[i].status;
    if (s === "completed" || s === "canceled" || s === "failed") {
      tasks.splice(i, 1);
    }
  }
}

/** 顺序启动下一个排队任务（Step 4 将完善队列调度） */
function startNextPending() {
  const next = tasks.find((t) => t.status === "pending");
  if (next) startTask(next.id);
}

/** 启动单个任务 */
async function startTask(id: string) {
  const task = tasks.find((t) => t.id === id);
  if (!task || task.status === "running") return;
  task.status = "running";
  task.started_at = Date.now();
  task.error = "";
  try {
    await startTranscode({
      task_id: task.id,
      input_path: task.file.path,
      output_path: task.output_path,
      target_codec: task.target_codec,
      preset: task.preset,
      encoder: task.encoder,
      duration: task.file.duration,
    });
  } catch (e: any) {
    task.status = "failed";
    task.error = String(e?.message ?? e);
    task.finished_at = Date.now();
    startNextPending();
  }
}

/** 启动全部待转码任务 */
function startAll() {
  if (activeTask.value) return; // 已有运行中任务，等其完成自动续接
  startNextPending();
}

/** 取消任务 */
async function cancelTask(id: string) {
  const task = tasks.find((t) => t.id === id);
  if (!task) return;
  await cancelTranscode(id);
  task.status = "canceled";
  task.finished_at = Date.now();
  startNextPending();
}

// ---- 全局事件订阅（在应用启动时调用一次） ----
let subscribed = false;
export async function subscribeTaskEvents() {
  if (subscribed) return;
  subscribed = true;
  await onProgress((e: ProgressEvent) => {
    const task = tasks.find((t) => t.id === e.task_id);
    if (!task) return;
    task.progress = e.progress;
    task.speed = e.speed;
    task.fps = e.fps;
    task.processed_time = e.processed_time;
    task.eta = e.eta;
  });
  await onStatus((e: StatusEvent) => {
    const task = tasks.find((t) => t.id === e.task_id);
    if (!task) return;
    task.status = e.status;
    if (e.output_path) task.output_path = e.output_path;
    if (e.message && e.status === "failed") task.error = e.message;
    if (["completed", "failed", "canceled"].includes(e.status)) {
      task.finished_at = Date.now();
      startNextPending(); // 完成后自动续接下一个排队任务
    }
  });
}

/** 暴露给组件的只读任务列表与操作 */
export function useTasks() {
  return {
    list: tasks,
    hasTasks,
    pendingTasks,
    activeTask,
    isProcessing,
    completedCount,
    failedCount,
    overallProgress,
    addFiles,
    removeTask,
    clearAll,
    clearFinished,
    startTask,
    startAll,
    cancelTask,
    startNextPending,
  };
}
