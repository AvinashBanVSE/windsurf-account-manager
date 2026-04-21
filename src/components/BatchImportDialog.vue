<template>
  <el-dialog
    v-model="visible"
    title="Batch Import Accounts"
    width="700px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <div class="import-container">
      <!-- Auth type (hidden for devin_session_token mode) -->
      <div class="mode-section" v-if="importMode !== 'devin_session_token'">
        <span class="mode-label">Auth Type:</span>
        <div class="mode-grid mode-grid--3col" role="radiogroup" aria-label="Auth Type">
          <div
            v-for="opt in authProviderOptions"
            :key="opt.value"
            class="mode-card"
            :class="{ 'is-active': authProvider === opt.value }"
            :title="opt.desc"
            role="radio"
            :aria-checked="authProvider === opt.value"
            tabindex="0"
            @click="selectAuthProvider(opt.value)"
            @keydown.enter.prevent="selectAuthProvider(opt.value)"
            @keydown.space.prevent="selectAuthProvider(opt.value)"
          >
            <el-icon class="mode-card__icon">
              <component :is="opt.icon" />
            </el-icon>
            <span class="mode-card__title">{{ opt.title }}</span>
            <el-tag
              v-if="opt.tag"
              :type="opt.tagType"
              size="small"
              effect="light"
              class="mode-card__tag"
            >
              {{ opt.tag }}
            </el-tag>
            <el-icon v-if="authProvider === opt.value" class="mode-card__check">
              <Check />
            </el-icon>
          </div>
        </div>
      </div>

      <!-- Import mode switch (Devin / smart doesn't support Refresh Token; devin_session_token only goes to Devin) -->
      <div class="mode-section">
        <span class="mode-label">Import Mode:</span>
        <div class="mode-grid mode-grid--3col" role="radiogroup" aria-label="Import Mode">
          <div
            v-for="opt in importModeOptions"
            :key="opt.value"
            class="mode-card"
            :class="{
              'is-active': importMode === opt.value,
              'is-disabled': opt.disabled,
            }"
            :title="opt.disabled && opt.disabledReason ? opt.disabledReason : opt.desc"
            role="radio"
            :aria-checked="importMode === opt.value"
            :aria-disabled="opt.disabled"
            :tabindex="opt.disabled ? -1 : 0"
            @click="!opt.disabled && selectImportMode(opt.value)"
            @keydown.enter.prevent="!opt.disabled && selectImportMode(opt.value)"
            @keydown.space.prevent="!opt.disabled && selectImportMode(opt.value)"
          >
            <el-icon class="mode-card__icon">
              <component :is="opt.icon" />
            </el-icon>
            <span class="mode-card__title">{{ opt.title }}</span>
            <el-tag
              v-if="opt.tag"
              :type="opt.tagType"
              size="small"
              effect="light"
              class="mode-card__tag"
            >
              {{ opt.tag }}
            </el-tag>
            <el-icon v-if="importMode === opt.value" class="mode-card__check">
              <Check />
            </el-icon>
          </div>
        </div>
      </div>

      <!-- Format instructions -->
      <el-alert
        :type="importMode === 'devin_session_token' ? 'warning' : (authProvider === 'firebase' ? 'info' : 'success')"
        :closable="false"
        show-icon
        style="margin-bottom: 16px;"
      >
        <template #title>
          <span v-if="importMode === 'devin_session_token'">
            [Devin Session Token] One token per line, format: <code>devin-session-token$... Remarks(optional)</code>.
            System calls GetCurrentUser for each line to get email / quota / api_key; invalid or expired tokens will be counted as import failures.
          </span>
          <span v-else-if="importMode === 'password' && authProvider === 'smart'">
            [Smart Detection] One account per line, format: <code>Email Password Remarks(optional)</code>.
            System concurrently detects <strong>Firebase</strong> / <strong>Devin Auth1</strong> for each line and auto-routes;
            SSO / no password set / unregistered accounts will be counted as import failures.
          </span>
          <span v-else-if="importMode === 'password' && authProvider === 'devin'">
            [Devin] One account per line, format: <code>Email Password Remarks(optional)</code>.
            Multi-organization accounts will automatically select the first organization to complete import.
          </span>
          <span v-else-if="importMode === 'password'">
            One account per line, supports space or hyphen separator:
            <code>Email Password Remarks(optional)</code> or <code>Email---Password---Remarks(optional)</code>
          </span>
          <span v-else>One token per line, format: <code>refresh_token Remarks(optional)</code></span>
        </template>
      </el-alert>

      <!-- Input area -->
      <div class="input-section">
        <div class="section-header">
          <span class="section-title">{{ sectionTitle }}</span>
          <el-button type="primary" link @click="handleFileImport">
            <el-icon><Upload /></el-icon>
