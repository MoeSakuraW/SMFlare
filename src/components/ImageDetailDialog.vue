<script setup lang="ts">
import {ref, computed} from 'vue'
import {ElMessage} from 'element-plus'
import {CopyDocument, Loading, Close, ZoomIn} from '@element-plus/icons-vue'
import {openUrl} from '@tauri-apps/plugin-opener'

// 标准化的图片数据接口
export interface ImageDetailData {
  url: string           // 图片URL
  pageUrl: string       // SM.MS页面URL
  filename: string      // 文件名
  storename: string     // 存储名
  width: number         // 宽度
  height: number        // 高度
  size: number          // 文件大小
  hash: string          // 哈希值
  createdAt: string     // 上传时间
  fileType?: string     // 文件类型（可选）
  remark?: string       // 备注信息（可选）
}

// Props
interface Props {
  modelValue: boolean
  image: ImageDetailData | null
  title?: string
}

const props = withDefaults(defineProps<Props>(), {
  title: ''
})

// Emits
const emit = defineEmits<{
  'update:modelValue': [value: boolean]
  'closed': []
}>()

// 内部状态
const imageLoading = ref(true)
const imageError = ref(false)

// 计算属性
const dialogVisible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
})

const dialogTitle = computed(() => {
  return props.title || props.image?.filename || '图片详情'
})

// 图片加载完成
const handleImageLoad = () => {
  imageLoading.value = false
}

// 图片加载失败
const handleImageError = () => {
  imageLoading.value = false
  imageError.value = true
}

// 关闭弹窗
const handleClosed = () => {
  imageLoading.value = true
  imageError.value = false
  emit('closed')
}

// 复制链接
const copyLink = async (url: string) => {
  try {
    await navigator.clipboard.writeText(url)
    ElMessage.success('链接已复制')
  } catch {
    ElMessage.error('复制失败，请手动复制')
  }
}

// 在 SM.MS 打开
const openInSmms = async (page: string) => {
  try {
    await openUrl(page)
  } catch {
    ElMessage.error('打开链接失败')
  }
}

// 格式化文件大小
const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
}
</script>

<template>
  <el-dialog
      v-model="dialogVisible"
      :title="dialogTitle"
      width="80%"
      top="5vh"
      destroy-on-close
      class="image-detail-dialog"
      @closed="handleClosed"
  >
    <div v-if="image" class="detail-content">
      <!-- 图片预览区 -->
      <div class="detail-preview">
        <div v-if="imageLoading" class="loading-placeholder">
          <el-icon class="is-loading" :size="40">
            <Loading/>
          </el-icon>
          <span>加载中...</span>
        </div>
        <div v-if="imageError" class="error-placeholder">
          <el-icon :size="40" color="#F56C6C">
            <Close/>
          </el-icon>
          <span>图片加载失败</span>
        </div>
        <el-image
            v-show="!imageLoading && !imageError"
            :src="image.url"
            :preview-src-list="[image.url]"
            fit="contain"
            class="detail-image"
            @load="handleImageLoad"
            @error="handleImageError"
        >
          <template #placeholder>
            <div class="image-slot">加载中...</div>
          </template>
        </el-image>
        <div v-if="!imageLoading && !imageError" class="zoom-hint">
          <el-icon>
            <ZoomIn/>
          </el-icon>
          <span>点击图片可放大查看</span>
        </div>
      </div>

      <!-- 图片信息区 -->
      <div class="detail-info">
        <h3>图片信息</h3>
        <el-descriptions :column="1" border label-width="80px">
          <el-descriptions-item label="文件名">
            {{ image.filename }}
          </el-descriptions-item>
          <el-descriptions-item label="存储名">
            {{ image.storename }}
          </el-descriptions-item>
          <el-descriptions-item label="尺寸">
            {{ image.width }} × {{ image.height }} px
          </el-descriptions-item>
          <el-descriptions-item label="文件大小">
            {{ formatFileSize(image.size) }}
          </el-descriptions-item>
          <el-descriptions-item v-if="image.fileType" label="文件类型">
            {{ image.fileType.toUpperCase() }}
          </el-descriptions-item>
          <el-descriptions-item label="上传时间">
            {{ image.createdAt }}
          </el-descriptions-item>
          <el-descriptions-item label="Hash">
            {{ image.hash }}
          </el-descriptions-item>
          <el-descriptions-item v-if="image.remark" label="备注">
            {{ image.remark }}
          </el-descriptions-item>

          <!-- 插槽：用于额外的自定义字段 -->
          <slot name="extra-fields"></slot>
        </el-descriptions>

        <div class="detail-actions">
          <el-button :icon="CopyDocument" class="btn-content" @click="copyLink(image.url)">
            复制链接
          </el-button>
          <el-button class="btn-secondary" @click="openInSmms(image.pageUrl)">
            在 SM.MS 打开
          </el-button>
        </div>
      </div>
    </div>
  </el-dialog>
</template>

<style scoped>
.detail-content {
  display: flex;
  gap: 24px;
  min-height: 400px;
}

.detail-preview {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background: var(--color-bg-secondary);
  border-radius: var(--radius-lg);
  min-height: 400px;
  position: relative;
}

.loading-placeholder,
.error-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  color: var(--color-text-secondary);
}

.loading-placeholder .is-loading {
  animation: rotating 2s linear infinite;
}

@keyframes rotating {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.detail-image {
  max-width: 100%;
  max-height: 500px;
  border-radius: var(--radius-md);
}

.zoom-hint {
  position: absolute;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: rgba(0, 0, 0, 0.65);
  backdrop-filter: blur(4px);
  color: rgba(255, 255, 255, 0.95);
  border-radius: var(--radius-md);
  font-size: var(--font-size-small);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.detail-info {
  width: 380px;
  flex-shrink: 0;
}

.detail-info h3 {
  font-size: var(--font-size-h5);
  color: var(--color-text-primary);
  margin-bottom: 16px;
  font-weight: var(--font-weight-medium);
}

.detail-actions {
  margin-top: 20px;
  display: flex;
  gap: var(--gap-md);
}

.detail-actions .el-button {
  flex: 1;
}

.detail-info :deep(.el-descriptions__label) {
  width: 80px;
  min-width: 80px;
  white-space: nowrap;
}

.image-slot {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  height: 100%;
  background: var(--color-bg-secondary);
  color: var(--color-text-secondary);
}

/* 响应式布局 */
@media (max-width: 768px) {
  .detail-content {
    flex-direction: column;
  }

  .detail-info {
    width: 100%;
  }
}
</style>
