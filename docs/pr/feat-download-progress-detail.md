# ✨ feat(下载界面): 增强下载状态的可视化展示

## 背景
下载队列页此前每个任务只有一条 7px 进度条加百分比数字，无法直观看出当前下载到哪个文件、
还剩多久、速度变化趋势，也无法看到各文件的分项进度，状态可读性较低。

## 改动
- `src/api/index.ts`：`ProgressEvent` 扩展可选字段 `downloadedBytes / totalBytes /
  fileBytesDone / fileBytesTotal / speedBps / files`，新增 `FileProgressItem` 类型；
  浏览器 mock 生成逐文件大小、字节进度与速度，支持完整预览。
- `src/stores/downloads.ts`：任务新增 `filesDetail / speedHistory / etaSeconds / downloadedBytes /
  totalBytes`；新增 `overallProgress / overallSpeedBps / overallBytes` 汇总度量；维护速度历史环
  形缓冲与基于近期均速的 ETA 估算；导出 `formatBytes / formatEta` 工具。
- `src/components/download/QueueStats.vue`：新增队列总体进度条（运行中流光动画）、汇总速度、
  已下/总字节，活跃任务脉冲圆点。
- `src/components/download/DownloadTaskCard.vue`：新增状态图标（旋转/勾/叉/横杠/时钟）、
  文件序号 `3 / 15 个文件`、已下/总大小、剩余时间 ETA、SVG 速度迷你趋势图、"详情"展开按钮。
- `src/components/download/TaskFileDetails.vue`：新增组件，可展开的逐文件分项进度列表。
- `src/views/DownloadView.vue`：接线汇总数据；非 Tauri 环境支持 `?demo=1` 注入 mock 任务预览。

## 自测
- 类型检查：`npm run build`（`vue-tsc --noEmit && vite build`）→ 0 错误，构建成功。
- 浏览器（mock）视觉验证：多任务队列下总体进度条、汇总速度、逐任务速度趋势图、ETA、
  文件序号均正常；展开"详情"后逐文件状态图标/进度条/字节数/百分比显示正确。

## 影响
仅影响下载队列页面的展示层与进度事件类型；后端未提供新字段时自动回退到原有字符串度量与
文件序号推断，保持向后兼容。

## 关联
下载界面可视化改版：更直观展现下载状态（总体进度、实时速度、剩余时间、文件级明细）。