Import from file
          </el-button>
        </div>
        <el-input
          v-model="inputText"
          type="textarea"
          :rows="12"
          :placeholder="inputPlaceholder"
          @input="parseAccounts"
        />
        <input
          ref="fileInputRef"
          type="file"
          accept=".txt,.csv"
          style="display: none;"
          @change="handleFileSelected"
        />
      </div>

      <!-- Parse preview -->
      <div class="preview-section" v-if="inputText.trim()">
        <div class="section-header">
          <span class="section-title">Parse Preview</span>
          <div class="stats">
            <el-tag type="success" size="small">Valid: {{ validAccounts.length }}</el-tag>
            <el-tag v-if="invalidLines.length > 0" type="danger" size="small">
              Invalid: {{ invalidLines.length }}
            </el-tag>
          </div>
        </div>
        
        <!-- Valid account table -->
        <el-table
          v-if="validAccounts.length > 0"
          :data="validAccounts.slice(0, 10)"
          size="small"
          max-height="200"
          stripe
        >
          <el-table-column prop="email" label="Email" min-width="180" />
          <el-table-column prop="password" label="Password" width="120">
            <template #default="{ row }">
              <span class="password-mask">{{ maskPassword(row.password) }}</span>
            </template>
          </el-table-column>
          <el-table-column prop="remark" label="Remarks" min-width="100">
            <template #default="{ row }">
              <span class="remark-text">{{ row.remark || '-' }}</span>
            </template>
          </el-table-column>
        </el-table>
        <div v-if="validAccounts.length > 10" class="more-hint">
          ... and {{ validAccounts.length - 10 }} more accounts
        </div>

        <!-- Invalid line hints -->
        <el-alert
          v-if="invalidLines.length > 0"
          type="warning"
          :closable="false"
          style="margin-top: 12px;"
        >
          <template #title>
            Invalid lines: {{ invalidLines.slice(0, 5).join(', ') }}
            <span v-if="invalidLines.length > 5">... and {{ invalidLines.length }} more lines</span>
          </template>
        </el-alert>
      </div>

      <!-- Import settings -->
      <div class="settings-section">
        <div class="section-header">
          <span class="section-title">Import Settings</span>
        </div>
        <div class="settings-content">
          <!-- Group selection -->
          <div class="setting-item">
            <span class="setting-label">Group:</span>
            <el-select
              v-model="selectedGroup"
              placeholder="Select group (optional)"
              clearable
              style="width: 180px;"
            >
              <el-option
                v-for="group in settingsStore.groups"
                :key="group"
                :label="group"
                :value="group"
              />
            </el-select>
            <span class="setting-hint">Leave empty to use default group</span>
          </div>
          
          <!-- Tag selection -->
          <div class="setting-item">
            <span class="setting-label">Tags:</span>
            <el-select
              v-model="selectedTags"
              multiple
              collapse-tags
              collapse-tags-tooltip
              placeholder="Select tags (optional)"
              clearable
              style="width: 180px;"
            >
              <el-option
                v-for="tag in settingsStore.tags"
                :key="tag.name"
                :label="tag.name"
                :value="tag.name"
              >
                <span :style="{ color: tag.color }">{{ tag.name }}</span>
              </el-option>
            </el-select>
            <span class="setting-hint">Leave empty to not add tags</span>
          </div>
          
          <div class="setting-item">
            <span class="setting-label">Concurrent Mode:</span>
            <el-tag :type="unlimitedConcurrent ? 'danger' : 'primary'" size="small">
              {{ unlimitedConcurrent ? 'Full Concurrency' : `Limited Concurrency (${concurrencyLimit})` }}
            </el-tag>
            <span class="setting-hint">Can modify in settings</span>
          </div>
          <div class="setting-item">
            <el-checkbox v-model="autoLogin">Auto login after import</el-checkbox>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose">Cancel</el-button>
        <el-button
          type="primary"
          :disabled="validAccounts.length === 0"
          :loading="importing"
          @click="handleImport"
        >
          {{ importing ? 'Importing...' : `Import ${validAccounts.length} accounts` }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import {
  Upload,
  MagicStick,
  Platform,
  User,
  Lock,
  Refresh,
  Connection,
  Check,
} from '@element-plus/icons-vue';
import { useSettingsStore } from '@/store';

const props = defineProps<{
  modelValue: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
  (
    e: 'import',
    accounts: Array<{ email: string; password: string; remark: string; refreshToken?: string; sessionToken?: string }>,
    autoLogin: boolean,
    group: string,
    tags: string[],
    mode: 'password' | 'refresh_token' | 'devin_session_token',
    authProvider: 'firebase' | 'devin' | 'smart',
  ): void;
}>();

const settingsStore = useSettingsStore();

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
});

