<template>
  <div class="app-shell">
    <input
      ref="fileInput"
      type="file"
      multiple
      accept=".ncm"
      class="hidden-input"
      @change="handleFileSelect"
    />
    <input
      ref="folderInput"
      type="file"
      webkitdirectory
      class="hidden-input"
      @change="handleFileSelect"
    />

    <header class="topbar">
      <div class="brand">
        <div class="brand-mark">
          <span></span>
        </div>
        <div class="brand-copy">
          <h1>EchoConvert</h1>
          <p>回声转换</p>
        </div>
      </div>

      <nav class="nav-tabs" aria-label="view switch">
        <button
          type="button"
          :class="['nav-tab', { active: view === 'online' }]"
          @click="view = 'online'"
        >
          在线歌单
        </button>
        <button
          type="button"
          :class="['nav-tab', { active: view === 'offline' }]"
          @click="view = 'offline'"
        >
          本地转换
        </button>
      </nav>

      <div class="topbar-actions">
        <button type="button" class="icon-btn" @click="chooseDownloadDirectory" title="设置存储路径">
          <span class="icon folder"></span>
        </button>
        <button type="button" class="icon-btn" @click="pushInfoLog" title="显示说明">
          <span class="icon info"></span>
        </button>
      </div>
    </header>

    <main class="content">
      <div class="content-inner">
        <section v-if="view === 'online'" class="online-view">
          <article class="glass-panel hero-card">
            <h2>解析在线歌单</h2>
            <p class="subtext">
              输入网易云音乐歌单链接或歌单 ID，解析后可批量勾选并下载到当前存储目录。
            </p>

            <div class="playlist-form">
              <div class="input-shell">
                <span class="icon link"></span>
                <input
                  v-model="inputUrl"
                  type="text"
                  placeholder="粘贴歌单链接或 ID..."
                  @keydown.enter.prevent="analyzePlaylist"
                />
              </div>
              <button type="button" class="dark-btn" :disabled="playlistLoading" @click="analyzePlaylist">
                {{ playlistLoading ? '解析中...' : '解析' }}
              </button>
            </div>

            <div class="playlist-summary" v-if="playlistMeta">
              <div>
                <strong>{{ playlistMeta.title }}</strong>
                <span>创建者：{{ playlistMeta.creator }}</span>
              </div>
              <div>
                <strong>{{ playlistMeta.trackCount }}</strong>
                <span>已解析歌曲</span>
              </div>
              <div>
                <strong>{{ downloadDirName }}</strong>
                <span>下载目录</span>
              </div>
            </div>

            <div v-if="playlistMeta?.coverUrl" class="playlist-cover-strip">
              <img :src="playlistMeta.coverUrl" :alt="playlistMeta.title" class="playlist-cover" />
              <div>
                <strong>{{ playlistMeta.title }}</strong>
                <span>{{ playlistMeta.creator }}</span>
              </div>
            </div>
          </article>

          <article v-if="songs.length" class="song-panel">
            <div class="song-toolbar">
              <label class="check-all">
                <input v-model="allSelected" type="checkbox" @change="selectAllSongs" />
                <div>
                  <strong>全选所有歌曲</strong>
                  <span>已解析 {{ songs.length }} 个项目</span>
                </div>
              </label>

              <button
                type="button"
                class="accent-btn"
                :disabled="selectedCount === 0 || onlineDownloading"
                @click="downloadSelected"
              >
                {{ onlineDownloading ? '下载中...' : `下载选中项${selectedCount ? ` (${selectedCount})` : ''}` }}
              </button>
            </div>

            <div class="song-list custom-scroll">
              <article v-for="song in songs" :key="song.id" class="song-row">
                <input v-model="song.selected" type="checkbox" />
                <img
                  v-if="song.coverUrl"
                  :src="song.coverUrl"
                  :alt="song.title"
                  class="song-cover"
                />
                <div v-else class="song-avatar">
                  <span class="icon music"></span>
                </div>
                <div class="song-meta">
                  <strong>{{ song.title }}</strong>
                  <span>{{ song.artist }}</span>
                  <span class="song-submeta">{{ song.album }} · {{ song.durationLabel }}</span>
                </div>
                <span class="song-size">{{ song.sizeLabel }}</span>
              </article>
            </div>
          </article>
        </section>

        <section v-else class="offline-view">
          <article
            class="drop-panel"
            :class="{ dragging: isDragging }"
            @click="selectFiles"
            @dragover.prevent="isDragging = true"
            @dragleave.prevent="isDragging = false"
            @drop.prevent="handleDrop"
          >
            <div class="upload-orb">
              <span class="icon upload"></span>
            </div>
            <h2>批量 NCM 转换</h2>
            <p>
              支持拖拽多个 `.ncm` 文件到此区域，应用会自动解密并输出为
              <strong>MP3 / FLAC</strong>。
            </p>
            <button type="button" class="dark-btn" @click.stop="selectFiles">
              浏览本地文件
            </button>
          </article>

          <div class="offline-grid">
            <article class="glass-panel queue-panel">
              <div class="section-head">
                <div>
                  <h3>转换队列</h3>
                  <p>{{ files.length ? `已导入 ${files.length} 个文件` : '等待导入文件' }}</p>
                </div>
                <button
                  type="button"
                  class="outline-btn"
                  :disabled="!files.length || isConverting"
                  @click="startConversion"
                >
                  {{ isConverting ? `转换中 ${progress}%` : '开始转换' }}
                </button>
              </div>

              <div class="progress-line">
                <div class="progress-value" :style="{ width: `${progress}%` }"></div>
              </div>

              <div v-if="files.length" class="queue-list custom-scroll">
                <article v-for="file in files" :key="file.path" class="queue-row">
                  <div class="queue-meta">
                    <strong>{{ file.name }}</strong>
                    <span>{{ file.path }}</span>
                  </div>
                  <span class="queue-state" :class="file.status">{{ statusMap[file.status] }}</span>
                </article>
              </div>

              <div v-else class="empty-panel">
                队列为空，点击上方区域或直接拖拽 `.ncm` 文件开始。
              </div>
            </article>

            <article class="glass-panel log-panel">
              <div class="section-head">
                <div>
                  <h3>运行日志</h3>
                  <p>展示当前导入、下载与转换过程</p>
                </div>
                <button type="button" class="outline-btn" :disabled="!logs.length" @click="clearLogs">
                  清空日志
                </button>
              </div>

              <div v-if="logs.length" class="log-list custom-scroll">
                <article v-for="(log, index) in logs" :key="`${index}-${log.message}`" class="log-row">
                  <span class="log-badge" :class="log.type">{{ log.type }}</span>
                  <p>{{ log.message }}</p>
                </article>
              </div>

              <div v-else class="empty-panel compact">
                当前还没有日志输出。
              </div>
            </article>
          </div>
        </section>
      </div>
    </main>

    <footer class="taskbar">
      <div class="taskbar-side">
        <div class="task-icon" :class="{ active: activeTask }">
          <span class="icon refresh" :class="{ spinning: activeTask }"></span>
        </div>
        <div class="task-copy">
          <strong>{{ activeTask ? '任务处理中...' : '系统就绪' }}</strong>
          <span>{{ taskCount }} 个任务待办</span>
        </div>
      </div>

      <div class="taskbar-center">
        <template v-if="activeTask">
          <div class="taskbar-progress-head">
            <span>{{ activeTask.name }}</span>
            <strong>{{ activeTask.progress }}%</strong>
          </div>
          <div class="taskbar-progress">
            <div class="taskbar-progress-value" :style="{ width: `${activeTask.progress}%` }"></div>
          </div>
        </template>
        <div v-else class="taskbar-empty">
          <span></span>
          <p>{{ downloadDir }}</p>
          <span></span>
        </div>
      </div>

      <div class="taskbar-actions">
        <button type="button" class="outline-btn" @click="clearFiles">清空列表</button>
        <button type="button" class="green-btn" @click="chooseDownloadDirectory">存储路径</button>
      </div>
    </footer>
  </div>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const view = ref('offline')
