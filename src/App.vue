<template>
  <div class="quark-app">
    <aside class="sidebar">
      <div class="sidebar-title">EchoConvert</div>

      <nav class="side-nav">
        <button
          type="button"
          class="side-item"
          :class="{ active: activeNav === 'home' }"
          @click="activeNav = 'home'"
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M3 11.5L12 4l9 7.5"></path>
            <path d="M6.5 10.5V20h11V10.5"></path>
          </svg>
          <span>主页</span>
        </button>

        <button
          type="button"
          class="side-item"
          :class="{ active: activeNav === 'transfer' }"
          @click="openTransfer('downloading')"
        >
          <svg viewBox="0 0 24 24" aria-hidden="true">
            <path d="M12 4v10"></path>
            <path d="M8 10.5L12 14.5l4-4"></path>
            <path d="M4 19h16"></path>
          </svg>
          <span>传输</span>
          <span v-if="runningTasks.length" class="side-badge">{{ runningTasks.length }}</span>
        </button>
      </nav>
    </aside>

    <main class="main-view">
      <section v-if="activeNav === 'home'" class="home-view">
        <div class="search-wrapper" :class="parsedOnce ? 'search-collapsed' : 'search-initial'">
          <div v-if="!parsedOnce" class="welcome">
            <h1>你好，我是歌单助手</h1>
            <p>粘贴链接，一键获取您的云端乐库</p>
          </div>

          <div class="search-box">
            <div class="search-input-row">
              <input
                v-model="inputUrl"
                type="text"
                placeholder="在此输入歌单网址..."
                @keydown.enter.prevent="parsePlaylist"
              />
            </div>

            <div class="search-action-row">
              <div class="auto-tag">自动识别</div>
              <div class="action-buttons">
                <button type="button" class="line-btn" @click="chooseDownloadDirectory">下载目录</button>
                <button type="button" class="primary-btn" :disabled="playlistLoading" @click="parsePlaylist">
                  {{ playlistLoading ? "正在解析..." : "开始解析" }}
                </button>
              </div>
            </div>
          </div>
        </div>

        <div v-if="songs.length" class="result-area">
          <div class="result-header">
            <div class="result-meta">
              <h2>解析结果</h2>
              <span>已选择 {{ selectedCount }} 项</span>
            </div>
            <button type="button" class="primary-btn" :disabled="!selectedCount" @click="downloadSelectedSongs">
              批量下载
            </button>
          </div>

          <div class="result-table-wrap custom-scroll">
            <table class="result-table">
              <thead>
                <tr>
                  <th>
                    <input
                      type="checkbox"
                      :checked="masterChecked"
                      :indeterminate.prop="!masterChecked && selectedCount > 0"
                      @change="toggleAllSongs($event.target.checked)"
                    />
                  </th>
                  <th>文件名</th>
                  <th>作者</th>
                  <th>类型</th>
                  <th>大小</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="song in songs" :key="song.id">
                  <td>
                    <input type="checkbox" v-model="song.selected" @change="syncMasterCheckbox" />
                  </td>
                  <td class="file-cell">
                    <img v-if="song.coverUrl" :src="song.coverUrl" :alt="song.title" class="song-thumb" />
                    <div v-else class="song-thumb placeholder">N</div>
                    <div>
                      <div class="song-title">{{ song.title }}</div>
                      <div class="song-sub">{{ song.album }} · {{ song.durationLabel }}</div>
                    </div>
                  </td>
                  <td>{{ song.artist }}</td>
                  <td>{{ song.fileType || "Unknown" }}</td>
                  <td>{{ song.sizeLabel }}</td>
                  <td>
                    <button type="button" class="icon-btn" @click="downloadSingleSong(song)">
                      <svg viewBox="0 0 24 24" aria-hidden="true">
                        <path d="M12 4v10"></path>
                        <path d="M8 10.5L12 14.5l4-4"></path>
                      </svg>
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>

      <section v-else class="transfer-view">
        <div class="transfer-header">
          <div class="transfer-title-wrap">
            <h2>传输</h2>
            <div class="tabs">
              <button
                type="button"
                class="tab-btn"
                :class="{ active: transferTab === 'downloading' }"
                @click="transferTab = 'downloading'"
              >
                正在下载 <span>{{ runningTasks.length }}</span>
              </button>
              <button
                type="button"
                class="tab-btn"
                :class="{ active: transferTab === 'completed' }"
                @click="transferTab = 'completed'"
              >
                已完成 <span>{{ completedTasks.length }}</span>
              </button>
            </div>
          </div>
          <button type="button" class="ghost-btn" @click="clearCompleted">清空全部记录</button>
        </div>

        <div class="transfer-list custom-scroll">
          <div v-if="transferTab === 'downloading'">
            <article v-for="task in runningTasks" :key="task.id" class="task-card">
              <div class="task-row">
                <div>
                  <strong>{{ task.title }}</strong>
                  <p>{{ task.artist }} · {{ task.type }}</p>
                </div>
                <div class="task-progress-side">
                  <span>{{ task.progress }}%</span>
                  <div class="mini-track">
                    <div class="mini-value" :style="{ width: `${task.progress}%` }"></div>
                  </div>
                </div>
              </div>
            </article>
            <div v-if="!runningTasks.length" class="empty">暂无进行中的任务</div>
          </div>

          <div v-else>
            <article
              v-for="task in completedTasks"
              :key="task.id"
              class="history-row"
              :class="{ 'open-ok': task.folderFeedback === 'ok', 'open-error': task.folderFeedback === 'error' }"
            >
              <div class="history-main">
                <strong>{{ task.title }}</strong>
                <p>{{ task.artist }} · {{ task.type }} · {{ task.sizeLabel }}</p>
                <details class="history-detail">
                  <summary>查看详情</summary>
                  <p class="history-time">{{ task.completedAt || task.createdAt }}</p>
                  <p v-if="task.savedPath" class="history-path">{{ task.savedPath }}</p>
                  <p v-if="task.error" class="history-error">{{ task.error }}</p>
                </details>
              </div>
              <div class="history-actions">
                <button
                  v-if="task.savedPath"
                  type="button"
                  class="folder-btn"
                  data-tip="打开保存文件夹"
                  @click="openSavedFolder(task)"
                >
                  <svg viewBox="0 0 24 24" aria-hidden="true">
                    <path d="M3 7h6l2 2h10v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
                    <path d="M3 9V7a2 2 0 0 1 2-2h4"></path>
                  </svg>
                </button>
                <span v-if="task.folderFeedback === 'ok'" class="folder-feedback ok">已打开</span>
                <span v-if="task.folderFeedback === 'error'" class="folder-feedback error">打开失败</span>
                <span class="history-status" :class="task.status">{{ task.status }}</span>
              </div>
            </article>
            <div v-if="!completedTasks.length" class="empty">暂无历史记录</div>
          </div>
        </div>
      </section>
    </main>

    <div class="toast" :class="{ show: !!toastText }">{{ toastText }}</div>
  </div>