const inputText = ref('');
const validAccounts = ref<Array<{ email: string; password: string; remark: string; refreshToken?: string; sessionToken?: string }>>([]);
const invalidLines = ref<number[]>([]);
const autoLogin = ref(true);
const importing = ref(false);
const fileInputRef = ref<HTMLInputElement | null>(null);
const selectedGroup = ref<string>('');
const selectedTags = ref<string[]>([]);
const importMode = ref<'password' | 'refresh_token' | 'devin_session_token'>('password');
/// Auth provider:
/// - `smart` (default, recommended): Sniffs each line for Firebase / Devin and automatically dispatches to the corresponding command
/// - `firebase`: Manually forces the original add_account + login_account flow
/// - `devin`：手动强制走 add_account_by_devin_login，多组织自动选 orgs[0]
const authProvider = ref<'firebase' | 'devin' | 'smart'>('smart');

// After switching to Devin / smart, Refresh Token mode is not applicable (smart mode cannot sniff tokens without an email)
// 自动回落到邮箱密码模式并清空输入
watch(authProvider, (val) => {
  if ((val === 'devin' || val === 'smart') && importMode.value === 'refresh_token') {
    importMode.value = 'password';
    handleModeChange();
  }
});

/**
 * Auth provider card options (3 fixed items)
 *
 * - smart: Recommended provider, sniffs and dispatches automatically
 * - firebase: Forces the traditional Firebase system
 * - devin: Forces the new Devin Session system
 */
const authProviderOptions = [
  {
    value: 'smart' as const,
    title: 'Smart Detection',
    desc: 'Concurrent detection for Firebase / Devin, auto-routes to corresponding mode',
    icon: MagicStick,
    tag: 'Recommended',
    tagType: 'primary' as const,
  },
  {
    value: 'firebase' as const,
    title: 'Firebase (Official)',
    desc: 'Force use original add_account + login_account (Firebase system)',
    icon: Platform,
    tag: '',
    tagType: 'info' as const,
  },
  {
    value: 'devin' as const,
    title: 'Devin (New)',
    desc: 'Force use add_account_by_devin_login, multi-organization auto-select orgs[0]',
    icon: User,
    tag: 'New',
    tagType: 'success' as const,
  },
];