const inputUrl = ref('')
const playlistMeta = ref(null)
const songs = ref([])
const allSelected = ref(false)
const playlistLoading = ref(false)
const onlineDownloading = ref(false)

const files = ref([])
const logs = ref([])
const tasks = ref([])
const isDragging = ref(false)
const isConverting = ref(false)
const progress = ref(0)
const downloadDir = ref('')
const fileInput = ref(null)
const folderInput = ref(null)

const statusMap = {
  pending: '待处理',
  processing: '处理中',
  done: '完成',
  error: '失败',
}

const selectedCount = computed(() => songs.value.filter((song) => song.selected).length)
const activeTask = computed(() => tasks.value[0] || null)
const taskCount = computed(() => tasks.value.length)
const downloadDirName = computed(() => {
  if (!downloadDir.value) return '未设置'
  const segments = downloadDir.value.split(/[\\/]/).filter(Boolean)
  return segments.at(-1) || downloadDir.value
})

function addLog(message, type = 'info') {
  logs.value.unshift({ message, type })
  logs.value = logs.value.slice(0, 80)
}

function normalizeFile(file) {
  const rawPath = file.path || file.webkitRelativePath || file.name
  return {
    name: file.name,
    path: rawPath,
    status: 'pending',
  }
}

function pushFiles(fileList) {
  const incoming = Array.from(fileList || [])
    .filter((file) => file.name.toLowerCase().endsWith('.ncm'))
    .map(normalizeFile)

  if (!incoming.length) {
    addLog('未检测到可导入的 .ncm 文件。', 'warn')
    return
  }

  let added = 0

  for (const file of incoming) {
    if (!files.value.some((item) => item.path === file.path)) {
      files.value.push(file)
      added += 1
    }
  }

  addLog(`已加入 ${added} 个文件到转换队列。`, 'info')
}

