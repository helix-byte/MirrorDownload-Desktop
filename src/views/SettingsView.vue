<script setup lang="ts">
// 设置页：完整实现下载源、并发与输出、认证、代理、目标适配等设置。
import { computed, onMounted, reactive, ref } from "vue";
import {
  getConfigDir,
  probeSources,
  isTauri,
  type ProbeReport,
  type SourceKind,
  type Settings,
} from "../api";
import { useSettingsStore } from "../stores/settings";
import AppButton from "../components/ui/AppButton.vue";
import { useToast } from "../components/ui/useToast";
import FieldRow from "../components/ui/FieldRow.vue";
import SettingsSection from "../components/settings/SettingsSection.vue";
import SourceSelector from "../components/settings/SourceSelector.vue";
import TokenInput from "../components/settings/TokenInput.vue";
import TargetDirPicker from "../components/settings/TargetDirPicker.vue";

const store = useSettingsStore();
const toast = useToast();

// 本地可编辑副本，避免直接修改 store（保存时才写回）
const form = reactive<Settings>({ ...store.settings });

const configDir = ref("");
const probing = ref(false);
const probeReport = ref<ProbeReport | null>(null);
const saving = ref(false);

const targetOptions = [
  { value: "none" as const, label: "不启用", desc: "仅下载到输出目录，不做额外目录适配" },
  { value: "comfyui" as const, label: "ComfyUI", desc: "按 ComfyUI 的模型目录结构放置文件" },
  { value: "lmstudio" as const, label: "LM Studio", desc: "按 LM Studio 模型库结构放置文件" },
];

const threadsInvalid = computed(
  () => !Number.isInteger(form.threads) || form.threads < 1 || form.threads > 64,
);

onMounted(async () => {
  try {
    if (!store.loaded) await store.load();
    Object.assign(form, store.settings);
    configDir.value = await getConfigDir();
  } catch (err) {
    toast.error("加载设置失败");
    console.error(err);
  }
});

function clampThreads() {
  if (Number.isNaN(form.threads)) form.threads = 1;
  form.threads = Math.min(64, Math.max(1, Math.round(form.threads)));
}

async function onProbe() {
  probing.value = true;
  try {
    probeReport.value = await probeSources({
      source: form.source,
      proxy: form.proxy || undefined,
    });
    toast.success("探测完成");
  } catch (err) {
    toast.error("探测失败，请检查网络或代理");
    console.error(err);
  } finally {
    probing.value = false;
  }
}

async function onSave() {
  if (threadsInvalid.value) {
    toast.error("并发线程数需在 1 - 64 之间");
    return;
  }
  saving.value = true;
  try {
    Object.assign(store.settings, form);
    await store.save();
    toast.success("设置已保存");
  } catch (err) {
    toast.error("保存失败");
    console.error(err);
  } finally {
    saving.value = false;
  }
}

function onReset() {
  Object.assign(form, store.settings);
  probeReport.value = null;
  toast.info("已还原为上次保存的设置");
}

function setSource(v: SourceKind) {
  form.source = v;
}
</script>