/**
 * Import mode card options (3 items, dynamically disabled by authProvider)
 *
 * - password: Email + Password + optional remarks
 * - refresh_token: Firebase refresh_token; only available when authProvider === 'firebase'
 * - devin_session_token: devin-session-token$... import
 */
const importModeOptions = computed(() => [
  {
    value: 'password' as const,
    title: 'Email & Password',
    desc: 'One account per line: Email Password [Remarks]',
    icon: Lock,
    tag: '',
    tagType: 'info' as const,
    disabled: false,
    disabledReason: '',
  },
  {
    value: 'refresh_token' as const,
    title: 'Refresh Token',
    desc: 'One Firebase refresh_token per line (+ optional remarks)',
    icon: Refresh,
    tag: '',
    tagType: 'info' as const,
    disabled: authProvider.value === 'devin' || authProvider.value === 'smart',
    disabledReason:
      authProvider.value === 'devin'
        ? 'Refresh token is not applicable for Devin system'
        : authProvider.value === 'smart'
          ? 'Smart detection requires email, Token format cannot be detected'
          : '',
  },
  {
    value: 'devin_session_token' as const,
    title: 'Devin Session Token',
    desc: 'Paste devin-session-token$... to import directly, no email/password needed',
    icon: Connection,
    tag: 'Import',
    tagType: 'warning' as const,
    disabled: false,
    disabledReason: '',
  },
]);

/**
 * Select auth provider: equivalent to the original v-model="authProvider".
 * Returns early on same-value clicks to avoid watch side effects.
 */
function selectAuthProvider(value: 'smart' | 'firebase' | 'devin') {
  if (authProvider.value === value) return;
  authProvider.value = value;
}

/**
 * Select import mode: equivalent to the original v-model + @change="handleModeChange".
 * Disabled items are intercepted at the template layer; this function only handles valid switches.
 */
function selectImportMode(value: 'password' | 'refresh_token' | 'devin_session_token') {
  if (importMode.value === value) return;
  importMode.value = value;
  handleModeChange();
}

const unlimitedConcurrent = computed(() => settingsStore.settings?.unlimitedConcurrentRefresh || false);
const concurrencyLimit = computed(() => settingsStore.settings?.concurrent_limit || 5);

// Generate section title and placeholder based on current mode
const sectionTitle = computed(() => {
  if (importMode.value === 'devin_session_token') return 'Devin Session Token List';
  return importMode.value === 'password' ? 'Account Data' : 'Refresh Token List';
});
const inputPlaceholder = computed(() => {
  if (importMode.value === 'password') {
    return 'user1@example.com password123 Test Account 1\nuser2@example.com---password456\nuser3@example.com---password789---Remarks Info';
  }
  if (importMode.value === 'refresh_token') {
    return 'AMf-vBx...longtoken... Test Account 1\nAMf-vBy...longtoken...\nAMf-vBz...longtoken... Remarks Info';
  }
  // devin_session_token
  return 'devin-session-token$eyJhbGciOi... Test Account 1\ndevin-session-token$eyJhbGciOi...\ndevin-session-token$eyJhbGciOi... Remarks Info';
});

/**
 * 批量导入行切分：同时支持空白分隔与 `---`（3+ 个 `-`）分隔。
 *
 * 优先判定是否存在 3+ 个连续 `-`（量阈避免与 emails / refresh_token 中偶发的单/双连字符
 * 冲突）——存在则按它切；否则回退到空白切分，保证历史格式向后兼容。
 *
 * 空段会被过滤（避免连续多个分隔符间的空值干扰后续 parts.length 判定）。
 */
function splitLine(line: string): string[] {
  const trimmed = line.trim();
  if (/-{3,}/.test(trimmed)) {
    return trimmed.split(/-{3,}/).map(s => s.trim()).filter(s => s !== '');
  }
  return trimmed.split(/\s+/);
}

