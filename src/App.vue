<script setup lang="ts">
import {type Component, computed, onMounted, onUnmounted, ref, watch} from 'vue'
import {Coin, CopyDocument, Edit, FolderOpened, Picture, Setting, Upload,} from '@element-plus/icons-vue'
import {invoke} from '@tauri-apps/api/core'
import {getCurrentWindow} from '@tauri-apps/api/window'
import {ElMessage} from 'element-plus'
import PictureManager from './components/PictureManager.vue'
import Gallery from './components/Gallery.vue'
import {ImageDetailData} from "./components/ImageDetailDialog.vue";

// 开发环境日志工具
const isDev = import.meta.env.DEV
const log = (...args: any[]) => isDev && console.log(...args)
const logError = (...args: any[]) => isDev && console.error(...args)

// 支持的图片格式
const IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp']

interface MenuItem {
  name: string
  icon: Component
  path: string
}

interface D1ConfigType {
  account_id: string
  database_id: string
  api_token: string
}

interface SmmsUserType {
  username: string
  password: string
  token: string
}

interface SmmsUploadItem {
  file_id: number
  width: number
  height: number
  filename: string
  storename: string
  size: number
  path: string
  hash: string
  created_at: string  // 字符串格式 "2022-01-20 20:07:16"
  url: string
  delete: string
  page: string
}

interface UploadResult {
  filename: string
  success: boolean
  message: string
  url?: string
  remark?: string
}

const activeMenu = ref('upload')
const uploading = ref(false)
const uploadResults = ref<UploadResult[]>([])
const isDragging = ref(false)
const remark = ref<string>('')
let unlistenDrop: (() => void) | null = null

// SM.MS 图床配置
const smmsForm = ref({
  username: '',
  password: '',
  token: ''
})
const smmsLoading = ref(false)

// SM.MS 配置加载的并发控制
let loadingPromise: Promise<void> | null = null

// 上传历史
const uploadHistory = ref<SmmsUploadItem[]>([])
const currentPage = ref(1)
const historyLoading = ref(false)
const hasMore = ref(true)
const isLoadingMore = ref(false)

// 配置状态
const d1ConfigExists = ref(false)
const smmsConfigExists = ref(false)

// 图片详情弹窗
const detailDialogVisible = ref(false)
const selectedImage = ref<SmmsUploadItem | null>(null)

// 转换图片数据为标准格式
const imageDetailData = computed<ImageDetailData | null>(() => {
  if (!selectedImage.value) return null
  return {
    url: selectedImage.value.url,
    pageUrl: selectedImage.value.page,
    filename: selectedImage.value.filename,
    storename: selectedImage.value.storename,
    width: selectedImage.value.width,
    height: selectedImage.value.height,
    size: selectedImage.value.size,
    hash: selectedImage.value.hash,
    createdAt: selectedImage.value.created_at
  }
})

// 上传结果逆序显示（最新的在前）
const reversedUploadResults = computed(() => {
  return [...uploadResults.value].reverse()
})

// 打开图片详情
const openImageDetail = (item: SmmsUploadItem) => {
  selectedImage.value = item
  detailDialogVisible.value = true
}

// Cloudflare D1 数据库配置
const d1Form = ref({
  account_id: '',
  database_id: '',
  api_token: ''
})
const d1TestLoading = ref(false)

const menuItems: MenuItem[] = [
  {name: '上传区', icon: Upload, path: 'upload'},
  {name: '相册', icon: Picture, path: 'gallery'},
  {name: '图床设置', icon: Setting, path: 'image-bed-settings'},
  {name: '数据库设置', icon: Coin, path: 'database-settings'},
  {name: '文件管理', icon: FolderOpened, path: 'files'}
]

const handleMenuClick = (path: string) => activeMenu.value = path

// 处理子组件的导航请求
const handleNavigate = (menu: string) => activeMenu.value = menu

