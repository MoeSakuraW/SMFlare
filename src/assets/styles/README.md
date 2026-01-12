# 样式规范文档

## 概述

本项目采用基于CSS变量的设计系统，统一管理颜色、字体、间距、圆角、阴影等设计规范，确保视觉一致性和代码可维护性。

## 文件结构

```
src/assets/styles/
├── variables.css    # CSS变量定义（设计token）
├── common.css       # 通用组件样式
└── README.md        # 本文档
```

## 使用方法

### 1. 引入样式文件

在 `src/main.ts` 中已全局引入：

```typescript
import './assets/styles/variables.css'
import './assets/styles/common.css'
```

### 2. 使用CSS变量

在组件的 `<style>` 标签中直接使用CSS变量：

```vue

<style scoped>
  .my-component {
    background: var(--color-bg-primary);
    color: var(--color-text-primary);
    padding: var(--spacing-xl);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-sm);
  }
</style>
```

### 3. 使用通用组件样式

直接在模板中使用预定义的类名：

```vue

<template>
  <div class="page-container">
    <div class="section-header">
      <h2>页面标题</h2>
      <div class="header-actions">
        <el-button>操作按钮</el-button>
      </div>
    </div>

    <div class="card">
      <h3 class="card-title">卡片标题</h3>
      <p class="card-desc">卡片描述文字</p>
      <!-- 卡片内容 -->
    </div>
  </div>
</template>
```

## 设计规范

### 颜色系统

#### 主色调

- `--color-primary`: #409EFF（主色）
- `--color-primary-light`: rgba(64, 158, 255, 0.2)（浅色）
- `--color-primary-lighter`: #f0f9ff（更浅）
- `--color-primary-lightest`: #e6f7ff（最浅）

#### 背景色

- `--color-bg-primary`: #ffffff（主背景）
- `--color-bg-secondary`: #f5f7fa（次要背景）
- `--color-bg-hover`: #f0f7ff（悬停背景）
- `--color-bg-selected`: #ecf5ff（选中背景）

#### 文字颜色

- `--color-text-primary`: #303133（主文字）
- `--color-text-secondary`: #606266（次要文字）
- `--color-text-tertiary`: #909399（辅助文字）
- `--color-text-disabled`: #c0c4cc（禁用文字）

#### 边框颜色

- `--color-border-base`: #dcdfe6（基础边框）
- `--color-border-light`: rgba(255, 255, 255, 0.1)（浅色边框）

#### 按钮颜色系统

按钮颜色基于业务逻辑分类，而非简单套用Element Plus默认样式：

**主要操作（绿色系）** - 核心业务操作

- `--btn-primary-action`: #10b981
- `--btn-primary-action-hover`: #059669
- `--btn-primary-action-active`: #047857
- `--btn-primary-action-disabled`: #a7f3d0

**引导操作（蓝色系）** - 配置引导

- `--btn-guide`: #409EFF
- `--btn-guide-hover`: #66b1ff
- `--btn-guide-active`: #3a8ee6
- `--btn-guide-disabled`: #a0cfff

**内容操作（灰蓝色系）** - 高频辅助

- `--btn-content`: #606266
- `--btn-content-hover`: #909399
- `--btn-content-active`: #303133
- `--btn-content-disabled`: #c0c4cc

**状态操作（橙色系）** - 状态切换

- `--btn-state-active`: #f59e0b
- `--btn-state-active-hover`: #d97706
- `--btn-state-inactive`: #e5e7eb
- `--btn-state-inactive-hover`: #d1d5db

**次要操作（灰色系）** - 低优先级

- `--btn-secondary`: #909399
- `--btn-secondary-hover`: #606266
- `--btn-secondary-active`: #303133
- `--btn-secondary-disabled`: #e4e7ed

**危险操作（红色系）** - 删除等破坏性操作

- `--btn-danger`: #f56c6c
- `--btn-danger-active`: #f23c3c
- `--btn-danger-disabled`: #fab6b6

