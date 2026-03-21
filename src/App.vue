<template>
  <div class="container">
    <header>
      <h1>🎵 NCM to MP3 Converter</h1>
      <p class="subtitle">Convert NetEase Cloud Music files to MP3 format</p>
    </header>

    <main>
      <div class="upload-section">
        <div 
          class="drop-zone" 
          @drop="handleDrop"
          @dragover.prevent="isDragging = true"
          @dragleave="isDragging = false"
          :class="{ dragging: isDragging }"
        >
          <div class="drop-content">
            <p>📂 Drag NCM files or folder here</p>
            <p class="or">or</p>
            <button @click="selectFiles" class="btn btn-primary">Select Files</button>
            <button @click="selectFolder" class="btn btn-secondary">Select Folder</button>
          </div>
        </div>
      </div>

      <div v-if="files.length > 0" class="file-list">
        <h2>Files to Convert ({{ files.length }})</h2>
        <ul>
          <li v-for="(file, idx) in files" :key="idx" class="file-item">
            <span class="name">{{ file.name }}</span>
            <span v-if="file.status === 'done'" class="badge success">✓ Done</span>
            <span v-else-if="file.status === 'error'" class="badge error">✗ Error</span>
            <span v-else class="badge pending">⏳ Pending</span>
          </li>
        </ul>
      </div>

      <div v-if="logs.length > 0" class="logs-section">
        <h2>Conversion Logs</h2>
        <div class="logs">
          <p v-for="(log, idx) in logs" :key="idx" :class="'log ' + log.type">
            {{ log.message }}
          </p>
        </div>
      </div>

      <button 
        v-if="files.length > 0 && !isConverting"
        @click="startConversion"
        class="btn btn-success"
      >
        🚀 Start Conversion
      </button>
      <button 
        v-if="isConverting"
        disabled
        class="btn btn-disabled"
      >
        Converting... {{ progress }}%
      </button>
    </main>

    <footer>
      <p>Local processing • No uploads • Open source</p>
    </footer>
  </div>
</template>

<script>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'

export default {
  name: 'App',
  setup() {
    const files = ref([])
    const logs = ref([])
    const isDragging = ref(false)
    const isConverting = ref(false)
    const progress = ref(0)

    const addLog = (message, type = 'info') => {
      logs.value.push({ message, type })
      // Keep only last 50 logs
      if (logs.value.length > 50) {
        logs.value.shift()
      }
    }

    const handleDrop = async (e) => {
      isDragging.value = false
      const items = e.dataTransfer.items
      if (!items) return

      for (let item of items) {
        const entry = item.webkitGetAsEntry()
        if (entry && entry.isFile) {
          const file = item.getAsFile()
          if (file.name.endsWith('.ncm')) {
            files.value.push({
              name: file.name,
              path: file.path || file.name,
              status: 'pending',
              file
            })
            addLog(`Added: ${file.name}`, 'info')
          }
        }
      }
    }

    const selectFiles = async () => {
      try {
        const selected = await open({
          multiple: true,
          filters: [
            {
              name: 'NCM Files',
              extensions: ['ncm']
            }
          ]
        })
        
        if (Array.isArray(selected)) {
          selected.forEach(path => {
            const name = path.split('/').pop()
            files.value.push({
              name,
              path,
              status: 'pending'
            })
            addLog(`Added: ${name}`, 'info')
          })
        }
      } catch (err) {
        addLog(`Error selecting files: ${err}`, 'error')
      }
    }

    const selectFolder = async () => {
      try {
        const selected = await open({
          directory: true
        })
        
        if (selected) {
          addLog(`Scanning folder: ${selected}`, 'info')
          // Note: In real implementation, backend will scan recursively
          files.value = []
          addLog('Ready to convert files from folder', 'info')
        }
      } catch (err) {
        addLog(`Error selecting folder: ${err}`, 'error')
      }
    }

    const startConversion = async () => {
      if (files.value.length === 0) return
      
      isConverting.value = true
      progress.value = 0
      addLog('🚀 Starting conversion...', 'info')

      try {
        // Call backend Rust function
        const result = await invoke('convert_ncm_files', {
          files: files.value.map(f => f.path)
        })
        
        addLog('✅ Conversion completed!', 'success')
        files.value.forEach(f => f.status = 'done')
      } catch (err) {
        addLog(`❌ Conversion failed: ${err}`, 'error')
        files.value.forEach(f => f.status = 'error')
      } finally {
        isConverting.value = false
        progress.value = 0
      }
    }

    return {
      files,
      logs,
      isDragging,
      isConverting,
      progress,
      handleDrop,
      selectFiles,
      selectFolder,
      startConversion
    }
  }
}
</script>