<template>
  <div class="settings-view">
    <header class="page-head">
      <div>
        <h1 class="page-title">设置</h1>
        <p class="page-sub">配置下载源、并发、认证与目标适配，设置将保存到本地配置文件。</p>
      </div>
      <div class="head-actions">
        <AppButton variant="ghost" :disabled="saving" @click="onReset">还原</AppButton>
        <AppButton variant="primary" :loading="saving" @click="onSave">保存设置</AppButton>
      </div>
    </header>

    <div class="settings-scroll">
      <div class="settings-body">
        <!-- 1. 下载源 -->
        <SettingsSection
          title="下载源"
          description="选择模型文件的下载源，可先探测各源延迟再做决定。"
          icon="M12 3v12m0 0l-4-4m4 4l4-4"
        >
          <SourceSelector
            :model-value="form.source"
            :probe-report="probeReport"
            :probing="probing"
            @update:model-value="setSource"
            @probe="onProbe"
          />
        </SettingsSection>

        <!-- 2. 并发与输出 -->
        <SettingsSection
          title="并发与输出"
          description="控制下载并发线程数与默认输出目录。"
          icon="M4 6h16M4 12h16M4 18h10"
        >
          <FieldRow label="并发线程数" hint="同时下载的分片数量，范围 1 - 64；过大可能触发限速。">
            <div class="threads-control">
              <input
                v-model.number="form.threads"
                type="number"
                min="1"
                max="64"
                step="1"
                class="threads-input"
                :class="{ invalid: threadsInvalid }"
                @blur="clampThreads"
              />
              <span class="unit">线程</span>
            </div>
          </FieldRow>
          <FieldRow
            label="输出目录"
            hint="模型文件的默认下载位置；留空则由 CLI 使用当前目录。"
          >
            <TargetDirPicker v-model="form.outputDir" placeholder="例如 D:\\Models\\Downloads" />
          </FieldRow>
        </SettingsSection>

        <!-- 3. 认证 -->
        <SettingsSection
          title="认证"
          description="访问私有仓库或提升速率限制时可填写访问令牌，敏感信息仅保存在本地。"
          icon="M12 15a3 3 0 100-6 3 3 0 000 6z"
        >
          <FieldRow label="Hugging Face Token" hint="HF 个人访问令牌（hf_ 开头），用于私有模型或提额。">
            <TokenInput v-model="form.hfToken" placeholder="hf_xxx（可选）" />
          </FieldRow>
          <FieldRow label="ModelScope Token" hint="魔搭访问令牌，用于私有模型下载。">
            <TokenInput v-model="form.modelscopeToken" placeholder="可选" />
          </FieldRow>
          <FieldRow label="通用 Token" hint="通用访问令牌，按需填写。">
            <TokenInput v-model="form.token" placeholder="可选" />
          </FieldRow>
        </SettingsSection>

        <!-- 4. 代理 -->
        <SettingsSection
          title="代理"
          description="通过 HTTP / SOCKS 代理访问下载源，留空表示直连。"
          icon="M4 12a8 8 0 0116 0M4 12a8 8 0 0016 0M12 4v16"
        >
          <FieldRow label="代理地址" hint="例如 http://127.0.0.1:7890 或 socks5://127.0.0.1:1080。">
            <input v-model="form.proxy" type="text" placeholder="http://127.0.0.1:7890（可选）" spellcheck="false" />
          </FieldRow>
        </SettingsSection>

        <!-- 5. 目标适配 -->
        <SettingsSection
          title="目标适配"
          description="下载完成后按目标框架的目录结构放置模型文件。"
          icon="M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2V7z"
        >
          <div class="target-options" role="radiogroup" aria-label="目标适配">
            <button
              v-for="opt in targetOptions"
              :key="opt.value"
              type="button"
              class="target-card"
              :class="{ active: form.targetMode === opt.value }"
              role="radio"
              :aria-checked="form.targetMode === opt.value"
              @click="form.targetMode = opt.value"
            >
              <span class="radio-dot" :class="{ on: form.targetMode === opt.value }"></span>
              <span class="target-text">
                <span class="target-name">{{ opt.label }}</span>
                <span class="target-desc">{{ opt.desc }}</span>
              </span>
            </button>
          </div>

          <div v-if="form.targetMode === 'comfyui'" class="target-extra">
            <FieldRow label="ComfyUI 根目录" hint="ComfyUI 安装目录，下载的模型会按类别放入 models/ 子目录。">
              <TargetDirPicker v-model="form.comfyuiRoot" placeholder="例如 D:\\ComfyUI" />
            </FieldRow>
          </div>

          <div v-if="form.targetMode === 'lmstudio'" class="target-extra">
            <FieldRow label="LM Studio 模型目录" hint="留空则使用默认目录 ~/.lmstudio/models。">
              <TargetDirPicker
                v-model="form.lmstudioRoot"
                placeholder="留空使用默认 ~/.lmstudio/models"
              />
            </FieldRow>
          </div>
        </SettingsSection>

        <!-- 配置文件目录 -->
        <SettingsSection
          title="配置文件位置"
          description="以上设置将持久化到以下目录下的配置文件。"
          icon="M4 6h16M4 12h16M4 18h10"
        >
          <div class="config-dir">
            <code class="config-path">{{ configDir || "加载中…" }}</code>
            <span v-if="!isTauri" class="tag">浏览器预览模式</span>
          </div>
        </SettingsSection>

        <footer class="bottom-actions">
          <AppButton variant="ghost" :disabled="saving" @click="onReset">还原</AppButton>
          <AppButton variant="primary" :loading="saving" @click="onSave">保存设置</AppButton>
        </footer>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}
.page-head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 22px 28px 16px;
  border-bottom: 1px solid var(--border);
}
.page-title {
  font-size: 20px;
  font-weight: 600;
  letter-spacing: 0.2px;
}
.page-sub {
  margin-top: 4px;
  font-size: 12.5px;
  color: var(--text-muted);
}
.head-actions {
  display: flex;
  gap: 10px;
  flex-shrink: 0;
}
.settings-scroll {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}
.settings-body {
  max-width: 900px;
  margin: 0 auto;
  padding: 22px 28px 40px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.threads-control {
  display: flex;
  align-items: center;
  gap: 10px;
  max-width: 180px;
}
.threads-input {
  width: 100px;
  font-family: var(--mono);
}
.threads-input.invalid {
  border-color: var(--danger);
  box-shadow: 0 0 0 3px rgba(242, 109, 109, 0.14);
}
.unit {
  font-size: 12.5px;
  color: var(--text-muted);
}
.target-options {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
}
.target-card {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  text-align: left;
  padding: 13px 14px;
  border-radius: var(--radius-md);
  background: var(--bg-base);
  border: 1px solid var(--border);
  transition: border-color 0.15s, background 0.15s;
}
.target-card:hover {
  border-color: var(--border-strong);
  background: var(--bg-hover);
}
.target-card.active {
  border-color: var(--accent);
  background: var(--accent-soft);
}
.radio-dot {
  flex-shrink: 0;
  margin-top: 2px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid var(--border-strong);
  position: relative;
  transition: border-color 0.15s;
}
.radio-dot.on {
  border-color: var(--accent);
}
.radio-dot.on::after {
  content: "";
  position: absolute;
  inset: 2px;
  border-radius: 50%;
  background: var(--accent);
}
.target-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}
.target-name {
  font-size: 13.5px;
  font-weight: 600;
}
.target-desc {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.45;
}
.target-extra {
  margin-top: 14px;
  padding-top: 2px;
}
.config-dir {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.config-path {
  font-family: var(--mono);
  font-size: 12.5px;
  color: var(--text-secondary);
  background: var(--bg-base);
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  padding: 7px 11px;
  word-break: break-all;
}
.bottom-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding-top: 4px;
}
@media (max-width: 900px) {
  .target-options {
    grid-template-columns: 1fr;
  }
  .page-head {
    flex-direction: column;
  }
}
</style>
