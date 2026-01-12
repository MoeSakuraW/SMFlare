<script setup lang="ts">
import {computed, onMounted, onUnmounted, ref, watch} from 'vue'
import {invoke} from '@tauri-apps/api/core'
import {save} from '@tauri-apps/plugin-dialog'
import {ElMessage} from 'element-plus'
import {
  Check,
  Close,
  CopyDocument,
  Delete,
  Download,
  Edit,
  PictureFilled,
  Star,
  StarFilled
} from '@element-plus/icons-vue'
import ImageDetailDialog, {type ImageDetailData} from './ImageDetailDialog.vue'

// 定义 emit
const emit = defineEmits<{
  navigateTo: [menu: string]
  refreshGallery: []
}>()

// 定义 props
const props = defineProps<{
  d1ConfigExists: boolean
}>()

interface Picture {
  id: number
  file_hash: string
  filename: string
  store_name: string
  file_type: string
  width: number
  height: number
  size: number
  url: string
  delete_url: string
  page_url: string
  is_favorite: number
  is_deleted: number
  deleted_at: string | null
  remark: string | null
  created_at: string
  updated_at: string
}

interface SyncStats {
  added: number
  skipped: number
  deleted: number
}

interface BatchDeleteResult {
  success_count: number
  failed_count: number
  failed_items: string[]
}

interface DownloadFileInfo {
  url: string
  filename: string
}

const pictures = ref<Picture[]>([])
const loading = ref(false)
const importing = ref(false)
const selectedPictures = ref<Set<number>>(new Set())

// 删除相关状态
const deleteDialogVisible = ref(false)
const pictureToDelete = ref<Picture | null>(null)
const deleting = ref(false)

// 批量删除相关状态
const batchDeleteDialogVisible = ref(false)
const batchDeleting = ref(false)

// 下载相关状态
const downloading = ref(false)

// 备注编辑相关状态
const remarkDialogVisible = ref(false)
const remarkEditMode = ref<'single' | 'batch'>('single')
const remarkEditPicture = ref<Picture | null>(null)
const remarkInput = ref('')
const remarkSaving = ref(false)

// 计算属性：可选择的图片ID列表
const selectableIds = computed(() =>
    pictures.value.filter(p => !p.is_deleted).map(p => p.id)
)

// 计算属性：是否全选
const isAllSelected = computed(() =>
    selectableIds.value.length > 0 &&
    selectableIds.value.every(id => selectedPictures.value.has(id))
)

// 图片详情弹窗
const detailDialogVisible = ref(false)
const selectedImage = ref<Picture | null>(null)

// 筛选和排序参数
const fileType = ref('')
const isFavorite = ref<boolean | undefined>(undefined)
const includeDeleted = ref<boolean | undefined>(false)
const orderBy = ref('created_at_desc')
const filename = ref('')
const storeName = ref('')
const remark = ref('')
const currentPage = ref(1)
const pageSize = ref(10)
const total = ref(0)

// 文件类型选项
const fileTypes = ref<string[]>([])

// 转换图片数据为标准格式
const imageDetailData = computed<ImageDetailData | null>(() => {
  if (!selectedImage.value) return null
  return {
    url: selectedImage.value.url,
    pageUrl: selectedImage.value.page_url,
    filename: selectedImage.value.filename,
    storename: selectedImage.value.store_name,
    width: selectedImage.value.width,
    height: selectedImage.value.height,
    size: selectedImage.value.size,
    hash: selectedImage.value.file_hash,
    createdAt: new Date(selectedImage.value.created_at).toLocaleString('zh-CN'),
    fileType: selectedImage.value.file_type,
    remark: selectedImage.value.remark ?? undefined
  }
})

// 选择图片
const toggleSelection = (pictureId: number) => {
  if (selectedPictures.value.has(pictureId)) {
    selectedPictures.value.delete(pictureId)
  } else {
    selectedPictures.value.add(pictureId)
  }
}

// 全选/取消全选
const toggleSelectAll = () => {
  if (isAllSelected.value) {
    selectedPictures.value.clear()
  } else {
    selectableIds.value.forEach(id => selectedPictures.value.add(id))
  }
}