// 上传图片（通用函数，支持选择和拖拽）
const uploadImages = async (filePaths: string[]) => {
  if (filePaths.length === 0) return

  uploading.value = true
  try {
    const results = await invoke<UploadResult[]>('upload_images', {
      filePaths,
      remark: remark.value || null
    })
    uploadResults.value.push(...results)

    const successCount = results.filter(r => r.success).length
    const failCount = results.length - successCount

    if (failCount === 0) {
      ElMessage.success(`成功上传 ${successCount} 张图片`)
    } else {
      ElMessage.warning(`成功 ${successCount} 张，失败 ${failCount} 张`)
    }
  } catch (error) {
    ElMessage.error(`上传失败: ${error}`)
  } finally {
    uploading.value = false
  }
}

// 使用 dialog 插件选择并上传图片
const selectAndUploadImages = async () => {
  try {
    const {open} = await import('@tauri-apps/plugin-dialog')

    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Images',
        extensions: IMAGE_EXTENSIONS
      }]
    })

    if (!selected) return

    const filePaths = Array.isArray(selected) ? selected : [selected]
    await uploadImages(filePaths)
  } catch (error) {
    ElMessage.error(`选择文件失败: ${error}`)
  }
}

// 获取 SM.MS Token
const getSmmsToken = async () => {
  if (!smmsForm.value.username || !smmsForm.value.password) {
    ElMessage.warning('请输入用户名和密码')
    return
  }

  smmsLoading.value = true
  try {
    const token = await invoke<string>('get_smms_token', {
      username: smmsForm.value.username,
      password: smmsForm.value.password
    })
    smmsForm.value.token = token
    ElMessage.success('API Token 获取成功')

    // 登录成功后，保存凭证到 D1 数据库
    try {
      await invoke<string>('save_smms_user', {
        username: smmsForm.value.username,
        password: smmsForm.value.password,
        token: token
      })

      // 清除缓存标记，确保数据一致性
      sessionStorage.removeItem('smms_loaded')
      sessionStorage.removeItem('smms_user_not_found')
      log('已清除 SM.MS 配置缓存标记')

      // 保存成功后，标记配置已加载（当前表单已有最新数据）
      sessionStorage.setItem('smms_loaded', 'true')
      smmsConfigExists.value = true
      log('登录成功，配置已更新')
    } catch (saveError) {
      logError('保存凭证失败:', saveError)
      // 静默失败，不影响用户体验
    }
  } catch (error) {
    ElMessage.error(`获取失败: ${error}`)
  } finally {
    smmsLoading.value = false
  }
}

// 加载 D1 配置（静默加载，不显示提示）
const loadD1ConfigSilent = async () => {
  log('=== loadD1ConfigSilent 开始 ===')
  try {
    const config = await invoke<D1ConfigType>('load_d1_config')
    log('D1 配置加载成功:', config)
    d1Form.value = config
    d1ConfigExists.value = true
  } catch (error) {
    logError('D1 配置加载失败:', error)
    d1ConfigExists.value = false
  }
}

// 加载 SM.MS 凭证（静默加载，不显示提示）
const loadSmmsUserSilent = async () => {
  log('=== loadSmmsUserSilent 开始 ===')

  // 如果正在加载，返回现有Promise，避免重复请求
  if (loadingPromise) {
    log('已有加载请求进行中，等待完成')
    return loadingPromise
  }

  // 检查是否已加载到内存
  const loadedFlag = sessionStorage.getItem('smms_loaded')
  if (loadedFlag && smmsForm.value.token) {
    log('从内存使用已加载的 SM.MS 凭证')
    return
  }

  // 检查是否已知用户不存在（空值缓存，防止缓存穿透）
  const notFoundKey = 'smms_user_not_found'
  const notFound = sessionStorage.getItem(notFoundKey)
  if (notFound) {
    const timestamp = parseInt(notFound)
    // 5分钟内不重复查询
    if (Date.now() - timestamp < 5 * 60 * 1000) {
      log('用户不存在（已缓存），跳过查询')
      return
    }
  }

  // 创建新的加载Promise
  loadingPromise = (async () => {
    try {
      // 如果表单中有用户名，使用该用户名查询；否则查询 id=1 的默认用户
      const user = await invoke<SmmsUserType>('load_smms_user', {
        username: smmsForm.value.username || null
      })
      log('从 D1 加载 SM.MS 凭证成功:', user)

      // 更新表单
      smmsForm.value.username = user.username
      smmsForm.value.password = user.password
      smmsForm.value.token = user.token

      // 标记已加载，清除"不存在"标记
      sessionStorage.setItem('smms_loaded', 'true')
      sessionStorage.removeItem(notFoundKey)
      log('SM.MS 凭证已加载到内存')
      smmsConfigExists.value = true
    } catch (error) {
      logError('SM.MS 凭证加载失败:', error)
      // 缓存"不存在"状态，防止重复查询
      sessionStorage.setItem(notFoundKey, Date.now().toString())
      smmsConfigExists.value = false
    } finally {
      // 请求完成，清除Promise缓存
      loadingPromise = null
    }
  })()

  return loadingPromise
}