function createTask(name, initialProgress = 0) {
  const task = {
    id: `${Date.now()}-${Math.random()}`,
    name,
    progress: initialProgress,
  }

  tasks.value.push(task)
  return task
}

function settleTask(task, finalProgress = 100) {
  task.progress = finalProgress
  window.setTimeout(() => {
    tasks.value = tasks.value.filter((item) => item.id !== task.id)
  }, 1200)
}

async function loadSettings() {
  try {
    const settings = await invoke('get_app_settings')
    downloadDir.value = settings.downloadDir || ''
  } catch (error) {
    addLog(`读取设置失败：${String(error)}`, 'error')
  }
}

function handleFileSelect(event) {
  pushFiles(event.target.files)
  event.target.value = ''
}

function handleDrop(event) {
  isDragging.value = false
  pushFiles(event.dataTransfer?.files)
}

function selectFiles() {
  fileInput.value?.click()
}

function selectFolder() {
  folderInput.value?.click()
}

function clearLogs() {
  logs.value = []
}

function clearFiles() {
  if (isConverting.value) return
  files.value = []
  progress.value = 0
  addLog('已清空待转换列表。', 'info')
}

function pushInfoLog() {
  addLog(`当前在线下载目录：${downloadDir.value || '未设置'}`, 'info')
  addLog('本地转换会在源文件目录输出解密后的音频文件。', 'info')
}

async function chooseDownloadDirectory() {
  const next = window.prompt('请输入在线下载保存路径', downloadDir.value || '')
  if (!next) return

  try {
    const settings = await invoke('set_download_directory', { path: next })
    downloadDir.value = settings.downloadDir || next
    addLog(`已更新下载目录：${downloadDir.value}`, 'success')
  } catch (error) {
    addLog(`设置下载目录失败：${String(error)}`, 'error')
  }
}