// 显示批量删除确认对话框
const showBatchDeleteDialog = () => {
  if (selectedPictures.value.size === 0) {
    ElMessage.warning('请先选择要删除的图片')
    return
  }
  batchDeleteDialogVisible.value = true
}

// 批量删除图片
const batchDeletePictures = async () => {
  if (selectedPictures.value.size === 0) return

  batchDeleting.value = true
  try {
    const ids = Array.from(selectedPictures.value)
    const result = await invoke<BatchDeleteResult>('batch_delete_pictures', {ids})

    if (result.success_count > 0) {
      ElMessage.success(`成功删除 ${result.success_count} 张图片`)
    }

    if (result.failed_count > 0) {
      const failedMsg = result.failed_items.slice(0, 3).join('\n')
      ElMessage.error({
        message: `删除失败 ${result.failed_count} 张图片:\n${failedMsg}${result.failed_items.length > 3 ? '\n...' : ''}`,
        duration: 5000
      })
    }

    batchDeleteDialogVisible.value = false
    selectedPictures.value.clear()
    await queryPictures()
    // 通知父组件刷新相册
    emit('refreshGallery')
  } catch (error) {
    ElMessage.error(`批量删除失败: ${error}`)
  } finally {
    batchDeleting.value = false
  }
}

// 查询图片列表
const queryPictures = async () => {
  loading.value = true
  try {
    const offset = (currentPage.value - 1) * pageSize.value

    // 并行获取图片列表和总数
    const [result, count] = await Promise.all([
      invoke<Picture[]>('query_smms_pictures', {
        params: {
          fileType: fileType.value || null,
          isFavorite: isFavorite.value,
          includeDeleted: includeDeleted.value,
          orderBy: orderBy.value,
          filename: filename.value || null,
          storeName: storeName.value || null,
          remark: remark.value || null,
          limit: pageSize.value,
          offset: offset
        }
      }),
      invoke<number>('get_pictures_count', {
        params: {
          fileType: fileType.value || null,
          isFavorite: isFavorite.value,
          includeDeleted: includeDeleted.value,
          filename: filename.value || null,
          storeName: storeName.value || null,
          remark: remark.value || null
        }
      })
    ])

    pictures.value = result
    total.value = count
  } catch (error) {
    ElMessage.error(`查询失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 导入所有相册图片
const importAllPictures = async () => {
  importing.value = true
  try {
    const stats = await invoke<SyncStats>('import_all_smms_pictures')
    const message = `同步完成：新增 ${stats.added} 张，跳过 ${stats.skipped} 张，删除 ${stats.deleted} 张`
    ElMessage.success(message)
    await loadFileTypes() // 重新加载文件类型
    await queryPictures()
    // 通知父组件刷新相册
    emit('refreshGallery')
  } catch (error) {
    ElMessage.error(`导入失败: ${error}`)
  } finally {
    importing.value = false
  }
}

// 加载所有文件类型
const loadFileTypes = async () => {
  try {
    fileTypes.value = await invoke<string[]>('get_all_file_types')
  } catch (error) {
    console.error('加载文件类型失败:', error)
  }
}

// 切换收藏状态
const toggleFavorite = async (picture: Picture) => {
  try {
    const newFavorite = picture.is_favorite === 0
    await invoke<string>('toggle_picture_favorite', {
      id: picture.id,
      isFavorite: newFavorite
    })
    picture.is_favorite = newFavorite ? 1 : 0
    ElMessage.success(newFavorite ? '已收藏' : '已取消收藏')
  } catch (error) {
    ElMessage.error(`操作失败: ${error}`)
  }
}

// 显示删除确认对话框
const showDeleteDialog = (picture: Picture) => {
  pictureToDelete.value = picture
  deleteDialogVisible.value = true
}

// 删除图片
const deletePicture = async () => {
  if (!pictureToDelete.value) return

  deleting.value = true
  try {
    const message = await invoke<string>('delete_picture', {
      id: pictureToDelete.value.id
    })
    ElMessage.success(message)
    deleteDialogVisible.value = false
    pictureToDelete.value = null
    // 重新查询图片列表
    await queryPictures()
    // 通知父组件刷新相册
    emit('refreshGallery')
  } catch (error) {
    ElMessage.error(`删除失败: ${error}`)
  } finally {
    deleting.value = false
  }
}

// 打开图片详情
const openImageDetail = (picture: Picture) => {
  selectedImage.value = picture
  detailDialogVisible.value = true
}

// 格式化文件大小（用于列表显示）
const formatSize = (bytes: number): string => {
  if (bytes < 1024) return bytes + ' B'
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(2) + ' KB'
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB'
}

// 格式化日期（用于列表显示）
const formatDate = (dateStr: string): string => {
  const d = new Date(dateStr)
  const Y = d.getFullYear()
  const M = String(d.getMonth() + 1).padStart(2, '0')
  const D = String(d.getDate()).padStart(2, '0')
  const h = String(d.getHours()).padStart(2, '0')
  const m = String(d.getMinutes()).padStart(2, '0')
  const s = String(d.getSeconds()).padStart(2, '0')
  return `${Y}-${M}-${D} ${h}:${m}:${s}`
}

// 复制链接到剪贴板（用于列表操作）
const copyLink = async (url: string) => {
  try {
    await navigator.clipboard.writeText(url)
    ElMessage.success('链接已复制')
  } catch {
    ElMessage.error('复制失败，请手动复制')
  }
}

// 下载单个文件
const downloadSingleFile = async (picture: Picture) => {
  try {
    // 打开保存对话框
    const savePath = await save({
      defaultPath: picture.filename,
      filters: [{
        name: '图片文件',
        extensions: [picture.file_type]
      }]
    })

    if (!savePath) {
      return // 用户取消了保存
    }

    downloading.value = true
    const message = await invoke<string>('download_single_file', {
      url: picture.url,
      savePath: savePath
    })
    ElMessage.success(message)
  } catch (error) {
    ElMessage.error(`下载失败: ${error}`)
  } finally {
    downloading.value = false
  }
}

// 批量下载文件
const downloadMultipleFiles = async () => {
  if (selectedPictures.value.size === 0) {
    ElMessage.warning('请先选择要下载的图片')
    return
  }

  try {
    // 打开保存对话框
    const savePath = await save({
      defaultPath: `images_${Date.now()}.zip`,
      filters: [{
        name: 'ZIP 压缩包',
        extensions: ['zip']
      }]
    })

    if (!savePath) {
      return // 用户取消了保存
    }

    downloading.value = true

    // 准备下载文件信息
    const files: DownloadFileInfo[] = Array.from(selectedPictures.value)
        .map(id => pictures.value.find(p => p.id === id))
        .filter((p): p is Picture => p !== undefined)
        .map(p => ({
          url: p.url,
          filename: p.filename
        }))

    const message = await invoke<string>('download_files_as_zip', {
      files: files,
      savePath: savePath
    })

    ElMessage.success(message)
    selectedPictures.value.clear()
  } catch (error) {
    ElMessage.error(`批量下载失败: ${error}`)
  } finally {
    downloading.value = false
  }
}

// 显示单个图片备注编辑对话框
const showRemarkDialog = (picture: Picture) => {
  remarkEditMode.value = 'single'
  remarkEditPicture.value = picture
  remarkInput.value = picture.remark || ''
  remarkDialogVisible.value = true
}

// 显示批量备注编辑对话框
const showBatchRemarkDialog = () => {
  if (selectedPictures.value.size === 0) {
    ElMessage.warning('请先选择要编辑备注的图片')
    return
  }
  remarkEditMode.value = 'batch'
  remarkEditPicture.value = null
  remarkInput.value = ''
  remarkDialogVisible.value = true
}

// 保存备注
const saveRemark = async () => {
  remarkSaving.value = true
  try {
    const remarkValue = remarkInput.value.trim() || null

    if (remarkEditMode.value === 'single' && remarkEditPicture.value) {
      // 单个图片备注更新
      await invoke<string>('update_picture_remark', {
        id: remarkEditPicture.value.id,
        remark: remarkValue
      })
      remarkEditPicture.value.remark = remarkValue
      ElMessage.success('备注更新成功')
    } else if (remarkEditMode.value === 'batch') {
      // 批量备注更新
      const ids = Array.from(selectedPictures.value)
      const message = await invoke<string>('batch_update_picture_remark', {
        ids,
        remark: remarkValue
      })
      ElMessage.success(message)
      await queryPictures()
      selectedPictures.value.clear()
    }
    remarkDialogVisible.value = false
  } catch (error) {
    ElMessage.error(`保存失败: ${error}`)
  } finally {
    remarkSaving.value = false
  }
}

// 页码改变
const handlePageChange = (page: number) => {
  currentPage.value = page
  selectedPictures.value.clear()
  queryPictures()
}

// 筛选改变
const handleFilterChange = () => {
  currentPage.value = 1
  selectedPictures.value.clear()
  queryPictures()
}

// 搜索防抖处理
let searchTimer: ReturnType<typeof setTimeout> | null = null
const handleSearchInput = () => {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    handleFilterChange()
  }, 300)
}

// 对话框关闭时清空缓存数据
watch(remarkDialogVisible, (newVal) => {
  if (!newVal) {
    remarkInput.value = ''
    remarkEditPicture.value = null
    remarkEditMode.value = 'single'
  }
})

onMounted(() => {
  if (props.d1ConfigExists) {
    loadFileTypes()
    queryPictures()
  }
})

onUnmounted(() => {
  if (searchTimer) clearTimeout(searchTimer)
})
</script>

<template>
  <div class="picture-manager" :class="{ 'has-floating-toolbar': selectedPictures.size > 0 }">
    <!-- 统一的section-header样式 -->
    <div class="section-header">
      <h2>文件管理</h2>
      <div class="header-actions">
        <el-button class="btn-primary-action" :loading="importing" @click="importAllPictures">
          同步相册图片
        </el-button>
      </div>
    </div>

    <!-- 主内容卡片 -->
    <div class="manager-card">
      <!-- 筛选器 -->
      <div class="filters">
        <el-input
            v-model="filename"
            placeholder="搜索文件名"
            clearable
            @input="handleSearchInput"
            style="width: 200px"
        />

        <el-input
            v-model="storeName"
            placeholder="搜索存储名"
            clearable
            @input="handleSearchInput"
            style="width: 200px"
        />

        <el-input
            v-model="remark"
            placeholder="搜索备注"
            clearable
            @input="handleSearchInput"
            style="width: 200px"
        />

        <el-select
            v-model="fileType"
            placeholder="文件类型"
            clearable
            @change="handleFilterChange"
            style="width: 150px"
        >
          <el-option
              v-for="type in fileTypes"
              :key="type"
              :label="type.toUpperCase()"
              :value="type"
          />
        </el-select>

        <el-select
            v-model="isFavorite"
            placeholder="收藏状态"
            clearable
            @change="handleFilterChange"
            style="width: 150px"
        >
          <el-option label="已收藏" :value="true"/>
          <el-option label="未收藏" :value="false"/>
        </el-select>

        <el-select
            v-model="includeDeleted"
            placeholder="删除状态"
            @change="handleFilterChange"
            style="width: 150px"
        >
          <el-option label="仅未删除" :value="false"/>
          <el-option label="全部显示" :value="undefined"/>
          <el-option label="仅已删除" :value="true"/>
        </el-select>

        <el-select
            v-model="orderBy"
            @change="handleFilterChange"
            style="width: 180px"
        >
          <el-option label="创建时间 ↓" value="created_at_desc"/>
          <el-option label="创建时间 ↑" value="created_at_asc"/>
          <el-option label="更新时间 ↓" value="updated_at_desc"/>
          <el-option label="更新时间 ↑" value="updated_at_asc"/>
          <el-option label="文件大小 ↓" value="size_desc"/>
          <el-option label="文件大小 ↑" value="size_asc"/>
        </el-select>
      </div>

      <!-- 文件列表 -->
      <div v-loading="loading" class="pictures-list">
        <!-- 空状态 -->
        <div v-if="!loading && pictures.length === 0" class="empty-state">
          <!-- 未配置数据库 -->
          <template v-if="!props.d1ConfigExists">
            <el-icon :size="64" color="#c0c4cc">
              <PictureFilled/>
            </el-icon>
            <p>请先配置数据库</p>
            <p style="margin-top: 10px; font-size: var(--font-size-base); color: var(--color-text-tertiary);">
              前往
              <el-button text class="btn-guide" @click="emit('navigateTo', 'database-settings')">
                数据库设置
              </el-button>
              配置 Cloudflare D1
            </p>
          </template>
          <!-- 已配置但无数据 -->
          <template v-else>
            <el-icon :size="64" color="#c0c4cc">
              <PictureFilled/>
            </el-icon>
            <p>暂无图片</p>
            <p style="margin-top: 10px; font-size: var(--font-size-base); color: var(--color-text-tertiary);">
              点击"同步相册图片"按钮导入 SM.MS 上传历史
            </p>
          </template>
        </div>

        <!-- 列表项 -->
        <div
            v-for="picture in pictures"
            :key="picture.id"
            :class="['list-item', { selected: selectedPictures.has(picture.id), deleted: picture.is_deleted }]"
            @click="toggleSelection(picture.id)"
        >
          <!-- 选择框 -->
          <div class="item-checkbox">
            <div :class="['checkbox', { checked: selectedPictures.has(picture.id) }]">
              <el-icon v-if="selectedPictures.has(picture.id)" :size="14" color="#fff">
                <Check/>
              </el-icon>
            </div>
          </div>

          <!-- 缩略图 -->
          <div class="item-thumbnail" @click.stop="openImageDetail(picture)">
            <img
                :src="picture.url"
                :alt="picture.filename"
                loading="lazy"
                @error="($event.target as HTMLImageElement).classList.add('error')"
            />
            <div class="thumbnail-error">
              <el-icon :size="20" color="#c0c4cc">
                <PictureFilled/>
              </el-icon>
            </div>
          </div>

          <!-- 文件名 -->
          <div class="item-name-wrapper">
            <div class="item-name" :title="picture.filename">
              {{ picture.filename }}
              <el-tag v-if="picture.is_deleted" type="danger" size="small" style="margin-left: 8px;">已删除</el-tag>
            </div>
            <div v-if="picture.remark" class="item-remark" :title="picture.remark">
              {{ picture.remark }}
            </div>
          </div>

          <!-- 文件类型 -->
          <div class="item-type">
            {{ picture.file_type.toUpperCase() }}
          </div>

          <!-- 尺寸 -->
          <div class="item-dimensions">
            {{ picture.width }} × {{ picture.height }}
          </div>

          <!-- 文件大小 -->
          <div class="item-size">
            {{ formatSize(picture.size) }}
          </div>

          <!-- 修改日期 -->
          <div class="item-date">
            {{ formatDate(picture.created_at) }}
          </div>

          <!-- 操作按钮 -->
          <div v-if="!picture.is_deleted" class="item-actions">
            <el-button
                :icon="CopyDocument"
                class="btn-content"
                circle
                title="复制链接"
                @click.stop="copyLink(picture.url)"
            />
            <el-button
                :icon="picture.is_favorite ? StarFilled : Star"
                :class="['btn-state', { 'is-active': picture.is_favorite }]"
                circle
                title="收藏"
                @click.stop="toggleFavorite(picture)"
            />
            <el-button
                :icon="Edit"
                class="btn-content"
                circle
                title="编辑备注"
                @click.stop="showRemarkDialog(picture)"
            />
            <el-button
                :icon="Download"
                class="btn-content"
                circle
                title="下载"
                :loading="downloading"
                @click.stop="downloadSingleFile(picture)"
            />
            <el-button
                :icon="Delete"
                class="btn-danger"
                circle
                title="删除"
                @click.stop="showDeleteDialog(picture)"
            />
          </div>
        </div>
      </div>

      <!-- 分页 -->
      <div class="pagination">
        <el-pagination
            v-model:current-page="currentPage"
            :page-size="pageSize"
            :total="total"
            layout="prev, pager, next, total"
            @current-change="handlePageChange"
        />
      </div>
    </div>

    <!-- 删除确认对话框 -->
    <el-dialog
        v-model="deleteDialogVisible"
        title="确认删除"
        width="400px"
        :close-on-click-modal="false"
    >
      <div style="padding: 20px 0;">
        <p style="margin-bottom: 10px; color: var(--color-text-primary);">
          确定要删除图片吗？
        </p>
        <p style="color: var(--color-text-secondary); font-size: var(--font-size-small);">
          <strong>{{ pictureToDelete?.filename }}</strong>
        </p>
        <p style="margin-top: 10px; color: var(--color-danger); font-size: var(--font-size-small);">
          此操作同时删除 SM.MS 服务器上的图片和数据库记录，且无法恢复
        </p>
      </div>
      <template #footer>
        <el-button @click="deleteDialogVisible = false" :disabled="deleting">
          取消
        </el-button>
        <el-button type="danger" @click="deletePicture" :loading="deleting">
          确认删除
        </el-button>
      </template>
    </el-dialog>

    <!-- 批量删除确认对话框 -->
    <el-dialog
        v-model="batchDeleteDialogVisible"
        title="确认批量删除"
        width="400px"
        :close-on-click-modal="false"
    >
      <div style="padding: 20px 0;">
        <p style="margin-bottom: 10px; color: var(--color-text-primary);">
          确定要删除选中的 <strong>{{ selectedPictures.size }}</strong> 张图片吗？
        </p>
        <p style="margin-top: 10px; color: var(--color-danger); font-size: var(--font-size-small);">
          此操作同时删除 SM.MS 服务器上的图片和数据库记录，且无法恢复
        </p>
      </div>
      <template #footer>
        <el-button @click="batchDeleteDialogVisible = false" :disabled="batchDeleting">
          取消
        </el-button>
        <el-button type="danger" @click="batchDeletePictures" :loading="batchDeleting">
          确认删除
        </el-button>
      </template>
    </el-dialog>

    <!-- 图片详情弹窗 -->
    <ImageDetailDialog
        v-model="detailDialogVisible"
        :image="imageDetailData"
    />

    <!-- 备注编辑对话框 -->
    <el-dialog
        v-model="remarkDialogVisible"
        :title="remarkEditMode === 'single' ? '编辑备注' : '批量编辑备注'"
        width="500px"
        :close-on-click-modal="false"
    >
      <div style="padding: 20px 0;">
        <p v-if="remarkEditMode === 'single' && remarkEditPicture"
           style="margin-bottom: 15px; color: var(--color-text-secondary); font-size: var(--font-size-small);">
          文件名：<strong>{{ remarkEditPicture.filename }}</strong>
        </p>
        <p v-else-if="remarkEditMode === 'batch'"
           style="margin-bottom: 15px; color: var(--color-text-secondary); font-size: var(--font-size-small);">
          将为选中的 <strong>{{ selectedPictures.size }}</strong> 张图片设置相同的备注
        </p>
        <el-input
            v-model="remarkInput"
            type="textarea"
            :rows="4"
            placeholder="请输入备注内容（可选）"
            maxlength="500"
            show-word-limit
        />
      </div>
      <template #footer>
        <el-button @click="remarkDialogVisible = false" :disabled="remarkSaving">
          取消
        </el-button>
        <el-button type="primary" @click="saveRemark" :loading="remarkSaving">
          保存
        </el-button>
      </template>
    </el-dialog>

    <!-- 底部浮动批量操作栏 -->
    <Transition name="slide-up">
      <div v-if="selectedPictures.size > 0" class="floating-batch-actions">
        <div class="batch-actions-content">
          <div class="batch-info">
            <span class="batch-count">已选择 {{ selectedPictures.size }} 项</span>
            <el-button
                :icon="Close"
                text
                size="small"
                class="cancel-btn"
                @click="selectedPictures.clear()"
                title="取消选择"
            />
          </div>
          <div class="batch-buttons">
            <el-button size="default" @click="toggleSelectAll">
              {{ isAllSelected ? '取消全选' : '全选' }}
            </el-button>
            <el-button type="primary" size="default" :loading="downloading" @click="downloadMultipleFiles">
              批量下载
            </el-button>
            <el-button size="default" @click="showBatchRemarkDialog">
              批量编辑备注
            </el-button>
            <el-button type="danger" size="default" @click="showBatchDeleteDialog">
              批量删除
            </el-button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
/* 统一布局样式 - 与上传区、相册保持一致 */
.picture-manager {
  max-width: var(--container-md);
  margin: 0 auto;
  transition: padding-bottom 0.3s ease;
}

/* 当显示浮动工具栏时，添加底部内边距避免遮挡 */
.picture-manager.has-floating-toolbar {
  padding-bottom: 90px;
}

/* 统一的section-header样式 */
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

.header-actions {
  display: flex;
  gap: var(--gap-sm);
  align-items: center;
}

/* 主内容卡片 - 扁平化设计 */
.manager-card {
  background: var(--color-bg-primary);
  padding: 30px;
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  transition: box-shadow var(--transition-base);
}

.manager-card:hover {
  box-shadow: var(--shadow-lg);
}

.filters {
  display: flex;
  gap: var(--gap-md);
  margin-bottom: var(--spacing-lg);
  flex-wrap: wrap;
}

/* 文件列表布局 */
.pictures-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin-bottom: var(--spacing-xl);
  background: var(--color-bg-secondary);
  border-radius: var(--radius-lg);
  overflow: hidden;
  min-height: 200px;
}

/* 列表项样式 */
.list-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
  background: var(--color-bg-primary);
  cursor: pointer;
  transition: background 0.2s ease;
}

.list-item:hover {
  background: var(--color-bg-hover);
}

.list-item.selected {
  background: var(--color-bg-selected);
}

/* 已删除图片样式 */
.list-item.deleted {
  opacity: 0.6;
  background: rgba(245, 108, 108, 0.08);
}

.list-item.deleted .item-name {
  text-decoration: line-through;
  color: var(--color-text-tertiary);
}

.list-item.deleted:hover {
  background: rgba(245, 108, 108, 0.15);
}

/* 选择框 */
.item-checkbox {
  flex-shrink: 0;
}

.checkbox {
  width: 18px;
  height: 18px;
  border: 2px solid var(--color-border-base);
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.checkbox.checked {
  background: var(--color-primary);
  border-color: var(--color-primary);
}

/* 缩略图 */
.item-thumbnail {
  width: 48px;
  height: 48px;
  flex-shrink: 0;
  border-radius: var(--radius-md);
  overflow: hidden;
  background: var(--color-bg-secondary);
  position: relative;
  cursor: pointer;
  transition: transform 0.2s ease, box-shadow 0.2s ease, background 0.2s ease;
}

.item-thumbnail:hover {
  transform: scale(1.05);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

/* 列表项 hover 时，缩略图背景变为透明，与列表背景融合 */
.list-item:hover .item-thumbnail {
  background: transparent;
}

/* 选中状态下的缩略图背景 */
.list-item.selected .item-thumbnail {
  background: transparent;
}

.item-thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.item-thumbnail img.error {
  opacity: 0;
}

.thumbnail-error {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: none;
  align-items: center;
  justify-content: center;
  background: var(--color-bg-secondary);
  transition: background 0.2s ease;
}

.item-thumbnail img.error + .thumbnail-error {
  display: flex;
}

/* 列表项 hover 时，错误状态背景也变为透明 */
.list-item:hover .thumbnail-error {
  background: transparent;
}

/* 选中状态下的错误背景 */
.list-item.selected .thumbnail-error {
  background: transparent;
}

/* 文件名列容器 */
.item-name-wrapper {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

/* 文件名 */
.item-name {
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.list-item:hover .item-name {
  color: var(--color-primary);
}

/* 备注文本 */
.item-remark {
  font-size: 12px;
  color: var(--color-text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-style: italic;
}

/* 列表列公共样式 */
.item-type,
.item-dimensions,
.item-size,
.item-date {
  flex-shrink: 0;
  font-size: 12px;
  color: var(--color-text-tertiary);
}

/* 文件类型列 */
.item-type {
  width: 50px;
  text-align: center;
}

/* 尺寸列 */
.item-dimensions {
  width: 80px;
  text-align: center;
}

/* 文件大小列 */
.item-size {
  width: 70px;
  text-align: right;
}

/* 日期列 */
.item-date {
  width: 135px;
  text-align: center;
}

/* 操作列 - 冻结在右侧 */
.item-actions {
  flex-shrink: 0;
  display: flex;
  gap: 0;
  justify-content: center;
  position: sticky;
  right: 0;
  background: var(--color-bg-primary);
  padding-left: 12px;
  margin-left: auto;
  transition: background 0.2s ease;
}

/* 操作按钮紧凑样式 */
.item-actions .el-button {
  width: 28px;
  height: 28px;
  padding: 0;
  margin: 0 1px;
}

/* 列表项 hover 时，操作区背景跟随变化 */
.list-item:hover .item-actions {
  background: var(--color-bg-hover);
}

/* 选中状态下的操作区背景 */
.list-item.selected .item-actions {
  background: var(--color-bg-selected);
}

/* 已删除图片的操作区背景 */
.list-item.deleted .item-actions {
  background: var(--color-danger-lighter);
}

.list-item.deleted:hover .item-actions {
  background: var(--color-danger-light);
}

/* 列表项 hover 时，操作按钮的特殊样式 */
/* 复制按钮 - 使用更深的背景色 */
.list-item:hover .item-actions .el-button.btn-content:hover {
  background-color: var(--color-overlay-light);
}

/* 收藏按钮 - 使用更深的背景色 */
.list-item:hover .item-actions .el-button.btn-state:hover {
  background-color: var(--color-overlay-light);
}

/* 删除按钮 - 使用更深的红色背景 */
.list-item:hover .item-actions .el-button.btn-danger:hover {
  background-color: var(--color-danger-light);
}

.pagination {
  display: flex;
  justify-content: center;
  margin-top: 20px;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--color-text-tertiary);
  background: var(--color-bg-primary);
}

.empty-state p {
  margin-top: 16px;
  font-size: var(--font-size-base);
}

/* 响应式：小屏幕隐藏部分列 */
@media (max-width: 768px) {
  .item-type,
  .item-dimensions {
    display: none;
  }

  .item-date {
    width: 120px;
  }
}

@media (max-width: 480px) {
  .item-date {
    display: none;
  }

  .list-item {
    gap: var(--gap-md);
    padding: 10px 12px;
  }
}

/* 底部浮动批量操作栏 */
.floating-batch-actions {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 1000;
  padding: 16px 24px;
  background: var(--color-bg-primary);
  box-shadow: var(--shadow-lg);
  border-top: 1px solid var(--color-border-base);
}

.batch-actions-content {
  max-width: var(--container-md);
  margin: 0 auto;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--gap-lg);
}

.batch-info {
  display: flex;
  align-items: center;
  gap: var(--gap-md);
}

.batch-count {
  font-size: var(--font-size-base);
  color: var(--color-text-primary);
  font-weight: var(--font-weight-medium);
}

.cancel-btn {
  transition: all 0.2s ease;
  opacity: 0.7;
}

.cancel-btn:hover {
  opacity: 1;
  transform: scale(1.1);
}

.batch-buttons {
  display: flex;
  gap: var(--gap-sm);
  flex-wrap: wrap;
}

/* 滑入动画 */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1),
  opacity 0.3s ease;
}

.slide-up-enter-from {
  transform: translateY(100%);
  opacity: 0;
}

.slide-up-leave-to {
  transform: translateY(100%);
  opacity: 0;
}

.slide-up-enter-to,
.slide-up-leave-from {
  transform: translateY(0);
  opacity: 1;
}

/* 移动端响应式适配 */
@media (max-width: 768px) {
  .floating-batch-actions {
    padding: 12px 16px;
  }

  .batch-actions-content {
    flex-direction: column;
    gap: var(--gap-md);
    align-items: stretch;
  }

  .batch-info {
    justify-content: space-between;
  }

  .batch-buttons {
    justify-content: center;
  }
}

@media (max-width: 480px) {
  .floating-batch-actions {
    padding: 10px 12px;
  }

  .batch-buttons {
    flex-direction: column;
    gap: 8px;
  }

  .batch-buttons .el-button {
    width: 100%;
  }
}
</style>
