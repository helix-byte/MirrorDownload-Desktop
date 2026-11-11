import { createRouter, createWebHashHistory } from "vue-router";

const routes = [
  { path: "/", redirect: "/download" },
  {
    path: "/download",
    name: "download",
    component: () => import("../views/DownloadView.vue"),
  },
  {
    path: "/browse",
    name: "browse",
    component: () => import("../views/BrowseView.vue"),
  },
  {
    path: "/settings",
    name: "settings",
    component: () => import("../views/SettingsView.vue"),
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