// 测试 D1 连接
const testD1Connection = async () => {
  if (!d1Form.value.account_id || !d1Form.value.database_id || !d1Form.value.api_token) {
    ElMessage.warning('请填写完整的配置信息')
    return
  }

  d1TestLoading.value = true
  try {
    const result = await invoke<string>('test_d1_connection', {
      config: d1Form.value
    })

    // 测试成功后自动保存数据库配置
    try {
      await invoke<string>('save_d1_config', {
        config: d1Form.value
      })

      // 更新 D1 配置状态
      d1ConfigExists.value = true

      // 数据库连接成功后，自动从数据库加载图床配置并填充到表单
      try {
        // 默认查询 id=1 的用户（首次配置时的默认用户）
        const user = await invoke<SmmsUserType>('load_smms_user', {
          username: null
        })
        log('数据库连接成功，自动加载图床配置:', user)

        // 清除缓存标记，确保数据一致性
        sessionStorage.removeItem('smms_loaded')
        sessionStorage.removeItem('smms_user_not_found')

        smmsForm.value.username = user.username
        smmsForm.value.password = user.password
        smmsForm.value.token = user.token

        // 标记配置已加载
        sessionStorage.setItem('smms_loaded', 'true')
        smmsConfigExists.value = true

        ElMessage.success('连接测试成功，数据库配置已保存，图床配置已自动加载')
      } catch (loadError) {
        logError('加载图床配置失败:', loadError)
        ElMessage.success('连接测试成功，数据库配置已保存（暂无图床配置）')
      }
    } catch (saveError) {
      // 测试成功但保存失败
      ElMessage.warning(`${result}，但配置保存失败: ${saveError}`)
    }
  } catch (error) {
    ElMessage.error(`测试失败: ${error}`)
  } finally {
    d1TestLoading.value = false
  }
}

// 获取上传历史
const fetchUploadHistory = async (page: number = 1, append: boolean = false) => {
  // 如果正在加载或没有更多数据，则返回
  if (isLoadingMore.value || (!append && historyLoading.value)) return
  if (append && !hasMore.value) return

  if (append) {
    isLoadingMore.value = true
  } else {
    historyLoading.value = true
  }

  try {
    const data = await invoke<SmmsUploadItem[]>('get_smms_upload_history', {page})

    if (append) {
      // 追加模式：添加到现有列表
      uploadHistory.value = [...uploadHistory.value, ...data]
    } else {
      // 替换模式：清空列表重新加载
      uploadHistory.value = data
    }

    currentPage.value = page

    // 如果返回的数据少于预期（通常一页10条），说明没有更多数据了
    hasMore.value = data.length >= 10
  } catch (error) {
    const errorMsg = String(error)
    // 如果是未登录错误，静默处理，让页面显示友好提示
    if (errorMsg.includes('未找到 SM.MS 凭证') || errorMsg.includes('请先登录')) {
      uploadHistory.value = []
      hasMore.value = false
      log('用户未登录 SM.MS，显示空状态')
    } else {
      ElMessage.error(`获取上传历史失败: ${error}`)
    }
  } finally {
    if (append) {
      isLoadingMore.value = false
    } else {
      historyLoading.value = false
    }
  }
}