// 切换模式时重置
function handleModeChange() {
  inputText.value = '';
  validAccounts.value = [];
  invalidLines.value = [];
}

// Parse account data
function parseAccounts() {
  const lines = inputText.value.split('\n').filter(line => line.trim());
  validAccounts.value = [];
  invalidLines.value = [];

  if (importMode.value === 'password') {
    // Email/password mode: supports both `email password remark` and `email---password---remark` formats
    lines.forEach((line, index) => {
      const parts = splitLine(line);
      if (parts.length >= 2) {
        const [email, password, ...remarkParts] = parts;
        if (email.includes('@')) {
          validAccounts.value.push({
            email,
            password,
            remark: remarkParts.join(' ') || ''
          });
        } else {
          invalidLines.value.push(index + 1);
        }
      } else {
        invalidLines.value.push(index + 1);
      }
    });
  } else if (importMode.value === 'devin_session_token') {
    // Devin Session Token mode: the first non-whitespace segment is the session_token, must start with devin-session-token$
    lines.forEach((line, index) => {
      const parts = splitLine(line);
      if (parts.length >= 1 && parts[0].startsWith('devin-session-token$')) {
        const [token, ...remarkParts] = parts;
        validAccounts.value.push({
          email: `Session #${index + 1}`, // 实际 email 由后端反查填写；占位用于预览表格
          password: '',
          remark: remarkParts.join(' ') || '',
          sessionToken: token,
        });
      } else {
        invalidLines.value.push(index + 1);
      }
    });
  } else {
    // Refresh Token 模式
    lines.forEach((line, index) => {
      const parts = splitLine(line);
      if (parts.length >= 1 && parts[0].length >= 10) {
        const [token, ...remarkParts] = parts;
        validAccounts.value.push({
          email: `Token #${index + 1}`,
          password: '',
          remark: remarkParts.join(' ') || '',
          refreshToken: token
        });
      } else {
        invalidLines.value.push(index + 1);
      }
    });
  }
}

// 遮蔽密码显示
function maskPassword(password: string): string {
  if (password.length <= 4) {
    return '*'.repeat(password.length);
  }
  return password.slice(0, 2) + '*'.repeat(password.length - 4) + password.slice(-2);
}

// 从文件导入
function handleFileImport() {
  fileInputRef.value?.click();
}

function handleFileSelected(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  const reader = new FileReader();
  reader.onload = (e) => {
    const content = e.target?.result as string;
    inputText.value = content;
    parseAccounts();
  };
  reader.readAsText(file);
  
  // 重置input，允许再次选择同一文件
  input.value = '';
}

// 执行导入
function handleImport() {
  if (validAccounts.value.length === 0) return;
  importing.value = true;
  emit(
    'import',
    [...validAccounts.value],
    autoLogin.value,
    selectedGroup.value || 'Default Group',
    [...selectedTags.value],
    importMode.value,
    authProvider.value,
  );
}

// Close dialog
function handleClose() {
  if (!importing.value) {
    inputText.value = '';
    validAccounts.value = [];
    invalidLines.value = [];
    selectedGroup.value = '';
    selectedTags.value = [];
    importMode.value = 'password';
    authProvider.value = 'smart';
    visible.value = false;
  }
}

// 导入完成后重置状态
function resetImporting() {
  importing.value = false;
}

// 监听对话框关闭
watch(visible, (val) => {
  if (!val) {
    inputText.value = '';
    validAccounts.value = [];
    invalidLines.value = [];
    selectedGroup.value = '';
    selectedTags.value = [];
    importing.value = false;
    importMode.value = 'password';
    authProvider.value = 'smart';
  }
});

defineExpose({
  resetImporting
});
</script>

