import { defineStore } from "pinia";
import { ref } from "vue";
import {
  getSettings,
  saveSettings as apiSave,
  type Settings,
} from "../api";

export const useSettingsStore = defineStore("settings", () => {
  const settings = ref<Settings>({
    source: "hf-mirror",
    threads: 5,
    hfToken: "",
    modelscopeToken: "",
    token: "",
    proxy: "",
    outputDir: "",
    targetMode: "none",
    comfyuiRoot: "",
    lmstudioRoot: "",
  });
  const loaded = ref(false);
  const saving = ref(false);

  async function load() {
    settings.value = await getSettings();
    loaded.value = true;
  }

  async function save() {
    saving.value = true;
    try {
      await apiSave(settings.value);
    } finally {
      saving.value = false;
    }
  }

  return { settings, loaded, saving, load, save };
});