// 复制链接到剪贴板
const copyLink = async (url: string) => {
  try {
    await navigator.clipboard.writeText(url)
    ElMessage.success('链接已复制')
  } catch {
    ElMessage.error('复制失败，请手动复制')
  }
}

// 清除备注
const clearRemark = () => {
  remark.value = ''
}

// 清除上传结果
const clearUploadResults = () => {
  uploadResults.value = []
}

// 刷新相册
const refreshGallery = () => {
  fetchUploadHistory(1, false)
}

// 清空相册缓存（文件管理操作后调用）
const clearGalleryCache = () => {
  uploadHistory.value = []
  currentPage.value = 1
  hasMore.value = true
}

// 滚动加载更多
const handleScroll = (e: Event) => {
  if (activeMenu.value !== 'gallery') return

  const target = e.target as HTMLElement
  const scrollTop = target.scrollTop
  const scrollHeight = target.scrollHeight
  const clientHeight = target.clientHeight

  // 当滚动到距离底部100px时，加载下一页
  if (scrollHeight - scrollTop - clientHeight < 100) {
    if (hasMore.value && !isLoadingMore.value && uploadHistory.value.length > 0) {
      fetchUploadHistory(currentPage.value + 1, true)
    }
  }
}

// 页面加载时自动加载 D1 配置和 SM.MS 凭证（静默加载）
onMounted(async () => {
  // 清除旧的 localStorage 缓存（已迁移到 sessionStorage）
  localStorage.removeItem('smms_user_cache')

  loadD1ConfigSilent()
  loadSmmsUserSilent()

  // 监听文件拖放事件
  try {
    const window = getCurrentWindow()
    log('开始监听拖放事件')

    unlistenDrop = await window.onDragDropEvent((event) => {
      log('拖放事件:', event.payload.type)

      switch (event.payload.type) {
        case 'drop': {
          isDragging.value = false
          const paths = event.payload.paths as string[]
          const imageFiles = paths.filter(path => {
            const ext = path.split('.').pop()?.toLowerCase()
            return ext && IMAGE_EXTENSIONS.includes(ext)
          })

          log('拖放的图片文件:', imageFiles)

          if (imageFiles.length === 0) {
            ElMessage.warning('请拖拽图片文件')
            return
          }
          if (activeMenu.value === 'upload') {
            uploadImages(imageFiles)
          }
          break
        }
        case 'enter':
        case 'over':
          isDragging.value = true
          break

        case 'leave':
          isDragging.value = false
          break
      }
    })

    log('拖放事件监听已设置')
  } catch (error) {
    logError('设置拖放监听失败:', error)
  }
})

onUnmounted(() => {
  if (unlistenDrop) unlistenDrop()
})

// 监听菜单切换，当切换到相关设置页面时重新加载配置
watch(activeMenu, (newMenu) => {
  if (newMenu === 'image-bed-settings') {
    log('切换到图床设置页面，重新加载配置')
    loadSmmsUserSilent()
  } else if (newMenu === 'database-settings') {
    log('切换到数据库设置页面，重新加载配置')
    loadD1ConfigSilent()
  } else if (newMenu === 'gallery') {
    log('切换到相册页面')
    // 检查配置是否存在
    if (!d1ConfigExists.value) {
      log('D1 配置不存在，不加载相册数据')
      uploadHistory.value = []
      return
    }
    if (!smmsConfigExists.value) {
      log('SM.MS 配置不存在，不加载相册数据')
      uploadHistory.value = []
      return
    }
    // 内存缓存优化：如果已有数据，直接显示，避免重复加载
    if (uploadHistory.value.length === 0) {
      log('首次加载相册数据')
      currentPage.value = 1
      hasMore.value = true
      fetchUploadHistory(1, false)
    } else {
      log('使用缓存的相册数据，共', uploadHistory.value.length, '张图片')
    }
  }
})
</script>