<style scoped>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

.container {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #333;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
}

header {
  background: rgba(0, 0, 0, 0.2);
  color: white;
  padding: 2rem;
  text-align: center;
  backdrop-filter: blur(10px);
}

header h1 {
  font-size: 2.5rem;
  margin-bottom: 0.5rem;
}

header .subtitle {
  font-size: 1.1rem;
  opacity: 0.9;
}

main {
  flex: 1;
  padding: 2rem;
  max-width: 800px;
  width: 100%;
  margin: 0 auto;
}

.upload-section {
  margin-bottom: 2rem;
}

.drop-zone {
  border: 3px dashed rgba(255, 255, 255, 0.5);
  border-radius: 12px;
  padding: 3rem 2rem;
  text-align: center;
  background: rgba(255, 255, 255, 0.1);
  cursor: pointer;
  transition: all 0.3s ease;
  backdrop-filter: blur(10px);
}

.drop-zone:hover,
.drop-zone.dragging {
  border-color: white;
  background: rgba(255, 255, 255, 0.2);
  transform: scale(1.02);
}

.drop-content {
  color: white;
}

.drop-content p {
  font-size: 1.1rem;
  margin: 0.5rem 0;
}

.drop-content .or {
  opacity: 0.7;
  font-size: 0.9rem;
  margin: 1rem 0;
}

.file-list {
  background: rgba(255, 255, 255, 0.95);
  border-radius: 12px;
  padding: 1.5rem;
  margin-bottom: 2rem;
}

.file-list h2 {
  margin-bottom: 1rem;
  color: #333;
}

.file-list ul {
  list-style: none;
}

.file-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 0.8rem;
  border-bottom: 1px solid #f0f0f0;
}

.file-item:last-child {
  border-bottom: none;
}

.file-item .name {
  flex: 1;
  word-break: break-all;
}

.badge {
  padding: 0.3rem 0.8rem;
  border-radius: 20px;
  font-size: 0.85rem;
  font-weight: bold;
  margin-left: 1rem;
}

.badge.success {
  background: #d4edda;
  color: #155724;
}

.badge.error {
  background: #f8d7da;
  color: #721c24;
}

.badge.pending {
  background: #fff3cd;
  color: #856404;
}

.logs-section {
  background: rgba(0, 0, 0, 0.3);
  border-radius: 12px;
  padding: 1.5rem;
  margin-bottom: 2rem;
  max-height: 300px;
  overflow-y: auto;
}

.logs-section h2 {
  color: white;
  margin-bottom: 1rem;
}

.logs {
  font-family: 'Courier New', monospace;
  font-size: 0.9rem;
}

.log {
  color: rgba(255, 255, 255, 0.8);
  margin: 0.3rem 0;
  padding: 0.3rem 0;
}

.log.success {
  color: #90ee90;
}

.log.error {
  color: #ff6b6b;
}

.log.info {
  color: #87ceeb;
}

.btn {
  padding: 0.8rem 1.5rem;
  border: none;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: bold;
  cursor: pointer;
  transition: all 0.3s ease;
  margin: 0.5rem;
}

.btn-primary,
.btn-secondary {
  background: white;
  color: #667eea;
  margin-top: 0.5rem;
}

.btn-primary:hover {
  background: #f0f0f0;
  transform: translateY(-2px);
}

.btn-secondary {
  background: rgba(255, 255, 255, 0.8);
}

.btn-secondary:hover {
  background: white;
}

.btn-success {
  background: #28a745;
  color: white;
  width: 100%;
  padding: 1rem;
}

.btn-success:hover {
  background: #218838;
  transform: translateY(-2px);
}

.btn-disabled {
  background: #ccc;
  color: white;
  cursor: not-allowed;
  width: 100%;
  padding: 1rem;
}

footer {
  background: rgba(0, 0, 0, 0.2);
  color: white;
  text-align: center;
  padding: 1rem;
  backdrop-filter: blur(10px);
}

footer p {
  opacity: 0.8;
}

@media (max-width: 600px) {
  header h1 {
    font-size: 1.8rem;
  }

  main {
    padding: 1rem;
  }

  .drop-zone {
    padding: 2rem 1rem;
  }
}
</style>