### 字体系统

#### 字体大小

- `--font-size-h1`: 32px
- `--font-size-h2`: 24px（页面标题）
- `--font-size-h3`: 20px
- `--font-size-h4`: 18px（卡片标题）
- `--font-size-h5`: 16px
- `--font-size-base`: 14px（正文）
- `--font-size-small`: 12px

#### 字重

- `--font-weight-normal`: 400
- `--font-weight-medium`: 500（标题）
- `--font-weight-semibold`: 600

#### 行高

- `--line-height-base`: 1.5
- `--line-height-relaxed`: 1.6
- `--line-height-loose`: 1.8

### 间距系统

#### 内边距/外边距

- `--spacing-xs`: 5px
- `--spacing-sm`: 10px
- `--spacing-md`: 15px
- `--spacing-lg`: 20px
- `--spacing-xl`: 30px（常用）
- `--spacing-2xl`: 40px
- `--spacing-3xl`: 80px

#### 间隙（gap）

- `--gap-xs`: 5px
- `--gap-sm`: 10px（常用）
- `--gap-md`: 12px
- `--gap-lg`: 16px
- `--gap-xl`: 20px

### 圆角系统

- `--radius-sm`: 4px
- `--radius-md`: 6px
- `--radius-lg`: 8px（常用）
- `--radius-xl`: 12px
- `--radius-full`: 50%（圆形）

### 阴影系统

- `--shadow-xs`: 0 1px 2px rgba(0, 0, 0, 0.05)
- `--shadow-sm`: 0 1px 4px rgba(0, 0, 0, 0.08)（卡片）
- `--shadow-md`: 0 2px 12px rgba(0, 0, 0, 0.05)
- `--shadow-lg`: 0 4px 12px rgba(0, 0, 0, 0.12)（hover）
- `--shadow-xl`: 0 8px 16px rgba(0, 0, 0, 0.12)
- `--shadow-sidebar`: 2px 0 8px rgba(0, 0, 0, 0.1)

### 过渡动画

- `--transition-fast`: 0.15s
- `--transition-base`: 0.3s（常用）
- `--transition-slow`: 0.5s

#### 缓动函数

- `--ease-in-out`: cubic-bezier(0.4, 0, 0.2, 1)
- `--ease-out`: cubic-bezier(0.0, 0, 0.2, 1)
- `--ease-in`: cubic-bezier(0.4, 0, 1, 1)

### 布局系统

#### 容器宽度

- `--container-sm`: 600px
- `--container-md`: 900px（常用）
- `--container-lg`: 1200px

#### 其他

- `--sidebar-width`: 180px

## 通用组件样式

### 页面容器

```vue

<div class="page-container">
  <!-- 内容自动居中，最大宽度900px -->
</div>
```

### 页面标题区域

```vue

<div class="section-header">
  <h2>标题</h2>
  <div class="header-actions">
    <el-button>操作</el-button>
  </div>
</div>
```

### 卡片组件

```vue

<div class="card">
  <h3 class="card-title">卡片标题</h3>
  <p class="card-desc">卡片描述</p>
  <!-- 卡片内容 -->
</div>
```

### 空状态

```vue

<div class="empty-state">
  <el-icon :size="64">
    <Picture/>
  </el-icon>
  <p>暂无数据</p>
</div>
```

### 加载状态

```vue

<div class="loading-more">
  <el-icon class="is-loading">
    <Loading/>
  </el-icon>
  <span>加载中...</span>
</div>

<div class="no-more">
  没有更多了
</div>
```

### 按钮样式

项目采用基于业务逻辑的按钮分类系统，共6种按钮类型：

#### 1. 主要操作按钮（btn-primary-action）

用于核心业务操作，如数据获取、验证、同步等。

```vue

<el-button class="btn-primary-action" :loading="loading" @click="handleAction">
  获取 API Token
</el-button>

<el-button class="btn-primary-action" @click="syncData">
  同步相册图片
</el-button>
```