<template>
  <div class="app-container">
    <!-- 左侧导航栏 -->
    <aside class="sidebar">
      <div class="logo">
        <el-icon :size="32" color="#409EFF">
          <Picture/>
        </el-icon>
        <span class="logo-text">SMFlare</span>
      </div>

      <nav class="menu">
        <div
            v-for="item in menuItems"
            :key="item.path"
            :class="['menu-item', { active: activeMenu === item.path }]"
            @click="handleMenuClick(item.path)"
        >
          <el-icon :size="20">
            <component :is="item.icon"/>
          </el-icon>
          <span class="menu-text">{{ item.name }}</span>
        </div>
      </nav>
    </aside>

    <!-- 右侧主内容区 -->
    <main class="main-content" @scroll="handleScroll">
      <!-- 上传区 -->
      <div v-if="activeMenu === 'upload'" class="upload-section">
        <div class="upload-header">
          <h2>上传区</h2>
        </div>

        <div class="upload-area" :class="{ uploading, dragging: isDragging }" @click="selectAndUploadImages">
          <el-icon :size="48" class="upload-icon">
            <Upload/>
          </el-icon>
          <p class="upload-text">{{ uploading ? '上传中...' : isDragging ? '松开鼠标上传' : '点击或拖拽图片上传' }}</p>
          <p class="upload-hint">支持 JPG、PNG、WebP 等格式</p>
        </div>

        <div class="remark-section">
          <div class="remark-card">
            <div class="remark-card-header">
              <div class="remark-title">
                <el-icon :size="18" class="remark-icon">
                  <Edit/>
                </el-icon>
                <span class="remark-label">备注信息</span>
              </div>
              <el-button
                  v-if="remark"
                  size="small"
                  text
                  type="danger"
                  :disabled="uploading"
                  @click="clearRemark"
              >
                清空
              </el-button>
            </div>
            <el-input
                v-model="remark"
                placeholder="为本次上传添加备注（可选）"
                :disabled="uploading"
                class="remark-input"
                clearable
                :rows="2"
                type="textarea"
                resize="none"
            />
          </div>
        </div>

        <div v-if="uploadResults.length > 0" class="upload-results">
          <div class="results-header">
            <h3>上传结果</h3>
            <div class="results-actions">
              <span class="results-count">共 {{ uploadResults.length }} 项</span>
              <el-button
                  size="small"
                  text
                  type="danger"
                  @click="clearUploadResults"
              >
                清空
              </el-button>
            </div>
          </div>
          <div class="results-list">
            <div
                v-for="(item, index) in reversedUploadResults"
                :key="`${item.filename}-${index}`"
                :class="['result-item', item.success ? 'success' : 'error']"
            >
              <div class="result-thumbnail">
                <div v-if="item.url" class="thumbnail-image">
                  <img :src="item.url" :alt="item.filename" loading="lazy"/>
                </div>
                <div v-else class="thumbnail-placeholder">
                  <el-icon :size="24">
                    <Picture/>
                  </el-icon>
                </div>
              </div>
              <div class="result-info">
                <div class="info-main">
                  <div class="info-status">
                    <span class="info-filename" :title="item.filename">{{ item.filename }}</span>
                  </div>
                  <div class="info-message" :title="item.message">{{ item.message }}</div>
                  <div v-if="item.remark" class="info-remark" :title="item.remark">
                    <el-icon :size="14" class="remark-icon">
                      <Edit/>
                    </el-icon>
                    <span>{{ item.remark }}</span>
                  </div>
                </div>
              </div>
              <div class="result-action">
                <el-button
                    v-if="item.url"
                    :icon="CopyDocument"
                    size="small"
                    type="primary"
                    @click="copyLink(item.url)"
                >
                  复制链接
                </el-button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 相册 -->
      <Gallery
          v-if="activeMenu === 'gallery'"
          :upload-history="uploadHistory"
          :history-loading="historyLoading"
          :d1-config-exists="d1ConfigExists"
          :smms-config-exists="smmsConfigExists"
          :is-loading-more="isLoadingMore"
          :has-more="hasMore"
          :image-detail-data="imageDetailData"
          v-model:detail-dialog-visible="detailDialogVisible"
          @navigate-to="handleNavigate"
          @copy-link="copyLink"
          @open-detail="openImageDetail"
          @refresh="refreshGallery"
      />

      <!-- 数据库设置 -->
      <div v-else-if="activeMenu === 'database-settings'" class="settings-section">
        <div class="section-header">
          <h2>数据库设置</h2>
        </div>

        <div class="settings-card">
          <h3 class="card-title">Cloudflare D1 数据库配置</h3>
          <p class="card-desc">配置 Cloudflare D1 数据库连接信息</p>

          <el-form :model="d1Form" label-width="120px" class="d1-form">
            <el-form-item label="Account ID">
              <el-input
                  v-model="d1Form.account_id"
                  placeholder="请输入 Cloudflare Account ID"
                  clearable
              />
            </el-form-item>

            <el-form-item label="Database ID">
              <el-input
                  v-model="d1Form.database_id"
                  placeholder="请输入 Database ID"
                  clearable
              />
            </el-form-item>

            <el-form-item label="API Token">
              <el-input
                  v-model="d1Form.api_token"
                  type="password"
                  placeholder="请输入 API Token"
                  show-password
                  clearable
              />
            </el-form-item>

            <el-form-item>
              <el-button
                  class="btn-primary-action"
                  :loading="d1TestLoading"
                  @click="testD1Connection"
              >
                {{ d1TestLoading ? '测试中...' : '测试连接' }}
              </el-button>
            </el-form-item>
          </el-form>
        </div>
      </div>

      <!-- 图床设置 -->
      <div v-else-if="activeMenu === 'image-bed-settings'" class="settings-section">
        <div class="section-header">
          <h2>图床设置</h2>
        </div>

        <div class="settings-card">
          <h3 class="card-title">SM.MS 图床配置</h3>
          <p class="card-desc">Base URL: https://sm.ms/api/v2/</p>

          <el-form :model="smmsForm" label-width="100px" class="smms-form">
            <el-form-item label="用户名/邮箱">
              <el-input
                  v-model="smmsForm.username"
                  placeholder="请输入用户名或邮箱"
                  clearable
              />
            </el-form-item>

            <el-form-item label="密码">
              <el-input
                  v-model="smmsForm.password"
                  type="password"
                  placeholder="请输入密码"
                  show-password
                  clearable
              />
            </el-form-item>

            <el-form-item>
              <el-button
                  class="btn-primary-action"
                  :loading="smmsLoading"
                  @click="getSmmsToken"
              >
                {{ smmsLoading ? '获取中...' : '获取 API Token' }}
              </el-button>
            </el-form-item>
          </el-form>
        </div>
      </div>

      <!-- 文件管理 -->
      <div v-else-if="activeMenu === 'files'" class="files-section">
        <PictureManager
            :d1-config-exists="d1ConfigExists"
            @navigate-to="handleNavigate"
            @refresh-gallery="clearGalleryCache"
        />
      </div>
    </main>
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

