<script setup lang="ts">
import {CopyDocument, Loading, Refresh} from '@element-plus/icons-vue'
import ImageDetailDialog, {type ImageDetailData} from './ImageDetailDialog.vue'

interface SmmsUploadItem {
  file_id: number
  width: number
  height: number
  filename: string
  storename: string
  size: number
  path: string
  hash: string
  created_at: string
  url: string
  delete: string
  page: string
}

interface Props {
  uploadHistory: SmmsUploadItem[]
  historyLoading: boolean
  d1ConfigExists: boolean
  smmsConfigExists: boolean
  isLoadingMore: boolean
  hasMore: boolean
  imageDetailData: ImageDetailData | null
  detailDialogVisible: boolean
}

interface Emits {
  (e: 'navigate-to', menu: string): void

  (e: 'copy-link', url: string): void

  (e: 'open-detail', item: SmmsUploadItem): void

  (e: 'update:detailDialogVisible', value: boolean): void

  (e: 'refresh'): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()

const handleNavigate = (menu: string) => emit('navigate-to', menu)
const handleCopyLink = (url: string) => emit('copy-link', url)
const handleOpenDetail = (item: SmmsUploadItem) => emit('open-detail', item)
const handleDialogUpdate = (value: boolean) => emit('update:detailDialogVisible', value)
const handleRefresh = () => emit('refresh')
</script>

<template>
  <div class="gallery-section">
    <div class="section-header">
      <h2>相册</h2>
      <el-button
          v-if="d1ConfigExists && smmsConfigExists"
          :icon="Refresh"
          :loading="historyLoading"
          :disabled="historyLoading"
          @click="handleRefresh"
      >
        刷新
      </el-button>
    </div>

    <div v-loading="historyLoading" class="gallery-content">
      <div v-if="uploadHistory.length === 0" class="empty-state">
        <!-- 未配置数据库 -->
        <el-empty v-if="!d1ConfigExists" description="请先配置数据库">
          <template #description>
            <p>请先配置数据库</p>
            <p class="empty-hint">
              前往
              <el-button text class="btn-guide" @click="handleNavigate('database-settings')">
                数据库设置
              </el-button>
              配置 Cloudflare D1
            </p>
          </template>
        </el-empty>
        <!-- 未登录 SM.MS -->
        <el-empty v-else-if="!smmsConfigExists" description="请先登录 SM.MS">
          <template #description>
            <p>请先登录 SM.MS</p>
            <p class="empty-hint">
              前往
              <el-button text class="btn-guide" @click="handleNavigate('image-bed-settings')">
                图床设置
              </el-button>
              登录 SM.MS
            </p>
          </template>
        </el-empty>
        <!-- 已配置但无数据 -->
        <el-empty v-else description="暂无上传记录">
          <template #description>
            <p>暂无上传记录</p>
          </template>
        </el-empty>
      </div>

      <div v-else class="image-container layout-grid">
        <div v-for="item in uploadHistory" :key="item.hash" class="image-card" @click="handleOpenDetail(item)">
          <div class="image-preview">
            <img :src="item.url" :alt="item.filename" loading="lazy"/>
          </div>
          <div class="image-info">
            <div class="image-name">{{ item.filename }}</div>
            <div class="image-meta">
              <span>{{ (item.size / 1024).toFixed(2) }} KB</span>
              <span>{{ item.width }} × {{ item.height }}</span>
            </div>
            <div class="image-date">{{ item.created_at }}</div>
            <div class="image-actions">
              <el-button :icon="CopyDocument" text class="btn-content"
                         @click.stop="handleCopyLink(item.url)">
                复制链接
              </el-button>
              <el-button text class="btn-content" @click.stop="handleOpenDetail(item)">
                查看详情
              </el-button>
            </div>
          </div>
        </div>
      </div>

      <div v-if="isLoadingMore" class="loading-more">
        <el-icon class="is-loading">
          <Loading/>
        </el-icon>
        <span>加载中...</span>
      </div>

      <div v-if="!hasMore && uploadHistory.length > 0" class="no-more">
        没有更多了
      </div>
    </div>

