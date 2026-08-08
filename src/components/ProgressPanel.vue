<script setup lang="ts">
// Step 4: 队列暂停/恢复 + 全部完成状态
import { useTasks } from "@/stores/tasks";
import TaskCard from "@/components/TaskCard.vue";

const {
  list,
  hasTasks,
  completedCount,
  failedCount,
  overallProgress,
  isProcessing,
  isQueuePaused,
  isAllDone,
  clearFinished,
  pauseQueue,
  resumeQueue,
} = useTasks();
</script>

<template>
  <section class="progress-panel">
    <div v-if="hasTasks" class="panel-header">
      <div class="header-left">
        <span class="header-title">转码列表</span>
        <span class="header-summary">
          共 {{ list.length }} 个 · 已完成 {{ completedCount }}
          <span v-if="failedCount > 0" class="failed-count">· 失败 {{ failedCount }}</span>
          <span v-if="isQueuePaused" class="paused-tag">· 已暂停</span>
          <span v-else-if="isAllDone" class="done-tag">· 全部完成</span>
        </span>
      </div>
      <div class="header-right">
        <div v-if="isProcessing" class="overall">
          <span class="overall-pct">{{ overallProgress }}%</span>
          <div class="overall-bar">
            <div class="overall-fill" :style="{ width: overallProgress + '%' }" />
          </div>
        </div>
        <!-- 暂停/恢复队列 -->
        <button
          v-if="isProcessing || isQueuePaused"
          class="btn btn-ghost header-pause"
          @click="isQueuePaused ? resumeQueue() : pauseQueue()"
        >
          {{ isQueuePaused ? "恢复队列" : "暂停队列" }}
        </button>
        <button class="btn btn-ghost header-clear" @click="clearFinished">清理已完成</button>
      </div>
    </div>

    <TransitionGroup v-if="hasTasks" name="list" tag="div" class="task-list">
      <TaskCard v-for="task in list" :key="task.id" :task="task" />
    </TransitionGroup>

    <!-- 空状态 -->
    <div v-else class="empty-state">
      <svg viewBox="0 0 64 64" width="48" height="48" fill="none">
        <circle cx="32" cy="32" r="28" stroke="var(--border-subtle)" stroke-width="2" stroke-dasharray="4 5" />
        <path d="M24 32h16M32 24v16" stroke="var(--border-strong)" stroke-width="2" stroke-linecap="round" />
      </svg>
      <p class="empty-title">还没有任务</p>
      <p class="empty-hint">拖拽或选择视频文件即可开始</p>
    </div>
  </section>
</template>

<style scoped>
.progress-panel {
  display: flex;
  flex-direction: column;
  min-height: 0;
  flex: 1;
}
.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
  padding: 0 2px;
}
.header-left {
  display: flex;
  align-items: baseline;
  gap: 10px;
}
.header-title {
  font-size: 13px;
  font-weight: 700;
  color: var(--text-primary);
}
.header-summary {
  font-size: 11px;
  color: var(--text-tertiary);
}
.failed-count {
  color: var(--status-error);
}
.paused-tag {
  color: var(--status-warning);
  font-weight: 600;
}
.done-tag {
  color: var(--status-success);
  font-weight: 600;
}
.header-right {
  display: flex;
  align-items: center;
  gap: 10px;
}
.overall {
  display: flex;
  align-items: center;
  gap: 8px;
}
.overall-pct {
  font-size: 12px;
  font-weight: 700;
  color: var(--accent);
  min-width: 34px;
  text-align: right;
}
.overall-bar {
  width: 90px;
  height: 5px;
  background: var(--track);
  border-radius: 3px;
  overflow: hidden;
}
.overall-fill {
  height: 100%;
  background: var(--accent);
  border-radius: 3px;
  transition: width 0.4s var(--ease-out);
}
.header-pause {
  padding: 5px 10px;
  font-size: 11px;
  color: var(--status-warning);
}
.header-clear {
  padding: 5px 10px;
  font-size: 11px;
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  overflow-y: auto;
  padding-right: 4px;
  flex: 1;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 48px 20px;
  color: var(--text-tertiary);
}
.empty-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-top: 8px;
}
.empty-hint {
  font-size: 12px;
  color: var(--text-tertiary);
}
</style>