</template>

<script setup>
import { computed, onMounted, ref } from "vue"
import { invoke } from "@tauri-apps/api/core"

const activeNav = ref("home")
const transferTab = ref("downloading")
const inputUrl = ref("")
const parsedOnce = ref(false)
const playlistLoading = ref(false)
const songs = ref([])
const masterChecked = ref(false)

const downloadDir = ref("")
const tasks = ref([])
const downloadQueue = ref([])
const queueWorkerRunning = ref(false)

const toastText = ref("")
let toastTimer = null

const selectedCount = computed(() => songs.value.filter((song) => song.selected).length)
const runningTasks = computed(() =>
  tasks.value.filter((task) => task.status === "queued" || task.status === "running"),
)
const completedTasks = computed(() =>
  tasks.value
    .filter((task) => task.status === "done" || task.status === "error")
    .sort((a, b) => new Date(b.completedAt || b.createdAt) - new Date(a.completedAt || a.createdAt)),
)

function nowLabel() {
  return new Date().toLocaleString("zh-CN", {
    hour12: false,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  })
}

function showToast(text) {
  toastText.value = text
  if (toastTimer) clearTimeout(toastTimer)
  toastTimer = setTimeout(() => {
    toastText.value = ""
  }, 2400)
}

function openTransfer(tab = "downloading") {
  activeNav.value = "transfer"
  transferTab.value = tab
}