/* 左侧导航栏 */
.sidebar {
  width: var(--sidebar-width);
  background: linear-gradient(180deg, var(--color-sidebar-bg-start) 0%, var(--color-sidebar-bg-end) 100%);
  color: var(--color-sidebar-text);
  display: flex;
  flex-direction: column;
  box-shadow: var(--shadow-sidebar);
}

.logo {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-xl) var(--spacing-lg);
  gap: var(--gap-sm);
  border-bottom: 1px solid var(--color-border-light);
}

.logo-text {
  font-size: var(--font-size-h3);
  font-weight: var(--font-weight-semibold);
  color: var(--color-primary);
}

.menu {
  flex: 1;
  padding: var(--spacing-lg) 0;
}

.menu-item {
  display: flex;
  align-items: center;
  padding: var(--spacing-md) var(--spacing-lg);
  cursor: pointer;
  transition: background var(--transition-base), color var(--transition-base);
  gap: var(--gap-md);
  color: var(--color-sidebar-text-secondary);
  position: relative;
}

.menu-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: var(--color-primary);
  transform: scaleX(0);
  transform-origin: left;
  transition: transform var(--transition-base);
}

.menu-item:hover {
  background: var(--color-border-light);
  color: var(--color-text-white);
}

.menu-item.active {
  background: var(--color-primary-light);
  color: var(--color-primary);
}