async function analyzePlaylist() {
  if (!inputUrl.value.trim()) {
    addLog('请输入歌单链接或 ID。', 'warn')
    return
  }

  playlistLoading.value = true
  const task = createTask('解析在线歌单', 12)

  try {
    const result = await invoke('analyze_online_playlist', { query: inputUrl.value.trim() })
    task.progress = 100
    playlistMeta.value = result
    songs.value = result.tracks.map((song) => ({ ...song, selected: false }))
    allSelected.value = false
    addLog(`歌单解析完成：${result.title}，共 ${result.trackCount} 首。`, 'success')
  } catch (error) {
    task.progress = 100
    addLog(`歌单解析失败：${String(error)}`, 'error')
  } finally {
    playlistLoading.value = false
    settleTask(task)
  }
}

function selectAllSongs() {
  for (const song of songs.value) {
    song.selected = allSelected.value
  }
}

async function downloadSelected() {
  const selected = songs.value.filter((song) => song.selected)
  if (!selected.length || onlineDownloading.value) return

  onlineDownloading.value = true
  const task = createTask(`在线下载 ${selected.length} 首`, 8)
  addLog(`开始下载 ${selected.length} 首歌曲到 ${downloadDir.value || '默认目录'}。`, 'info')

  try {
    task.progress = 35
    const result = await invoke('download_online_tracks', {
      tracks: selected.map((song) => ({
        id: song.id,
        title: song.title,
        artist: song.artist,
      })),
    })
    task.progress = 100
    addLog(`在线下载完成：成功 ${result.successCount} 首，失败 ${result.failureCount} 首。`, 'success')

    for (const item of result.items) {
      addLog(`${item.title}：${item.message}${item.savedPath ? ` -> ${item.savedPath}` : ''}`, item.status === 'done' ? 'success' : 'error')
    }
  } catch (error) {
    task.progress = 100
    addLog(`在线下载失败：${String(error)}`, 'error')
  } finally {
    onlineDownloading.value = false
    settleTask(task)
  }
}

async function startConversion() {
  if (!files.value.length || isConverting.value) return

  isConverting.value = true
  progress.value = 8
  files.value = files.value.map((file) => ({ ...file, status: 'processing' }))
  const conversionTask = createTask(`本地转换 ${files.value.length} 项`, 8)
  addLog(`开始处理 ${files.value.length} 个文件。`, 'info')

  const paths = files.value.map((file) => file.path)

  try {
    conversionTask.progress = 45
    const result = await invoke('convert_ncm_files', { files: paths })
    progress.value = 100
    conversionTask.progress = 100

    const resultByPath = new Map(result.items.map((item) => [item.sourcePath, item]))
    files.value = files.value.map((file) => {
      const item = resultByPath.get(file.path)
      return {
        ...file,
        status: item?.status || 'error',
      }
    })

    addLog(`本地转换完成：成功 ${result.successCount} 个，失败 ${result.failureCount} 个。`, result.failureCount ? 'warn' : 'success')

    for (const item of result.items) {
      addLog(`${item.sourcePath}：${item.message}${item.outputPath ? ` -> ${item.outputPath}` : ''}`, item.status === 'done' ? 'success' : 'error')
    }
  } catch (error) {
    progress.value = 100
    conversionTask.progress = 100
    files.value = files.value.map((file) => ({ ...file, status: 'error' }))
    addLog(`本地转换失败：${String(error)}`, 'error')
  } finally {
    isConverting.value = false
    settleTask(conversionTask, 100)
  }
}

onMounted(() => {
  loadSettings()
})
</script>

<style scoped>
:global(*) {
  box-sizing: border-box;
}

:global(body) {
  margin: 0;
  min-width: 320px;
  min-height: 100vh;
  background: #e8e2d8;
  color: #4a5553;
  font-family: "Noto Sans SC", "PingFang SC", "Segoe UI", sans-serif;
  -webkit-font-smoothing: antialiased;
}