function createTask(payload) {
  const task = {
    id: `${Date.now()}-${Math.random()}`,
    status: "queued",
    progress: 0,
    createdAt: nowLabel(),
    completedAt: "",
    savedPath: "",
    error: "",
    ...payload,
  }
  tasks.value.unshift(task)
  return task
}

function updateTask(id, patch) {
  const target = tasks.value.find((item) => item.id === id)
  if (!target) return
  Object.assign(target, patch)
}

async function loadSettings() {
  try {
    const settings = await invoke("get_app_settings")
    downloadDir.value = settings.downloadDir || ""
  } catch (error) {
    showToast(`读取设置失败: ${String(error)}`)
  }
}

async function chooseDownloadDirectory() {
  const next = window.prompt("请输入在线下载保存路径", downloadDir.value || "")
  if (!next) return
  try {
    const settings = await invoke("set_download_directory", { path: next })
    downloadDir.value = settings.downloadDir || next
    showToast("存储路径已更新")
  } catch (error) {
    showToast(`设置失败: ${String(error)}`)
  }
}

async function parsePlaylist() {
  if (!inputUrl.value.trim()) {
    showToast("请输入歌单链接")
    return
  }
  playlistLoading.value = true
  try {
    const result = await invoke("analyze_online_playlist", { query: inputUrl.value.trim() })
    songs.value = (result.tracks || []).map((song) => ({ ...song, selected: false }))
    masterChecked.value = false
    parsedOnce.value = true
    showToast(`解析完成，共 ${songs.value.length} 首`)
  } catch (error) {
    showToast(`解析失败: ${String(error)}`)
  } finally {
    playlistLoading.value = false
  }
}

function syncMasterCheckbox() {
  const total = songs.value.length
  const selected = selectedCount.value
  masterChecked.value = total > 0 && selected === total
}

function toggleAllSongs(checked) {
  songs.value = songs.value.map((song) => ({ ...song, selected: checked }))
  masterChecked.value = checked
}

function enqueueDownloadTask(song) {
  const task = createTask({
    kind: "online_download",
    title: song.title,
    artist: song.artist || "未知歌手",
    type: song.fileType || "Unknown",
    sizeLabel: song.sizeLabel || "未知大小",
    sourceSongId: song.id,
  })
  downloadQueue.value.push({
    taskId: task.id,
    track: { id: song.id, title: song.title, artist: song.artist || "未知歌手" },
  })
}

function downloadSingleSong(song) {
  enqueueDownloadTask(song)
  processDownloadQueue()
  showToast(`已加入下载队列: ${song.title}`)
  openTransfer("downloading")
}

function downloadSelectedSongs() {
  const selected = songs.value.filter((song) => song.selected)
  if (!selected.length) return
  for (const song of selected) enqueueDownloadTask(song)
  songs.value = songs.value.map((song) => ({ ...song, selected: false }))
  masterChecked.value = false
  processDownloadQueue()
  showToast(`已加入 ${selected.length} 首到下载队列`)
  openTransfer("downloading")
}

async function processDownloadQueue() {
  if (queueWorkerRunning.value) return
  queueWorkerRunning.value = true
  try {
    while (downloadQueue.value.length) {
      const item = downloadQueue.value.shift()
      if (!item) continue
      updateTask(item.taskId, { status: "running", progress: 20 })
      const result = await invoke("download_online_track", { track: item.track })
      if (result.status === "done") {
        updateTask(item.taskId, {
          status: "done",
          progress: 100,
          completedAt: nowLabel(),
          savedPath: result.savedPath || "",
        })
      } else {
        updateTask(item.taskId, {
          status: "error",
          progress: 100,
          completedAt: nowLabel(),
          error: result.message || "下载失败",
        })
      }
    }
  } catch (error) {
    showToast(`下载队列中断: ${String(error)}`)
  } finally {
    queueWorkerRunning.value = false
  }
}