#### 2. 引导操作按钮（btn-guide）

用于配置引导和导航，通常是文本按钮。

```vue

<el-button text class="btn-guide" @click="navigateToSettings">
  数据库设置
</el-button>
```

#### 3. 内容操作按钮（btn-content）

用于高频辅助操作，如复制、查看等。

```vue

<el-button text class="btn-content" :icon="CopyDocument" @click="copyLink">
  复制链接
</el-button>

<el-button text class="btn-content" @click="viewDetail">
  查看详情
</el-button>
```

#### 4. 状态操作按钮（btn-state）

用于状态切换，支持激活/未激活状态。

```vue
<!-- 简单状态按钮 -->
<el-button circle class="btn-state" :icon="Grid" @click="toggleLayout"/>

<!-- 动态状态按钮 -->
<el-button
    :icon="isFavorite ? StarFilled : Star"
    :class="['btn-state', { 'is-active': isFavorite }]"
    circle
    @click="toggleFavorite"
/>
```

#### 5. 次要操作按钮（btn-secondary）

用于低优先级操作，如外部跳转、取消等。

```vue

<el-button class="btn-secondary" @click="openExternal">
  在 SM.MS 打开
</el-button>
```

#### 6. 危险操作按钮（btn-danger）

用于删除等破坏性操作，通常是圆形图标按钮。

```vue

<el-button
    :icon="Delete"
    class="btn-danger"
    circle
    title="删除"
    @click="handleDelete"
/>
```

#### 视觉层级

按钮的视觉优先级从高到低：

1. **btn-primary-action**（最高）- 绿色，实心，最突出
2. **btn-danger**（高）- 红色，圆形按钮，危险警示
3. **btn-guide**（中高）- 蓝色，文本按钮，柔和
4. **btn-content**（中）- 灰蓝色，文本按钮
5. **btn-state**（中低）- 橙色/灰色，圆形按钮
6. **btn-secondary**（最低）- 灰色，边框按钮

## 最佳实践

### 1. 优先使用CSS变量

❌ 不推荐：

```css
.my-component {
    color: #303133;
    padding: 20px;
    border-radius: 8px;
}
```

✅ 推荐：

```css
.my-component {
    color: var(--color-text-primary);
    padding: var(--spacing-lg);
    border-radius: var(--radius-lg);
}
```

### 2. 优先使用通用组件样式

❌ 不推荐：

```vue

<div style="max-width: 900px; margin: 0 auto;">
  <div style="display: flex; justify-content: space-between;">
    <h2>标题</h2>
    <button>操作</button>
  </div>
</div>
```

✅ 推荐：

```vue

<div class="page-container">
  <div class="section-header">
    <h2>标题</h2>
    <div class="header-actions">
      <el-button>操作</el-button>
    </div>
  </div>
</div>
```

### 3. 保持一致性

- 页面标题统一使用 `--font-size-h2` (24px)
- 卡片标题统一使用 `--font-size-h4` (18px)
- 卡片圆角统一使用 `--radius-lg` (8px)
- 卡片阴影统一使用 `--shadow-sm`
- hover效果统一使用 `--shadow-lg`

### 4. 按钮使用规范

#### 选择正确的按钮类型

根据按钮的业务功能选择对应的样式类：

❌ 不推荐：

```vue
<!-- 错误：数据操作使用Element Plus默认type -->
<el-button type="primary" @click="syncData">同步数据</el-button>

<!-- 错误：复制链接使用primary -->
<el-button type="primary" text @click="copyLink">复制链接</el-button>
```

✅ 推荐：

```vue
<!-- 正确：数据操作使用btn-primary-action -->
<el-button class="btn-primary-action" @click="syncData">同步数据</el-button>

<!-- 正确：复制链接使用btn-content -->
<el-button text class="btn-content" @click="copyLink">复制链接</el-button>
```

#### 状态按钮的正确使用

❌ 不推荐：