:global(button),
:global(input) {
  font: inherit;
}

#app,
.app-shell {
  min-height: 100vh;
}

.app-shell {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.topbar {
  height: 80px;
  display: flex;
  flex: none;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  padding: 0 48px;
  background: rgba(255, 255, 255, 0.3);
  border-bottom: 1px solid rgba(191, 198, 196, 0.2);
}

.brand {
  display: flex;
  align-items: center;
  gap: 12px;
}

.brand-mark {
  width: 40px;
  height: 40px;
  display: grid;
  place-items: center;
  border-radius: 14px;
  background: #6f8f72;
}

.brand-mark span,
.icon {
  display: inline-block;
  position: relative;
}

.brand-mark span {
  width: 20px;
  height: 20px;
}

.brand-mark span::before,
.brand-mark span::after {
  content: "";
  position: absolute;
  inset: 0;
  border: 2px solid #fff;
  border-color: #fff transparent transparent transparent;
  border-radius: 50%;
}

.brand-mark span::after {
  inset: 5px 0 0;
}

.brand-copy h1,
.brand-copy p {
  margin: 0;
  line-height: 1;
}

.brand-copy h1 {
  font-size: 20px;
  font-weight: 700;
  color: #4a5553;
}

.brand-copy p {
  margin-top: 4px;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.35em;
  text-transform: uppercase;
  color: rgba(111, 143, 114, 0.68);
}

.nav-tabs {
  display: flex;
  gap: 56px;
}

.nav-tab {
  position: relative;
  border: 0;
  background: transparent;
  color: #bfc6c4;
  cursor: pointer;
  transition: color 0.2s ease;
}

.nav-tab.active {
  color: #6f8f72;
  font-weight: 700;
}

.nav-tab.active::after {
  content: "";
  position: absolute;
  left: 50%;
  bottom: -12px;
  width: 20px;
  height: 3px;
  background: #6f8f72;
  border-radius: 99px;
  transform: translateX(-50%);
}

.topbar-actions {
  display: flex;
  gap: 18px;
}

.icon-btn,
.dark-btn,
.outline-btn,
.accent-btn,
.green-btn {
  border: 0;
  cursor: pointer;
  transition: transform 0.18s ease, background 0.18s ease, opacity 0.18s ease;
}

.icon-btn:hover,
.dark-btn:hover,
.outline-btn:hover,
.accent-btn:hover,
.green-btn:hover {
  transform: translateY(-1px);
}

.icon-btn {
  width: 36px;
  height: 36px;
  display: grid;
  place-items: center;
  background: transparent;
  color: #bfc6c4;
}

.content {
  flex: 1;
  overflow: auto;
  padding: 48px;
}

.content-inner {
  max-width: 1100px;
  margin: 0 auto;
}

.online-view,
.offline-view {
  min-height: 100%;
}

.glass-panel,
.song-panel {
  background: rgba(255, 255, 255, 0.45);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(191, 198, 196, 0.3);
  box-shadow: 0 10px 30px rgba(74, 85, 83, 0.06);
}

.hero-card {
  padding: 40px;
  border-radius: 32px;
}

.hero-card h2,
.drop-panel h2,
.section-head h3 {
  margin: 0 0 10px;
  color: #4a5553;
}

.hero-card h2,
.drop-panel h2 {
  font-size: 32px;
}

.subtext,
.drop-panel p,
.section-head p,
.queue-meta span,
.song-meta span,
.task-copy span {
  margin: 0;
  color: #8f9996;
  line-height: 1.7;
}

.playlist-form {
  display: flex;
  gap: 16px;
  margin-top: 28px;
}

.playlist-summary {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
  margin-top: 22px;
}

.playlist-summary > div {
  padding: 16px 18px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.28);
}

.playlist-summary strong,
.task-copy strong {
  display: block;
  color: #4a5553;
}

