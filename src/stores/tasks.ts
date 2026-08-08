// 任务状态管理：任务队列、进度更新、生命周期控制
//
// 采用模块级单例：tasks（reactive 数组）与所有 computed 均在模块作用域定义，
// useTasks() 返回对这些单例的引用，多处调用共享同一状态。
import { reactive, ref, computed } from "vue";
import {
  startTranscode,
  cancelTranscode,
  onProgress,
  onStatus,
} from "@/api/backend";
import { useApp } from "@/stores/app";
import { defaultOutputPath, genTaskId } from "@/utils/format";
import { notifyTaskCompleted, notifyTaskFailed, notifyAllDone } from "@/utils/notify";
import type {
  TranscodeTask,
  FileInfo,
  ProgressEvent,
  StatusEvent,
} from "@/types";

const tasks = reactive<TranscodeTask[]>([]);

/** 队列暂停标志：暂停后任务完成不再自动续接下一个 */
const isQueuePaused = ref(false);

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
/** 队列是否全部处理完毕（无运行中、无待处理） */
export const isAllDone = computed(
  () => tasks.length > 0 && !activeTask.value && pendingTasks.value.length === 0,
);

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

/** 顺序启动下一个排队任务（队列暂停时不续接） */
function startNextPending() {
  if (isQueuePaused.value) return;
  const next = tasks.find((t) => t.status === "pending");
  if (next) startTask(next.id);
}

/** 检查是否全部完成，若是则推送汇总通知 */
function checkAllDoneAndNotify() {
  if (isAllDone.value) {
    const { state } = useApp();
    if (state.notificationsEnabled) {
      notifyAllDone(completedCount.value, failedCount.value);
    }
  }
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
    // invoke 本身抛错（非 ffmpeg 退出码错误），标记失败
    task.status = "failed";
    task.error = String(e?.message ?? e);
    task.finished_at = Date.now();
    startNextPending();
    checkAllDoneAndNotify();
  }
}

/** 启动全部待转码任务 */
function startAll() {
  if (activeTask.value) return; // 已有运行中任务，等其完成自动续接
  isQueuePaused.value = false;
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
  checkAllDoneAndNotify();
}

/** 重试失败/已取消的任务：重置为 pending 并尝试启动 */
function retryTask(id: string) {
  const task = tasks.find((t) => t.id === id);
  if (!task) return;
  if (task.status !== "failed" && task.status !== "canceled") return;
  task.status = "pending";
  task.progress = 0;
  task.speed = 0;
  task.fps = 0;
  task.processed_time = 0;
  task.eta = 0;
  task.error = "";
  task.started_at = null;
  task.finished_at = null;
  // 若当前无运行中任务则立即启动，否则等队列自动续接
  if (!activeTask.value) startTask(id);
}

/** 将任务上移（仅 pending 任务可移动） */
function moveTaskUp(id: string) {
  const idx = tasks.findIndex((t) => t.id === id);
  if (idx <= 0) return;
  if (tasks[idx].status !== "pending") return;
  // 与前一个 pending 任务交换位置
  for (let i = idx - 1; i >= 0; i--) {
    if (tasks[i].status === "pending") {
      [tasks[i], tasks[idx]] = [tasks[idx], tasks[i]];
      return;
    }
  }
}

/** 将任务下移（仅 pending 任务可移动） */
function moveTaskDown(id: string) {
  const idx = tasks.findIndex((t) => t.id === id);
  if (idx < 0 || idx >= tasks.length - 1) return;
  if (tasks[idx].status !== "pending") return;
  // 与后一个 pending 任务交换位置
  for (let i = idx + 1; i < tasks.length; i++) {
    if (tasks[i].status === "pending") {
      [tasks[i], tasks[idx]] = [tasks[idx], tasks[i]];
      return;
    }
  }
}

/** 暂停队列：当前运行中的任务继续，完成后不再自动续接 */
function pauseQueue() {
  isQueuePaused.value = true;
}

/** 恢复队列：继续处理待转码任务 */
function resumeQueue() {
  isQueuePaused.value = false;
  if (!activeTask.value) startNextPending();
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

      // 推送系统通知
      const { state } = useApp();
      if (state.notificationsEnabled) {
        if (e.status === "completed") {
          notifyTaskCompleted(task.file.name);
        } else if (e.status === "failed") {
          notifyTaskFailed(task.file.name, task.error);
        }
      }

      // 续接下一个 + 检查全部完成
      startNextPending();
      checkAllDoneAndNotify();
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
    isQueuePaused,
    isAllDone,
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
    retryTask,
    moveTaskUp,
    moveTaskDown,
    pauseQueue,
    resumeQueue,
    startNextPending,
  };
}