    <!-- 图片详情弹窗 -->
    <ImageDetailDialog
        :model-value="detailDialogVisible"
        :image="imageDetailData"
        @update:model-value="handleDialogUpdate"
    />
  </div>
</template>

<style scoped>
.gallery-section {
  max-width: var(--container-md);
  margin: 0 auto;
}

.section-header {
  margin-bottom: var(--spacing-xl);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.section-header h2 {
  font-size: var(--font-size-h2);
  color: var(--color-text-primary);
  font-weight: var(--font-weight-medium);
  margin: 0;
}

/* 相册样式 */
.gallery-content {
  min-height: 400px;
}

.empty-hint {
  margin-top: 10px;
  font-size: var(--font-size-base);
  color: var(--color-text-tertiary);
}

/* 图片容器 - 基础样式 */
.image-container {
  margin-bottom: var(--spacing-xl);
}

/* 宫格布局 - 自适应列数 */
.image-container.layout-grid {
  display: grid;
  gap: var(--gap-sm);
  grid-template-columns: repeat(auto-fill, minmax(min(100%, 160px), 1fr));
}

@media (min-width: 640px) {
  .image-container.layout-grid {
    gap: var(--gap-md);
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  }
}

@media (min-width: 900px) {
  .image-container.layout-grid {
    gap: var(--gap-lg);
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  }
}

/* 宫格布局的卡片样式 */
.layout-grid .image-card {
  background: var(--color-bg-primary);
  border-radius: var(--radius-lg);
  overflow: hidden;
  box-shadow: var(--shadow-sm);
  transition: transform var(--transition-base) cubic-bezier(0.4, 0, 0.2, 1),
  box-shadow var(--transition-base) cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  flex-direction: column;
  cursor: pointer;
}

.layout-grid .image-card:hover {
  transform: translateY(-6px);
  box-shadow: var(--shadow-xl);
}

.layout-grid .image-preview {
  width: 100%;
  aspect-ratio: 1 / 1;
  overflow: hidden;
  background: linear-gradient(135deg, var(--color-bg-gradient-start) 0%, var(--color-bg-gradient-end) 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

/* 骨架屏加载动画 */
.layout-grid .image-preview::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(
      90deg,
      transparent 0%,
      rgba(255, 255, 255, 0.6) 50%,
      transparent 100%
  );
  animation: skeleton-loading 1.5s ease-in-out infinite;
}

@keyframes skeleton-loading {
  0% {
    left: -100%;
  }
  100% {
    left: 100%;
  }
}

.layout-grid .image-preview img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  transition: transform var(--transition-base);
  position: relative;
  z-index: 1;
  /* 背景色 - 遮挡骨架屏动画 */
  background-color: var(--color-bg-primary);
}

.layout-grid .image-card:hover .image-preview img {
  transform: scale(1.05);
}

.layout-grid .image-info {
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.layout-grid .image-name {
  font-size: 13px;
  font-weight: var(--font-weight-semibold);
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.3;
}

.layout-grid .image-meta {
  font-size: 11px;
  color: var(--color-text-tertiary);
  display: flex;
  gap: 8px;
  align-items: center;
}

.layout-grid .image-meta span {
  display: flex;
  align-items: center;
  gap: 4px;
}

.layout-grid .image-date {
  font-size: 10px;
  color: var(--color-text-disabled);
}

.layout-grid .image-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 4px;
  padding-top: 10px;
  border-top: 1px solid var(--color-border-base);
}

.layout-grid .image-actions .el-button {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  padding: 6px 10px;
  height: auto;
}

/* 小屏幕优化：按钮全宽显示 */
@media (max-width: 639px) {
  .layout-grid .image-actions .el-button {
    flex: 1 1 100%;
  }
}

.loading-more,
.no-more {
  text-align: center;
  padding: 30px 0;
  font-size: var(--font-size-base);
  color: var(--color-text-tertiary);
}

.loading-more {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
}

.loading-more .is-loading {
  animation: rotating 2s linear infinite;
}

@keyframes rotating {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

.no-more {
  color: var(--color-text-disabled);
}
</style>