.playlist-summary span {
  display: block;
  margin-top: 6px;
  color: #8f9996;
  font-size: 12px;
}

.playlist-cover-strip {
  display: flex;
  align-items: center;
  gap: 14px;
  margin-top: 18px;
  padding: 14px;
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.26);
}

.playlist-cover {
  width: 64px;
  height: 64px;
  object-fit: cover;
  border-radius: 16px;
  flex: none;
  box-shadow: 0 8px 18px rgba(74, 85, 83, 0.08);
}

.playlist-cover-strip strong {
  display: block;
  color: #4a5553;
}

.playlist-cover-strip span {
  display: block;
  margin-top: 6px;
  color: #8f9996;
  font-size: 12px;
}

.input-shell {
  position: relative;
  flex: 1;
}

.input-shell input {
  width: 100%;
  height: 56px;
  padding: 0 16px 0 48px;
  border: 1px solid rgba(191, 198, 196, 0.3);
  border-radius: 18px;
  background: rgba(255, 255, 255, 0.6);
  outline: none;
}

.input-shell input:focus {
  border-color: rgba(111, 143, 114, 0.5);
  box-shadow: 0 0 0 4px rgba(111, 143, 114, 0.12);
}

.song-panel {
  margin-top: 32px;
  overflow: hidden;
  border-radius: 32px;
}

.song-toolbar,
.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.song-toolbar {
  padding: 24px 32px;
  border-bottom: 1px solid rgba(191, 198, 196, 0.16);
  background: rgba(255, 255, 255, 0.35);
}

.check-all {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
}

.check-all strong,
.song-meta strong,
.queue-meta strong {
  display: block;
  color: #4a5553;
}

.check-all span {
  font-size: 11px;
  color: #8f9996;
}

.song-list,
.queue-list,
.log-list {
  overflow: auto;
}

.song-row,
.queue-row,
.log-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.song-row {
  padding: 20px 32px;
  border-top: 1px solid rgba(191, 198, 196, 0.12);
}

.song-avatar {
  width: 48px;
  height: 48px;
  display: grid;
  place-items: center;
  border-radius: 16px;
  background: rgba(191, 198, 196, 0.2);
}

.song-cover {
  width: 48px;
  height: 48px;
  object-fit: cover;
  border-radius: 16px;
  flex: none;
  box-shadow: 0 4px 10px rgba(74, 85, 83, 0.08);
}

.song-meta {
  flex: 1;
  min-width: 0;
}

.song-meta strong,
.queue-meta strong {
  word-break: break-word;
}

.song-size {
  padding: 6px 12px;
  border-radius: 999px;
  background: rgba(191, 198, 196, 0.14);
  color: rgba(74, 85, 83, 0.55);
  font-size: 12px;
  font-weight: 700;
}

.song-submeta {
  margin-top: 4px;
  font-size: 12px;
  color: #a1aaa8;
}

.drop-panel {
  min-height: 380px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 24px;
  border: 2px dashed rgba(191, 198, 196, 0.45);
  border-radius: 48px;
  background: rgba(255, 255, 255, 0.4);
  text-align: center;
  transition: background 0.22s ease, border-color 0.22s ease, box-shadow 0.22s ease;
}

.drop-panel.dragging {
  background: rgba(111, 143, 114, 0.1);
  border-color: #6f8f72;
  box-shadow: inset 0 0 0 1px rgba(111, 143, 114, 0.08);
}

.upload-orb {
  width: 80px;
  height: 80px;
  display: grid;
  place-items: center;
  margin-bottom: 24px;
  border-radius: 999px;
  background: #e8e2d8;
  color: #6f8f72;
  box-shadow: 0 8px 18px rgba(74, 85, 83, 0.08);
}

.drop-panel strong {
  color: #f2a65a;
}

.dark-btn,
.accent-btn,
.outline-btn,
.green-btn {
  padding: 14px 24px;
  border-radius: 18px;
  font-weight: 700;
}

