<script setup lang="ts">
import { RouterLink, useRoute } from "vue-router";
import { computed } from "vue";

const route = useRoute();

const navs = [
  { path: "/download", label: "下载", icon: "M12 3v12m0 0l-4-4m4 4l4-4M5 21h14" },
  { path: "/browse", label: "模型浏览", icon: "M4 6h16M4 12h16M4 18h10" },
  { path: "/settings", label: "设置", icon: "M12 15a3 3 0 100-6 3 3 0 000 6z" },
];

const activePath = computed(() => route.path);
</script>

<template>
  <aside class="sidebar">
    <div class="brand">
      <div class="brand-logo">
        <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 3v12m0 0l-4-4m4 4l4-4M5 21h14" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
      </div>
      <div class="brand-text">
        <div class="brand-title">Mirror Download</div>
        <div class="brand-sub">桌面版</div>
      </div>
    </div>

    <nav class="nav">
      <RouterLink
        v-for="n in navs"
        :key="n.path"
        :to="n.path"
        class="nav-item"
        :class="{ active: activePath === n.path }"
      >
        <svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2">
          <path :d="n.icon" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <span>{{ n.label }}</span>
      </RouterLink>
    </nav>

    <div class="sidebar-footer">
      <div class="version">v0.1.0</div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 216px;
  flex-shrink: 0;
  background: var(--bg-elevated);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 16px 12px;
}
.brand {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 6px 8px 20px;
}
.brand-logo {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: linear-gradient(135deg, var(--accent), #7c5cff);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  box-shadow: 0 4px 14px rgba(79, 140, 255, 0.35);
}
.brand-title {
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.2px;
}
.brand-sub {
  font-size: 11px;
  color: var(--text-muted);
}
.nav {
  display: flex;
  flex-direction: column;
  gap: 3px;
  flex: 1;
}
.nav-item {
  display: flex;
  align-items: center;
  gap: 11px;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  color: var(--text-secondary);
  text-decoration: none;
  font-size: 13.5px;
  font-weight: 500;
  transition: background 0.15s, color 0.15s;
}
.nav-item:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}
.nav-item.active {
  background: var(--accent-soft);
  color: var(--accent);
}
.sidebar-footer {
  padding: 12px 12px 4px;
  border-top: 1px solid var(--border);
}
.version {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--mono);
}
</style>