.menu-item.active::before {
  transform: scaleX(1);
}

.menu-text {
  font-size: var(--font-size-base);
}

/* 右侧主内容区 */
.main-content {
  flex: 1;
  background: var(--color-bg-secondary);
  overflow-y: auto;
  padding: var(--spacing-xl);
}

.upload-section,
.settings-section,
.files-section {
  max-width: var(--container-md);
  margin: 0 auto;
}

.upload-header,
.section-header {
  margin-bottom: var(--spacing-xl);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.upload-header h2,
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

/* 上传区域 - 现代扁平化设计 */
.upload-area {
  background: var(--color-bg-primary);
  border: 1px dashed var(--color-border-base);
  border-radius: var(--radius-lg);
  padding: var(--spacing-3xl) var(--spacing-2xl);
  text-align: center;
  cursor: pointer;
  box-shadow: var(--shadow-sm);
  transition: border-color var(--transition-base), background-color var(--transition-base), box-shadow var(--transition-base), transform var(--transition-base);
}

.upload-area:hover {
  border-color: var(--color-primary);
  background-color: var(--color-primary-lighter);
  box-shadow: var(--shadow-lg);
  transform: translateY(-2px);
}

.upload-area.uploading {
  pointer-events: none;
  border-color: var(--color-primary);
}

.upload-area.dragging {
  border-color: var(--color-primary);
  background-color: var(--color-primary-lighter);
  transform: scale(1.02);
}

.upload-icon {
  display: block;
  margin: 0 auto var(--spacing-lg);
  color: var(--color-primary);
  transition: transform var(--transition-base);
}

.upload-area:hover .upload-icon {
  transform: scale(1.1);
}

/* 上传中：上下浮动效果 */
.upload-area.uploading .upload-icon {
  animation: uploadFloat 1.5s ease-in-out infinite;
}

@keyframes uploadFloat {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

.upload-text {
  font-size: var(--font-size-h5);
  color: var(--color-text-secondary);
  margin: 0 0 var(--gap-sm);
  font-weight: var(--font-weight-medium);
}

.upload-hint {
  font-size: var(--font-size-small);
  color: var(--color-text-tertiary);
}

/* 备注卡片 - 现代浮动设计 */
.remark-section {
  margin-top: var(--spacing-lg);
}

.remark-card {
  width: 100%;
  background: var(--color-bg-primary);
  border-radius: var(--radius-lg);
  padding: var(--spacing-lg);
  box-shadow: var(--shadow-sm);
  transition: box-shadow var(--transition-base), transform var(--transition-base);
  border: 1px solid var(--color-border-light);
}

.remark-card:hover {
  box-shadow: var(--shadow-lg);
  transform: translateY(-2px);
}

.remark-card:focus-within {
  box-shadow: 0 6px 24px rgba(0, 0, 0, 0.12);
}

.remark-card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--spacing-md);
}

.remark-title {
  display: flex;
  align-items: center;
  gap: var(--gap-sm);
}

.remark-icon {
  color: var(--color-primary);
}

.remark-label {
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-medium);
  color: var(--color-text-secondary);
}

.remark-input :deep(.el-textarea__inner) {
  background-color: var(--color-bg-primary);
  border: 1px dashed var(--color-border-base) !important;
  border-radius: var(--radius-md);
  transition: border-color var(--transition-base), background-color var(--transition-base);
  font-size: var(--font-size-base);
  line-height: 1.6;
  padding: var(--spacing-md);
  box-shadow: none !important;
}

.remark-input :deep(.el-textarea__inner):hover {
  border-color: var(--color-primary) !important;
  background-color: var(--color-primary-lighter);
}

.remark-input :deep(.el-textarea__inner):focus {
  border-color: var(--color-primary) !important;
  outline: none;
}

.remark-input :deep(.el-textarea__inner):disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.file-list {
  margin-top: var(--spacing-xl);
  background: var(--color-bg-primary);
  padding: var(--spacing-lg);
  border-radius: var(--radius-lg);
}

