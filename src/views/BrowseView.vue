<script setup lang="ts">
import { computed, onMounted, reactive, ref } from "vue";
import { useRouter } from "vue-router";
import { findRepo, type RepoInfo, type RepoFile, type SourceKind, type DownloadRequest } from "../api";
import { useSettingsStore } from "../stores/settings";
import { useDownloadsStore } from "../stores/downloads";
import RepoSearchBar from "../components/browse/RepoSearchBar.vue";
import QuantFilter, { type QuantChip } from "../components/browse/QuantFilter.vue";
import FileList from "../components/browse/FileList.vue";
import { useToast } from "../components/ui/useToast";

const toast = useToast();

const settingsStore = useSettingsStore();
const downloadsStore = useDownloadsStore();
const router = useRouter();

const form = reactive<{
  repoId: string;
  repoType: string;
  source: SourceKind | "";
  revision: string;
}>({
  repoId: "Qwen/Qwen2.5-7B-Instruct",
  repoType: "model",
  source: "",
  // 留空表示使用各源仓库自身的默认分支（HF 通常为 main，ModelScope 通常为 master），
  // 避免硬编码错误分支导致 404。
  revision: "",
});

const loading = ref(false);
const error = ref("");
const repo = ref<RepoInfo | null>(null);
const selected = ref<string[]>([]);
const keyword = ref("");
const selectedQuants = ref<string[]>([]);

const sourceLabel = computed(() => {
  const s = form.source || settingsStore.settings.source;
  const map: Record<string, string> = {
    hf: "Hugging Face",
    "hf-mirror": "HF Mirror",
    modelscope: "ModelScope",
  };
  return map[s] ?? s;
});

// 量化类型识别（从路径中匹配）
const QUANT_PATTERNS: { key: string; label: string; re: RegExp }[] = [
  { key: "q4_k_m", label: "Q4_K_M", re: /q4[_-]?k[_-]?m/i },
  { key: "q5_k_m", label: "Q5_K_M", re: /q5[_-]?k[_-]?m/i },
  { key: "q6_k", label: "Q6_K", re: /q6[_-]?k/i },
  { key: "q8_0", label: "Q8_0", re: /q8[_-]?0/i },
  { key: "q4_0", label: "Q4_0", re: /q4[_-]?0/i },
  { key: "fp16", label: "FP16", re: /fp16/i },
  { key: "bf16", label: "BF16", re: /bf16/i },
  { key: "awq", label: "AWQ", re: /awq/i },
  { key: "gptq", label: "GPTQ", re: /gptq/i },
  { key: "gguf", label: "GGUF", re: /\.gguf$/i },
];

function quantOf(path: string): string | null {
  for (const p of QUANT_PATTERNS) {
    if (p.re.test(path)) return p.key;
  }
  return null;
}

const quantChips = computed<QuantChip[]>(() => {
  if (!repo.value) return [];
  const counts = new Map<string, number>();
  for (const f of repo.value.files) {
    const q = quantOf(f.path);
    if (q) counts.set(q, (counts.get(q) ?? 0) + 1);
  }
  return QUANT_PATTERNS.filter((p) => (counts.get(p.key) ?? 0) > 0).map((p) => ({
    key: p.key,
    label: p.label,
    count: counts.get(p.key) ?? 0,
  }));
});

// 量化筛选后的文件（无筛选时返回全部）
const quantFilteredFiles = computed<RepoFile[]>(() => {
  if (!repo.value) return [];
  if (!selectedQuants.value.length) return repo.value.files;
  return repo.value.files.filter((f) => {
    const q = quantOf(f.path);
    return q !== null && selectedQuants.value.includes(q);
  });
});

const selectedCount = computed(() => selected.value.length);
const selectedTotalSize = computed(() => {
  if (!repo.value) return 0;
  const set = new Set(selected.value);
  return repo.value.files
    .filter((f) => set.has(f.path) && typeof f.size === "number")
    .reduce((acc, f) => acc + (f.size ?? 0), 0);
});

function formatBytes(size: number): string {
  if (!size) return "";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let n = size;
  let i = 0;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i++;
  }
  return `${n.toFixed(n >= 100 || i === 0 ? 0 : 1)} ${units[i]}`;
}