```vue
<!-- 错误：使用硬编码颜色 -->
<el-button :style="{ backgroundColor: isFavorite ? '#f59e0b' : '#e5e7eb' }">
  收藏
</el-button>
```

✅ 推荐：

```vue
<!-- 正确：使用btn-state和动态class -->
<el-button
    :icon="isFavorite ? StarFilled : Star"
    :class="['btn-state', { 'is-active': isFavorite }]"
    circle
    @click="toggleFavorite"
/>
```

#### 按钮分类决策树

```
需要添加按钮？
├─ 是核心业务操作（获取、测试、同步）？
│  └─ 使用 btn-primary-action
├─ 是危险/破坏性操作（删除、移除）？
│  └─ 使用 btn-danger（圆形按钮）
├─ 是配置引导或导航？
│  └─ 使用 btn-guide（文本按钮）
├─ 是高频辅助操作（复制、查看）？
│  └─ 使用 btn-content（文本按钮）
├─ 是状态切换（收藏、布局）？
│  └─ 使用 btn-state（圆形按钮）
└─ 是低优先级操作（外部跳转、取消）？
   └─ 使用 btn-secondary
```

### 5. 响应式设计

使用CSS变量配合媒体查询：

```css
.my-component {
    padding: var(--spacing-lg);
}

@media (min-width: 768px) {
    .my-component {
        padding: var(--spacing-xl);
    }
}
```

## 扩展设计系统

如需添加新的设计token，请在 `variables.css` 中添加：

```css
:root {
    /* 新增颜色 */
    --color-success: #67c23a;
    --color-warning: #e6a23c;
    --color-danger: #f56c6c;

    /* 新增间距 */
    --spacing-4xl: 100px;
}
```

## 维护指南

1. **不要直接修改组件中的硬编码值**，应该修改 `variables.css` 中的变量
2. **新增组件样式时**，优先考虑是否可以复用现有的通用样式
3. **定期审查**，确保所有组件都使用了CSS变量
4. **保持文档更新**，新增变量或样式时同步更新本文档

## 迁移指南

如果需要将旧代码迁移到新的设计系统：

1. 使用批量替换工具替换常见的硬编码值
2. 手动检查特殊情况
3. 测试所有页面确保样式正确

常见替换规则：

- `#409EFF` → `var(--color-primary)`
- `#303133` → `var(--color-text-primary)`
- `24px` (标题) → `var(--font-size-h2)`
- `14px` (正文) → `var(--font-size-base)`
- `8px` (圆角) → `var(--radius-lg)`
- `30px` (间距) → `var(--spacing-xl)`

### 按钮迁移规则

将Element Plus默认type迁移到业务逻辑分类：

**数据操作按钮：**

```vue
<!-- 旧代码 -->
<el-button type="success" @click="syncData">同步数据</el-button>
<el-button type="primary" @click="getData">获取数据</el-button>

<!-- 新代码 -->
<el-button class="btn-primary-action" @click="syncData">同步数据</el-button>
<el-button class="btn-primary-action" @click="getData">获取数据</el-button>
```

**引导和导航按钮：**

```vue
<!-- 旧代码 -->
<el-button text type="primary" @click="goToSettings">设置</el-button>

<!-- 新代码 -->
<el-button text class="btn-guide" @click="goToSettings">设置</el-button>
```

**辅助操作按钮：**

```vue
<!-- 旧代码 -->
<el-button type="info" text @click="copyLink">复制</el-button>

<!-- 新代码 -->
<el-button text class="btn-content" @click="copyLink">复制</el-button>
```

**状态切换按钮：**

```vue
<!-- 旧代码 -->
<el-button :type="isFavorite ? 'warning' : 'default'" circle>
  <el-icon>
    <Star/>
  </el-icon>
</el-button>

<!-- 新代码 -->
<el-button
    :icon="isFavorite ? StarFilled : Star"
    :class="['btn-state', { 'is-active': isFavorite }]"
    circle
    @click="toggleFavorite"
/>
```
