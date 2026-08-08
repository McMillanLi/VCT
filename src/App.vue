<script setup lang="ts">
import { onMounted } from "vue";
import TitleBar from "@/components/TitleBar.vue";
import DropZone from "@/components/DropZone.vue";
import ConfigPanel from "@/components/ConfigPanel.vue";
import ProgressPanel from "@/components/ProgressPanel.vue";
import { useApp } from "@/stores/app";
import { useTasks, subscribeTaskEvents } from "@/stores/tasks";
import { isTauri } from "@/api/backend";

const { state, init } = useApp();
const { hasTasks, isProcessing, pendingTasks, startAll, clearAll } = useTasks();

onMounted(async () => {
  await init();
  await subscribeTaskEvents();
});

function onStart() {
  if (!hasTasks.value || isProcessing.value) return;
  startAll();
}
</script>

<template>
  <TitleBar />

  <main class="app-body">
    <div class="body-inner">
      <DropZone />

      <ConfigPanel />

      <ProgressPanel />
    </div>
  </main>

  <footer class="app-footer">
    <div class="footer-info">
      <span v-if="!isTauri" class="env-tag" title="当前为 Web 预览模式，转码为模拟数据">WEB 预览</span>
      <span v-if="pendingTasks.length > 0" class="footer-pending">
        {{ pendingTasks.length }} 个任务待转码
      </span>
    </div>
    <div class="footer-actions">
      <button class="btn btn-ghost" :disabled="!hasTasks" @click="clearAll">清空</button>
      <button class="btn btn-primary start-btn" :disabled="!hasTasks || isProcessing" @click="onStart">
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
          <path d="M3 2l9 5-9 5V2Z" fill="currentColor" />
        </svg>
        {{ isProcessing ? "转码进行中…" : "开始转码" }}
      </button>
    </div>
  </footer>
</template>

<style scoped>
.app-body {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}
.body-inner {
  display: flex;
  flex-direction: column;
  gap: 14px;
  max-width: 840px;
  margin: 0 auto;
}

.app-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 18px;
  background: var(--bg-surface);
  border-top: 1px solid var(--border-subtle);
  flex-shrink: 0;
}
.footer-info {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
  color: var(--text-tertiary);
}
.env-tag {
  padding: 2px 8px;
  border-radius: 4px;
  background: rgba(251, 191, 36, 0.12);
  color: var(--status-warning);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.3px;
}
.footer-pending {
  color: var(--accent);
  font-weight: 600;
}
.footer-actions {
  display: flex;
  gap: 10px;
}
.start-btn {
  min-width: 130px;
}
</style>