const sourceValue = computed<SourceKind | "">({
  get: () => form.source,
  set: (v) => (form.source = v),
});

async function doQuery() {
  const id = form.repoId.trim();
  if (!id) return;
  loading.value = true;
  error.value = "";
  try {
    const s = settingsStore.settings;
    const info = await findRepo({
      repoId: id,
      repoType: form.repoType || undefined,
      revision: form.revision.trim() || undefined,
      source: form.source || undefined,
      hfToken: s.hfToken || undefined,
      modelscopeToken: s.modelscopeToken || undefined,
      token: s.token || undefined,
      proxy: s.proxy || undefined,
    });
    repo.value = info;
    selected.value = [];
    selectedQuants.value = [];
    keyword.value = "";
  } catch (e) {
    repo.value = null;
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    loading.value = false;
  }
}

function toggleFile(path: string) {
  const idx = selected.value.indexOf(path);
  if (idx >= 0) selected.value.splice(idx, 1);
  else selected.value.push(path);
}

function toggleAll(checked: boolean) {
  const visible = quantFilteredFiles.value;
  if (checked) {
    const set = new Set(selected.value);
    for (const f of visible) set.add(f.path);
    selected.value = Array.from(set);
  } else {
    const remove = new Set(visible.map((f) => f.path));
    selected.value = selected.value.filter((p) => !remove.has(p));
  }
}

function toggleQuant(key: string) {
  const idx = selectedQuants.value.indexOf(key);
  if (idx >= 0) selectedQuants.value.splice(idx, 1);
  else selectedQuants.value.push(key);
  // 切换筛选后清理不可见的已选文件
  const visible = new Set(quantFilteredFiles.value.map((f) => f.path));
  selected.value = selected.value.filter((p) => visible.has(p));
}

function clearQuants() {
  selectedQuants.value = [];
}

function buildRequest(): DownloadRequest | null {
  if (!repo.value || !selected.value.length) return null;
  const s = settingsStore.settings;
  const repoId = repo.value.id;
  const files = [...selected.value];
  return {
    taskId: `dl-${Date.now()}`,
    repoId,
    repoType: repo.value.repoType || form.repoType || undefined,
    revision: repo.value.revision || form.revision.trim() || undefined,
    source: form.source || s.source,
    threads: s.threads,
    output: s.outputDir || undefined,
    files,
    include: files,
    exclude: [],
    quant: selectedQuants.value.length === 1 ? selectedQuants.value[0] : undefined,
    targetMode: s.targetMode,
    comfyuiRoot: s.comfyuiRoot,
    lmstudioRoot: s.lmstudioRoot,
    hfToken: s.hfToken,
    modelscopeToken: s.modelscopeToken,
    token: s.token,
    proxy: s.proxy,
  };
}

async function downloadSelected() {
  const req = buildRequest();
  if (!req) {
    toast.error("请先勾选需要下载的文件");
    return;
  }
  try {
    await downloadsStore.init();
    await downloadsStore.enqueue(req);
    toast.success(`已加入下载队列（${req.files.length} 个文件），正在跳转…`);
    setTimeout(() => router.push("/download"), 700);
  } catch (e) {
    toast.error(`加入下载失败：${e instanceof Error ? e.message : String(e)}`);
  }
}

onMounted(async () => {
  if (!settingsStore.loaded) {
    try {
      await settingsStore.load();
    } catch {
      /* 使用默认设置 */
    }
  }
});
</script>