.dark-btn {
  background: #4a5553;
  color: #fff;
}

.dark-btn:hover,
.green-btn:hover {
  background: #6f8f72;
}

.dark-btn:disabled,
.accent-btn:disabled,
.outline-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.outline-btn {
  background: transparent;
  color: #4a5553;
  border: 1px solid rgba(191, 198, 196, 0.5);
}

.accent-btn {
  background: #f2a65a;
  color: #fff;
  box-shadow: 0 10px 20px rgba(242, 166, 90, 0.18);
}

.green-btn {
  background: #6f8f72;
  color: #fff;
}

.offline-grid {
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(320px, 0.9fr);
  gap: 24px;
  margin-top: 24px;
}

.queue-panel,
.log-panel {
  padding: 28px;
  border-radius: 32px;
}

.progress-line,
.taskbar-progress {
  height: 8px;
  border-radius: 999px;
  overflow: hidden;
  background: rgba(191, 198, 196, 0.24);
}

.progress-line {
  margin-top: 18px;
}

.progress-value,
.taskbar-progress-value {
  height: 100%;
  background: #6f8f72;
  border-radius: inherit;
  transition: width 0.25s ease;
}

.queue-list,
.log-list {
  max-height: 360px;
  margin-top: 18px;
  padding-right: 6px;
}

.queue-row,
.log-row {
  align-items: flex-start;
  padding: 16px 0;
  border-top: 1px solid rgba(191, 198, 196, 0.14);
}

.queue-meta,
.log-row p {
  flex: 1;
  min-width: 0;
}

.queue-meta span,
.log-row p {
  word-break: break-word;
}

.queue-state,
.log-badge {
  flex: none;
  padding: 6px 12px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 700;
}

.queue-state.pending,
.log-badge.info {
  background: rgba(191, 198, 196, 0.24);
  color: #6f8f72;
}

.queue-state.processing,
.log-badge.warn {
  background: rgba(242, 166, 90, 0.14);
  color: #d88739;
}

.queue-state.done,
.log-badge.success {
  background: rgba(111, 143, 114, 0.14);
  color: #6f8f72;
}

.queue-state.error,
.log-badge.error {
  background: rgba(204, 122, 110, 0.16);
  color: #b45e4f;
}

.empty-panel {
  margin-top: 18px;
  padding: 28px 20px;
  border-radius: 24px;
  background: rgba(255, 255, 255, 0.22);
  color: #8f9996;
  text-align: center;
}

.empty-panel.compact {
  min-height: 160px;
  display: grid;
  place-items: center;
}

.taskbar {
  height: 96px;
  display: flex;
  flex: none;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  padding: 0 48px;
  background: rgba(255, 255, 255, 0.6);
  border-top: 1px solid rgba(191, 198, 196, 0.2);
  backdrop-filter: blur(8px);
}

.taskbar-side,
.taskbar-actions {
  width: 25%;
  display: flex;
  align-items: center;
  gap: 14px;
}

.taskbar-actions {
  justify-content: flex-end;
}

.task-icon {
  width: 48px;
  height: 48px;
  display: grid;
  place-items: center;
  border-radius: 18px;
  background: rgba(191, 198, 196, 0.16);
}

.task-icon.active {
  background: rgba(242, 166, 90, 0.14);
}

.taskbar-center {
  flex: 1;
  max-width: 480px;
}

.taskbar-progress-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
  gap: 16px;
  font-size: 11px;
  font-weight: 700;
  color: #6f8f72;
}

.taskbar-empty {
  display: flex;
  align-items: center;
  gap: 16px;
  color: rgba(74, 85, 83, 0.55);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.1em;
}

.taskbar-empty span {
  flex: 1;
  height: 1px;
  background: rgba(191, 198, 196, 0.3);
}