<style scoped>
.import-container {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* ==================== 模式选择区（认证流派 / 导入模式） ==================== */

/* label 在左，卡片网格占右侧剩余空间；背景切换为中性浅灰，不再用绿背高亮 */
.mode-section {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 10px 12px;
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
}

.mode-label {
  flex-shrink: 0;
  padding-top: 8px; /* 与卡片文字垂直居中 */
  min-width: 72px;
  font-size: 13px;
  font-weight: 500;
  color: var(--el-text-color-regular);
}

/* ==================== 卡片式 radio 网格（对齐 AddAccountDialog 风格） ==================== */

/* 默认 2 列；BatchImport 使用 .mode-grid--3col 显式声明 3 列。窄屏自动降为单列 */
.mode-grid {
  flex: 1;
  min-width: 0;
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
}
.mode-grid--3col {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

/* Single card: single-line flex, 34px height; description is in the native title tooltip */
.mode-card {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border: 1.5px solid var(--el-border-color);
  border-radius: 6px;
  background-color: var(--el-bg-color);
  cursor: pointer;
  transition: border-color 0.2s ease, background-color 0.2s ease, box-shadow 0.2s ease, opacity 0.2s ease;
  user-select: none;
  outline: none;
  min-height: 34px;
}

.mode-card:hover {
  border-color: var(--el-color-primary-light-3);
  background-color: var(--el-color-primary-light-9);
}

.mode-card:focus-visible {
  box-shadow: 0 0 0 2px var(--el-color-primary-light-5);
}

.mode-card.is-active {
  border-color: var(--el-color-primary);
  background-color: var(--el-color-primary-light-9);
  box-shadow: 0 0 0 2px var(--el-color-primary-light-7);
}

/* 禁用态：变灰不可点击；覆盖 hover 效果；原因说明通过模板的 :title 属性展示 */
.mode-card.is-disabled {
  opacity: 0.55;
  cursor: not-allowed;
  background-color: var(--el-fill-color-light);
}
.mode-card.is-disabled:hover {
  border-color: var(--el-border-color);
  background-color: var(--el-fill-color-light);
  box-shadow: none;
}

.mode-card__icon {
  flex-shrink: 0;
  font-size: 18px;
  color: var(--el-color-primary);
  width: 18px;
  height: 18px;
}

.mode-card.is-disabled .mode-card__icon {
  color: var(--el-text-color-placeholder);
}

.mode-card__title {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--el-text-color-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mode-card.is-disabled .mode-card__title {
  color: var(--el-text-color-secondary);
}

.mode-card__tag {
  flex-shrink: 0;
}

.mode-card__check {
  flex-shrink: 0;
  font-size: 14px;
  color: var(--el-color-primary);
}

/* 窄屏降级：dialog 宽度 < 680px 时卡片变单列，label 也换行 */
@media (max-width: 680px) {
  .mode-section {
    flex-direction: column;
    align-items: stretch;
  }
  .mode-label {
    padding-top: 0;
    min-width: 0;
  }
  .mode-grid,
  .mode-grid--3col {
    grid-template-columns: 1fr;
  }
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.section-title {
  font-weight: 600;
  color: var(--el-text-color-primary);
}

.input-section :deep(.el-textarea__inner) {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.6;
}

.preview-section {
  background: var(--el-fill-color-light);
  border-radius: 8px;
  padding: 12px;
}

.stats {
  display: flex;
  gap: 8px;
}

.password-mask {
  font-family: monospace;
  color: var(--el-text-color-secondary);
}

.remark-text {
  color: var(--el-text-color-secondary);
  font-size: 12px;
}

.more-hint {
  text-align: center;
  color: var(--el-text-color-secondary);
  font-size: 12px;
  padding: 8px 0;
}

.settings-section {
  background: var(--el-fill-color-lighter);
  border-radius: 8px;
  padding: 12px;
}

.settings-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.setting-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.setting-label {
  color: var(--el-text-color-regular);
  font-size: 13px;
}

.setting-hint {
  color: var(--el-text-color-placeholder);
  font-size: 12px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

code {
  background: var(--el-fill-color);
  padding: 2px 6px;
  border-radius: 4px;
  font-family: monospace;
}                                