<template>
  <div class="browse-view">
    <header class="view-header">
      <div>
        <h1 class="view-title">模型浏览</h1>
        <p class="view-sub">搜索仓库、筛选量化文件，勾选后加入下载队列。</p>
      </div>
    </header>

    <div class="scroll-area">
      <RepoSearchBar
        v-model:repo-id="form.repoId"
        v-model:repo-type="form.repoType"
        v-model:source="sourceValue"
        v-model:revision="form.revision"
        :loading="loading"
        :source-label="sourceLabel"
        @submit="doQuery"
      />

      <div v-if="error" class="error-box">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="9" /><path d="M12 8v5M12 16h.01" stroke-linecap="round" />
        </svg>
        <span>{{ error }}</span>
      </div>

      <template v-if="repo">
        <div class="repo-head card">
          <div class="repo-main">
            <div class="repo-title">
              <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="1.8">
                <path d="M4 7a2 2 0 012-2h4l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H6a2 2 0 01-2-2V7z" stroke-linejoin="round" />
              </svg>
              <span>{{ repo.id }}</span>
            </div>
            <div class="repo-meta">
              <span class="tag tag-best">{{ repo.source }}</span>
              <span class="tag">{{ repo.repoType }}</span>
              <span class="tag">revision: {{ repo.revision }}</span>
              <span class="tag">{{ repo.files.length }} 个文件</span>
            </div>
          </div>
          <div class="download-action">
            <div class="sel-info">
              <template v-if="selectedCount">
                已选 <b>{{ selectedCount }}</b> 个
                <template v-if="selectedTotalSize"> · {{ formatBytes(selectedTotalSize) }}</template>
              </template>
              <template v-else>尚未选择文件</template>
            </div>
            <button class="btn btn-primary" :disabled="!selectedCount" @click="downloadSelected">
              <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 3v12m0 0l-4-4m4 4l4-4M5 21h14" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
              下载选中文件
            </button>
          </div>
        </div>

        <QuantFilter
          :chips="quantChips"
          :selected="selectedQuants"
          @toggle="toggleQuant"
          @clear="clearQuants"
        />

        <FileList
          v-model:keyword="keyword"
          :files="quantFilteredFiles"
          :selected="selected"
          @toggle="toggleFile"
          @toggle-all="toggleAll"
        />
      </template>

      <div v-else-if="!loading && !error" class="browse-hint card">
        <svg viewBox="0 0 24 24" width="40" height="40" fill="none" stroke="currentColor" stroke-width="1.3">
          <path d="M11 4a7 7 0 100 14 7 7 0 000-14zM21 21l-4.3-4.3" stroke-linecap="round" />
        </svg>
        <p>输入仓库 ID 后点击「查询」，即可浏览仓库文件与量化版本。</p>
        <p class="hint-example">示例：<code>Qwen/Qwen2.5-7B-Instruct</code> · <code>TheBloke/Mistral-7B-Instruct-v0.2-GGUF</code></p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.browse-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 24px 28px;
  gap: 16px;
  overflow: hidden;
  min-height: 0;
  position: relative;
}
.view-header {
  flex-shrink: 0;
}
.view-title {
  font-size: 20px;
  font-weight: 650;
  letter-spacing: 0.2px;
}
.view-sub {
  font-size: 12.5px;
  color: var(--text-muted);
  margin-top: 3px;
}
.scroll-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
  min-height: 0;
  padding-right: 4px;
  padding-bottom: 6px;
}
.error-box {
  display: flex;
  align-items: center;
  gap: 9px;
  padding: 12px 16px;
  border-radius: var(--radius-md);
  background: rgba(242, 109, 109, 0.1);
  border: 1px solid rgba(242, 109, 109, 0.35);
  color: var(--danger);
  font-size: 13px;
  flex-shrink: 0;
}
.repo-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
  flex-shrink: 0;
}
.repo-main {
  display: flex;
  flex-direction: column;
  gap: 9px;
  min-width: 0;
  flex: 1;
}
.repo-title {
  display: flex;
  align-items: center;
  gap: 9px;
  font-size: 15.5px;
  font-weight: 650;
  color: var(--text-primary);
  word-break: break-all;
}
.repo-title svg {
  color: var(--accent);
  flex-shrink: 0;
}
.repo-meta {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.download-action {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  gap: 8px;
  flex-shrink: 0;
}
.sel-info {
  font-size: 12px;
  color: var(--text-muted);
}
.sel-info b {
  color: var(--accent);
}
.browse-hint {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  text-align: center;
  color: var(--text-muted);
  padding: 44px 24px;
  font-size: 13px;
}
.browse-hint svg {
  color: var(--text-secondary);
  opacity: 0.7;
}
.hint-example code {
  font-family: var(--mono);
  font-size: 12px;
  color: var(--accent);
  background: var(--accent-soft);
  padding: 1px 6px;
  border-radius: 4px;
}
</style>