.taskbar-empty p {
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.hidden-input {
  display: none;
}

.custom-scroll::-webkit-scrollbar {
  width: 4px;
}

.custom-scroll::-webkit-scrollbar-thumb {
  background: #bfc6c4;
  border-radius: 999px;
}

.icon.folder,
.icon.info,
.icon.link,
.icon.music,
.icon.upload,
.icon.refresh {
  width: 18px;
  height: 18px;
}

.icon.folder::before {
  content: "";
  position: absolute;
  left: 0;
  top: 4px;
  width: 18px;
  height: 11px;
  border: 2px solid currentColor;
  border-radius: 4px;
}

.icon.folder::after {
  content: "";
  position: absolute;
  left: 2px;
  top: 0;
  width: 8px;
  height: 5px;
  border: 2px solid currentColor;
  border-bottom: 0;
  border-radius: 4px 4px 0 0;
}

.icon.info::before {
  content: "i";
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  border: 2px solid currentColor;
  border-radius: 50%;
  font-size: 12px;
  font-weight: 700;
}

.icon.link::before,
.icon.link::after {
  content: "";
  position: absolute;
  width: 10px;
  height: 6px;
  border: 2px solid currentColor;
  border-radius: 999px;
  transform: rotate(-24deg);
}

.icon.link::before {
  left: 0;
  top: 6px;
}

.icon.link::after {
  right: 0;
  top: 2px;
}

.icon.music::before {
  content: "";
  position: absolute;
  left: 8px;
  top: 1px;
  width: 2px;
  height: 12px;
  background: currentColor;
}

.icon.music::after {
  content: "";
  position: absolute;
  left: 3px;
  top: 10px;
  width: 7px;
  height: 7px;
  border: 2px solid currentColor;
  border-radius: 50%;
}

.icon.upload::before {
  content: "";
  position: absolute;
  left: 8px;
  top: 1px;
  width: 2px;
  height: 11px;
  background: currentColor;
}

.icon.upload::after {
  content: "";
  position: absolute;
  left: 4px;
  top: 1px;
  width: 8px;
  height: 8px;
  border-left: 2px solid currentColor;
  border-top: 2px solid currentColor;
  transform: rotate(45deg);
}

.icon.refresh::before,
.icon.refresh::after {
  content: "";
  position: absolute;
  border: 2px solid currentColor;
  border-color: currentColor transparent transparent transparent;
  border-radius: 50%;
}

.icon.refresh::before {
  inset: 1px;
}

.icon.refresh::after {
  inset: 4px;
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }

  to {
    transform: rotate(360deg);
  }
}

@media (max-width: 960px) {
  .topbar,
  .content,
  .taskbar {
    padding-left: 20px;
    padding-right: 20px;
  }

  .nav-tabs {
    gap: 20px;
  }

  .offline-grid {
    grid-template-columns: 1fr;
  }

  .playlist-summary {
    grid-template-columns: 1fr;
  }

  .taskbar {
    height: auto;
    flex-wrap: wrap;
    padding-top: 16px;
    padding-bottom: 16px;
  }

  .taskbar-side,
  .taskbar-actions,
  .taskbar-center {
    width: 100%;
    max-width: none;
  }

  .taskbar-actions {
    justify-content: flex-start;
  }
}

@media (max-width: 720px) {
  .topbar {
    height: auto;
    flex-wrap: wrap;
    padding-top: 18px;
    padding-bottom: 18px;
  }

  .content {
    padding: 20px 14px;
  }

  .hero-card,
  .queue-panel,
  .log-panel {
    padding: 20px;
    border-radius: 24px;
  }

  .drop-panel {
    min-height: 300px;
    border-radius: 28px;
  }

  .playlist-form,
  .song-toolbar,
  .section-head,
  .song-row,
  .queue-row,
  .log-row {
    flex-direction: column;
    align-items: stretch;
  }

  .nav-tabs {
    width: 100%;
    justify-content: space-between;
  }

  .taskbar-actions {
    flex-wrap: wrap;
  }
}
</style>