function clearCompleted() {
  tasks.value = tasks.value.filter((task) => task.status === "queued" || task.status === "running")
  showToast("已清空已完成记录")
}

async function openSavedFolder(taskRef) {
  const target = tasks.value.find((task) => task.id === taskRef.id)
  if (!target) return
  try {
    await invoke("open_saved_folder", { savedPath: taskRef.savedPath })
    target.folderFeedback = "ok"
  } catch (error) {
    target.folderFeedback = "error"
    showToast(`打开失败: ${String(error)}`)
  } finally {
    window.setTimeout(() => {
      target.folderFeedback = ""
    }, 1600)
  }
}

onMounted(() => {
  loadSettings()
})
</script>

<style scoped>
:global(*) { box-sizing: border-box; }
:global(body) {
  margin: 0;
  font-family: "PingFang SC", "Microsoft YaHei", sans-serif;
  background: #fff;
  color: #1a1a1a;
}

.quark-app {
  --blue: #3267ff;
  display: flex;
  min-height: 100vh;
}

.sidebar {
  width: 86px;
  border-right: 1px solid #eef1f6;
  background: #fff;
  padding: 16px 10px;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.sidebar-title {
  font-size: 11px;
  color: #9ca4b6;
  margin-bottom: 18px;
}

.side-nav {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.side-item {
  width: 100%;
  border: 1px solid transparent;
  background: transparent;
  color: #7d8493;
  border-radius: 12px;
  padding: 10px 0 8px;
  display: grid;
  justify-items: center;
  gap: 4px;
  position: relative;
  cursor: pointer;
  transition: all 0.2s ease;
}

.side-item svg {
  width: 19px;
  height: 19px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.side-item.active {
  color: var(--blue);
  background: #edf2ff;
  border-color: #d9e4ff;
  box-shadow: 0 6px 16px rgba(50, 103, 255, 0.14);
}

.side-item span {
  font-size: 11px;
  font-weight: 600;
}

.side-badge {
  position: absolute;
  top: 6px;
  right: 8px;
  min-width: 16px;
  height: 16px;
  border-radius: 999px;
  background: var(--blue);
  color: #fff;
  font-size: 10px;
  line-height: 16px;
  text-align: center;
  padding: 0 3px;
}

.main-view {
  flex: 1;
  min-width: 0;
}

.home-view {
  min-height: 100vh;
  padding: 22px 30px;
}

.search-wrapper {
  width: 100%;
  max-width: 860px;
  margin: 0 auto;
  transition: all 0.45s cubic-bezier(0.16, 1, 0.3, 1);
}

.search-initial { transform: translateY(18vh); }
.search-collapsed { transform: translateY(0); margin-bottom: 18px; max-width: 100%; }

.welcome {
  text-align: center;
  margin-bottom: 26px;
}

.welcome h1 {
  margin: 0;
  font-size: clamp(24px, 5vw, 36px);
}

.welcome p {
  margin: 10px 0 0;
  color: #8a90a0;
}

.search-box {
  border: 1px solid #edf0f6;
  border-radius: 24px;
  box-shadow: 0 6px 24px rgba(20, 28, 45, 0.06);
  background: #fff;
  padding: 8px;
}

.search-input-row input {
  border: 0;
  outline: none;
  width: 100%;
  font-size: 17px;
  padding: 14px 14px 12px;
}

.search-action-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding: 4px 6px 2px;
}

.action-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.auto-tag {
  background: #eef3ff;
  color: var(--blue);
  border-radius: 12px;
  font-size: 13px;
  font-weight: 600;
  padding: 7px 11px;
}

.primary-btn,
.line-btn,
.ghost-btn {
  border-radius: 12px;
  font-weight: 700;
  cursor: pointer;
}

.primary-btn {
  border: 0;
  background: var(--blue);
  color: #fff;
  padding: 10px 14px;
}

.primary-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.line-btn {
  border: 1px solid #d9e4ff;
  background: #f8faff;
  color: var(--blue);
  padding: 9px 12px;
}

.result-area {
  margin-top: 14px;
}

.result-header {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
  margin-bottom: 12px;
}

.result-meta h2 {
  margin: 0;
  font-size: 20px;
}

.result-meta span {
  color: #8a90a0;
  font-size: 12px;
}

.result-table-wrap {
  border: 1px solid #edf0f6;
  border-radius: 16px;
  overflow: auto;
  max-height: min(60vh, 620px);
}

.result-table {
  width: 100%;
  border-collapse: collapse;
}

.result-table th,
.result-table td {
  padding: 14px 12px;
  border-bottom: 1px solid #f4f6fa;
  text-align: left;
  font-size: 14px;
}

.result-table th {
  position: sticky;
  top: 0;
  background: #fff;
  color: #8a90a0;
  font-size: 11px;
  text-transform: uppercase;
  z-index: 1;
}

.file-cell {
  display: flex;
  gap: 10px;
  align-items: center;
  min-width: 220px;
}

.song-thumb {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  object-fit: cover;
  flex: none;
}

.song-thumb.placeholder {
  background: #eef3ff;
  color: var(--blue);
  display: grid;
  place-items: center;
  font-size: 11px;
  font-weight: 700;
}

.song-title {
  font-weight: 600;
}

.song-sub {
  margin-top: 2px;
  color: #8a90a0;
  font-size: 12px;
}

.icon-btn {
  width: 30px;
  height: 30px;
  border: 1px solid #e5eaf5;
  background: #fff;
  color: #6782d8;
  border-radius: 9px;
  cursor: pointer;
  display: grid;
  place-items: center;
}

.icon-btn svg {
  width: 16px;
  height: 16px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.transfer-view {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.transfer-header {
  padding: 18px 30px 12px;
  border-bottom: 1px solid #edf0f6;
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
}

.transfer-title-wrap h2 {
  margin: 0;
}

.tabs {
  margin-top: 8px;
  display: flex;
  gap: 8px;
}

.tab-btn {
  border: 1px solid #e2e8f5;
  background: #fff;
  color: #7d8493;
  border-radius: 999px;
  padding: 7px 12px;
  cursor: pointer;
  font-weight: 600;
  transition: all 0.2s ease;
}

.tab-btn.active {
  color: var(--blue);
  border-color: #cddcff;
  background: #edf2ff;
  box-shadow: 0 4px 12px rgba(50, 103, 255, 0.15);
}

.ghost-btn {
  border: 0;
  background: transparent;
  color: #8a90a0;
  padding: 8px 10px;
}

.transfer-list {
  padding: 14px 30px 26px;
  overflow: auto;
  flex: 1;
}

.task-card {
  border: 1px solid #edf0f6;
  border-radius: 14px;
  background: #fff;
  padding: 14px;
  margin-bottom: 10px;
}

.task-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: center;
}

.task-row strong {
  display: block;
}

.task-row p {
  margin: 3px 0 0;
  color: #8a90a0;
  font-size: 12px;
}

.task-progress-side {
  min-width: 84px;
  display: grid;
  justify-items: end;
  gap: 4px;
}

.mini-track {
  width: 80px;
  height: 4px;
  border-radius: 999px;
  background: #edf2ff;
  overflow: hidden;
}

.mini-value {
  height: 100%;
  background: var(--blue);
}

.history-row {
  display: flex;
  justify-content: space-between;
  gap: 14px;
  border-bottom: 1px solid #f4f6fa;
  padding: 12px 0;
  transition: background 0.2s ease, box-shadow 0.2s ease;
}

.history-row.open-ok {
  background: rgba(40, 184, 88, 0.06);
  box-shadow: inset 0 0 0 1px rgba(40, 184, 88, 0.18);
}

.history-row.open-error {
  background: rgba(203, 68, 68, 0.06);
  box-shadow: inset 0 0 0 1px rgba(203, 68, 68, 0.18);
}

.history-main strong {
  display: block;
}

.history-main p {
  margin: 3px 0 0;
  color: #8a90a0;
  font-size: 12px;
  word-break: break-word;
}

.history-detail {
  margin-top: 6px;
}

.history-detail summary {
  list-style: none;
  cursor: pointer;
  color: #6f7d99;
  font-size: 12px;
  user-select: none;
}

.history-detail summary::-webkit-details-marker {
  display: none;
}

.history-detail summary::before {
  content: "+";
  display: inline-block;
  width: 14px;
  font-weight: 700;
}

.history-detail[open] summary::before {
  content: "-";
}

.history-main .history-error {
  color: #ba3f3f;
}

.history-status {
  height: 24px;
  border-radius: 999px;
  padding: 0 10px;
  display: inline-flex;
  align-items: center;
  font-size: 12px;
  font-weight: 700;
}

.history-actions {
  display: grid;
  justify-items: end;
  align-content: start;
  gap: 8px;
}

.folder-btn {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  border: 1px solid #e4e9f5;
  background: #fff;
  color: #6f7d99;
  cursor: pointer;
  display: grid;
  place-items: center;
  position: relative;
}

.folder-btn svg {
  width: 15px;
  height: 15px;
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}

.folder-btn::after {
  content: attr(data-tip);
  position: absolute;
  left: 50%;
  bottom: calc(100% + 8px);
  transform: translateX(-50%);
  background: #1f2634;
  color: #fff;
  border-radius: 8px;
  padding: 4px 7px;
  font-size: 11px;
  white-space: nowrap;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.16s ease;
}

.folder-btn:hover::after {
  opacity: 1;
}

.folder-feedback {
  font-size: 11px;
  font-weight: 700;
}

.folder-feedback.ok {
  color: #2b8f58;
}

.folder-feedback.error {
  color: #bf4141;
}

.history-status.done {
  background: #e9f9ef;
  color: #2b8f58;
}

.history-status.error {
  background: #fdeeee;
  color: #bf4141;
}

.empty {
  text-align: center;
  color: #9ba2b1;
  padding: 48px 0;
}

.toast {
  position: fixed;
  left: 50%;
  bottom: 28px;
  transform: translate(-50%, 76px);
  opacity: 0;
  background: #1f2634;
  color: #fff;
  border-radius: 999px;
  padding: 10px 14px;
  font-size: 13px;
  transition: all 0.24s ease;
  pointer-events: none;
}

.toast.show {
  transform: translate(-50%, 0);
  opacity: 1;
}

.custom-scroll::-webkit-scrollbar { width: 6px; height: 6px; }
.custom-scroll::-webkit-scrollbar-thumb { background: #dbe2f2; border-radius: 999px; }

@media (max-width: 900px) {
  .quark-app {
    flex-direction: column;
  }

  .sidebar {
    width: 100%;
    border-right: 0;
    border-bottom: 1px solid #eef1f6;
    flex-direction: row;
    justify-content: space-between;
    padding: 10px 12px;
  }

  .sidebar-title {
    margin: 0;
  }

  .side-nav {
    width: auto;
    flex-direction: row;
    gap: 8px;
  }

  .side-item {
    width: 72px;
    padding: 8px 0 6px;
  }

  .home-view,
  .transfer-header,
  .transfer-list {
    padding-left: 14px;
    padding-right: 14px;
  }

  .search-initial {
    transform: translateY(8vh);
  }

  .result-table-wrap {
    max-height: 52vh;
  }
}

@media (max-width: 620px) {
  .search-action-row {
    flex-direction: column;
    align-items: flex-start;
  }

  .action-buttons {
    width: 100%;
  }

  .primary-btn,
  .line-btn {
    flex: 1;
    min-width: 0;
    text-align: center;
  }

  .result-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .tabs {
    width: 100%;
  }

  .tab-btn {
    flex: 1;
    text-align: center;
  }
}
</style>