.file-list h3 {
  font-size: var(--font-size-h5);
  color: var(--color-text-primary);
  margin-bottom: var(--spacing-md);
}

/* 上传结果 - 现代化列表布局 */
.upload-results {
  margin-top: var(--spacing-xl);
}

.results-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--spacing-lg);
}

.results-header h3 {
  font-size: var(--font-size-h5);
  color: var(--color-text-primary);
  font-weight: var(--font-weight-medium);
  margin: 0;
}

.results-actions {
  display: flex;
  align-items: center;
  gap: var(--gap-md);
}

.results-count {
  font-size: var(--font-size-small);
  color: var(--color-text-tertiary);
  background: var(--color-bg-secondary);
  padding: 4px 12px;
  border-radius: var(--radius-md);
  font-weight: var(--font-weight-medium);
}

.results-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.result-item {
  background: var(--color-bg-primary);
  border-radius: var(--radius-lg);
  padding: 14px 16px;
  display: flex;
  align-items: center;
  gap: 14px;
  box-shadow: var(--shadow-sm);
  transition: box-shadow var(--transition-base), transform var(--transition-base);
  border: 1px solid var(--color-border-light);
  border-left: 3px solid transparent;
}

.result-item:hover {
  box-shadow: var(--shadow-lg);
  transform: translateY(-2px);
}

.result-item.success {
  border-left-color: var(--el-color-success);
}

.result-item.error {
  border-left-color: var(--el-color-danger);
}

.result-thumbnail {
  width: 60px;
  height: 60px;
  flex-shrink: 0;
  border-radius: var(--radius-md);
  overflow: hidden;
  background: linear-gradient(135deg, var(--color-bg-gradient-start) 0%, var(--color-bg-gradient-end) 100%);
}

.thumbnail-image {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.thumbnail-image img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.thumbnail-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--color-text-disabled);
}

.result-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.info-main {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.info-status {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-icon {
  flex-shrink: 0;
}

.status-icon.success {
  color: var(--el-color-success);
}

.status-icon.error {
  color: var(--el-color-danger);
}

.info-filename {
  font-size: var(--font-size-base);
  font-weight: var(--font-weight-semibold);
  color: var(--color-text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.4;
}

.info-message {
  font-size: var(--font-size-small);
  color: var(--color-text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.4;
}

.result-item.error .info-message {
  color: var(--el-color-danger);
}

.info-remark {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: var(--font-size-small);
  color: var(--color-text-secondary);
  line-height: 1.4;
  margin-top: 4px;
}

.info-remark .remark-icon {
  flex-shrink: 0;
  color: var(--el-color-primary);
  opacity: 0.8;
}

.info-remark span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-action {
  flex-shrink: 0;
}

.result-action .el-button {
  min-width: 90px;
}

/* 响应式设计 */
@media (max-width: 640px) {
  .result-item {
    flex-wrap: wrap;
    padding: 12px;
  }

  .result-thumbnail {
    width: 50px;
    height: 50px;
  }

  .result-info {
    flex: 1 1 calc(100% - 64px);
  }

  .result-action {
    flex: 1 1 100%;
    margin-top: 8px;
  }

  .result-action .el-button {
    width: 100%;
  }

  .info-message {
    padding-left: 0;
  }
}

/* 设置页面样式 - 统一扁平化设计 */
.settings-card {
  background: var(--color-bg-primary);
  padding: var(--spacing-xl);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  transition: box-shadow var(--transition-base);
}

.settings-card:hover {
  box-shadow: var(--shadow-lg);
}

.card-title {
  font-size: var(--font-size-h4);
  color: var(--color-text-primary);
  margin-bottom: var(--gap-sm);
  font-weight: var(--font-weight-medium);
}

.card-desc {
  font-size: var(--font-size-base);
  color: var(--color-text-tertiary);
  margin-bottom: var(--spacing-xl);
  line-height: var(--line-height-relaxed);
}

.smms-form {
  max-width: 500px;
}

.d1-form {
  max-width: 600px;
}

</style>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

#app {
  height: 100vh;
}
</style>
