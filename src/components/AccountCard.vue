<template>
  <div 
    class="account-card" 
    :class="{ 'selected': isSelected, 'active': account.status === 'active', 'current': isCurrent, 'has-tag-color': hasColoredTag, 'is-disabled': account.is_disabled, 'subscription-inactive': isPaidPlan && account.subscription_active === false }"
    :style="cardBorderStyle"
    @click="handleCardClick"
  >
    <div class="card-header">
      <!-- 拖拽手柄 - simple 版本已禁用
      <div class="drag-handle" title="Drag to reorder">
        <el-icon><Rank /></el-icon>
      </div>
      -->
      <el-checkbox 
        :model-value="isSelected"
        @change="handleSelect"
        class="select-checkbox"
      />
      <div class="account-info">
        <div class="email" :title="'Click to copy: ' + account.email" @click.stop="copyEmail">{{ displayEmail }}</div>
        <el-tag 
          v-if="account.nickname"
          type="warning"
          size="small"
          effect="light"
          class="nickname-tag"
        >
          {{ account.nickname }}
        </el-tag>
        <el-tooltip
          v-if="account.auth_provider === 'devin'"
          content="Authenticated via Devin Session (new system)"
          placement="top"
        >
          <el-tag
            type="success"
            size="small"
            effect="dark"
            class="devin-tag"
          >
            Devin
          </el-tag>
        </el-tooltip>
      </div>
      <div class="status-indicator" :class="statusClass">
        <span class="status-dot"></span>
        <span class="status-text">{{ statusText }}</span>
      </div>
    </div>

    <div class="card-body">
      <!-- Quota and plan information -->
      <div class="quota-section" v-if="hasQuotaData">
        <div class="quota-header">
          <div class="quota-header-left">
            <el-tag v-if="account.plan_name" :class="['plan-tag', `plan-${account.plan_name?.toLowerCase()}`]" size="small">
              <el-icon>
                <User v-if="account.plan_name?.toLowerCase() === 'free'" />
                <Trophy v-else />
              </el-icon>
              {{ account.plan_name }}
            </el-tag>
          </div>
          <!-- CREDITS mode: display credit value -->
          <div class="quota-header-right" v-if="!isQuotaMode">
            <span class="quota-used">{{ formatQuota(account.used_quota) }}</span>
            <span class="quota-separator">/</span>
            <span class="quota-total">{{ formatQuota(account.total_quota) }}</span>
          </div>
        </div>

        <!-- QUOTA mode: daily and weekly quota percentage -->
        <template v-if="isQuotaMode">
          <div class="quota-percent-row">
            <span class="quota-percent-label">Daily Quota</span>
            <el-progress
              :percentage="account.daily_quota_remaining_percent ?? 0"
              :stroke-width="8"
              :color="dailyQuotaColor"
              :show-text="false"
              class="quota-percent-bar"
            />
            <span class="quota-percent-value">{{ account.daily_quota_remaining_percent ?? 0 }}%</span>
            <span class="quota-reset-time" v-if="formatResetTime(account.daily_quota_reset_at_unix)">
              {{ formatResetTime(account.daily_quota_reset_at_unix) }}
            </span>
          </div>
          <div class="quota-percent-row">
            <span class="quota-percent-label">Weekly Quota</span>
            <el-progress
              :percentage="account.weekly_quota_remaining_percent ?? 0"
              :stroke-width="8"
              :color="weeklyQuotaColor"
              :show-text="false"
              class="quota-percent-bar"
            />
            <span class="quota-percent-value">{{ account.weekly_quota_remaining_percent ?? 0 }}%</span>
            <span class="quota-reset-time" v-if="formatResetTime(account.weekly_quota_reset_at_unix)">
              {{ formatResetTime(account.weekly_quota_reset_at_unix) }}
            </span>
          </div>
        </template>

        <!-- CREDITS 模式：积分进度条 -->
        <template v-else>
          <div class="quota-progress">
            <el-progress
              :percentage="quotaPercentage"
              :stroke-width="8"
              :color="quotaColor"
              :show-text="false"
            />
            <span class="quota-percentage">{{ quotaPercentage }}%</span>
          </div>
        </template>
        
        <!-- Subscription expiration time (integrated within the quota block) -->
        <div class="quota-expiry" v-if="account.subscription_expires_at">
          <el-icon class="expiry-icon"><Clock /></el-icon>
          <span class="expiry-label">Expiry:</span>
          <span class="expiry-date">{{ formattedExpiryDate }}</span>
          <span v-if="daysUntilExpiry !== null" :class="['expiry-badge', expiryClass]">
            {{ expiryText }}
          </span>
        </div>
      </div>

      <div class="tags">
        <el-tag 
          v-for="tag in account.tags" 
          :key="tag"
          size="small"
          :style="getTagStyle(tag)"
          class="custom-tag"
        >
          {{ tag }}
        </el-tag>
      </div>
      
      <!-- 信息标签组 -->
      <div class="info-tags">
        <el-tooltip v-if="account.group" content="Group" placement="top">
          <el-tag 
            size="small"
            class="info-tag group-tag"
          >
            <el-icon><Folder /></el-icon>
            <span>{{ account.group }}</span>
          </el-tag>
        </el-tooltip>

        <el-tooltip v-if="account.created_at" content="Created" placement="top">
          <el-tag 
            size="small"
            class="info-tag create-tag"
          >
            <el-icon><Calendar /></el-icon>
            <span>{{ formatDate(account.created_at) }}</span>
          </el-tag>
        </el-tooltip>

        <el-tooltip v-if="account.token_expires_at" :content="tokenExpiryTooltip" placement="top">
          <el-tag
            size="small"
            class="info-tag token-tag"
            :type="tokenExpiryType"
          >
            <el-icon><Key /></el-icon>
            <span>{{ formatDate(account.token_expires_at) }}</span>
          </el-tag>
        </el-tooltip>
      </div>
    </div>

    <div class="card-actions">
      <!-- 第一排按钮（6个） -->
      <div class="action-buttons">
        <el-tooltip content="Batch reset team credits" placement="top">
          <el-button
            size="small"
            :icon="Refresh"
            circle
            type="warning"
            plain
            @click="handleBatchResetTeamCredits"
            :loading="isResettingCredits"
          />
        </el-tooltip>

        <el-tooltip content="View billing" placement="top">
          <el-button 
            size="small" 
            :icon="Document"
            circle
            @click="handleGetBilling"
            :loading="isGettingBilling"
          />
        </el-tooltip>

        <el-tooltip content="Auto refill" placement="top">
          <el-button
            size="small"
            :icon="Money"
            circle
            type="warning"
            plain
            @click="handleAutoRefill"
          />
        </el-tooltip>

        <el-tooltip content="Credit history" placement="top">
          <el-button 
            size="small" 
            :icon="TrendCharts"
            circle
            @click="handleShowCreditHistory"
            :loading="isLoadingCreditHistory"
          />
        </el-tooltip>

        <el-tooltip :content="refreshButtonTooltip" placement="top">
          <el-button 
            size="small" 
            :icon="RefreshRight"
            circle
            @click="handleRefreshToken"
            :loading="isRefreshing"
          />
        </el-tooltip>

        <el-tooltip content="Account info" placement="top">
          <el-button 
            size="small" 
            :icon="User"
            circle
            @click="handleAccountInfo"
          />
        </el-tooltip>

        <el-tooltip content="Delete user (Windsurf)" placement="top">
          <el-button
            size="small"
            :icon="UserFilled"
            circle
            type="warning"
            plain
            :loading="deletingUser"
            @click="handleDeleteWindsurfUser"
          />
        </el-tooltip>

        <el-tooltip content="Delete" placement="top">
          <el-button
            size="small"
            :icon="Delete"
            circle
            type="danger"
            plain
            @click="handleDelete"
          />
        </el-tooltip>
      </div>
      
      <!-- Second row of buttons (5) -->
      <div class="action-buttons">
        <el-tooltip content="Edit" placement="top">
          <el-button 
            size="small" 
            :icon="Edit"
            circle
            @click="handleEdit"
          />
        </el-tooltip>

        <el-tooltip content="Re-login" placement="top">
          <el-button 
            size="small" 
            :icon="Key"
            circle
            @click="handleLogin"
          />
        </el-tooltip>

        <el-tooltip content="Usage analytics" placement="top">
          <el-button
            size="small"
            :icon="DataAnalysis"
            circle
            @click="handleShowAnalytics"
          />
        </el-tooltip>

        <el-tooltip content="Team settings" placement="top">
          <el-button
            size="small"
            :icon="Setting"
            circle
            @click="handleTeamSettings"
          />
        </el-tooltip>

        <el-tooltip content="Team management" placement="top">
          <el-button
            size="small"
            :icon="UserFilled"
            circle
            type="primary"
            plain
            @click="handleTeamManagement"
          />
        </el-tooltip>

        <el-tooltip content="Change subscription" placement="top">
          <el-button
            size="small"
            :icon="Sell"
            circle
            @click="handleUpdatePlan"
            :loading="isUpdatingPlan"
          />
        </el-tooltip>

        <el-tooltip content="Get trial link" placement="top">
          <el-button
            size="small"
            :icon="Link"
            circle
            @click="handleGetTrialLink"
            :loading="isGettingTrialLink"
          />
        </el-tooltip>

        <el-tooltip content="Quick switch" placement="top">
          <el-button
            size="small"
            :icon="Switch"
            circle
            type="success"
            plain
            @click="handleSwitchAccount"
            :loading="isSwitching"
          />
        </el-tooltip>
      </div>
    </div>
  </div>

  <!-- 积分记录对话框 -->
  <CreditHistoryDialog
    v-model="showCreditHistoryDialog"
    :account-id="account.id"
  />

  <!-- 座位更新结果对话框 -->
  <UpdateSeatsResultDialog
    v-model="showSeatsResultDialog"
    :result-data="seatsResultData"
  />

  <!-- 使用分析对话框 -->
  <AnalyticsDialog
    v-model="showAnalyticsDialog"
    :account-id="account.id"
    :account-email="account.email"
  />

  <!-- 团队设置对话框 -->
  <TeamSettingsDialog
    v-model="showTeamSettingsDialog"
    :account-id="account.id"
  />

  <!-- 团队管理对话框 -->
  <TeamManagementDialog
    v-model="showTeamManagementDialog"
    :account-id="account.id"
  />

  <!-- 自动充值设置对话框 -->
  <AutoRefillDialog
    v-model="showAutoRefillDialog"
    :account-id="account.id"
  />

  <!-- Change Subscription Dialog -->
  <UpdatePlanDialog
    v-model="showUpdatePlanDialog"
    :account-id="account.id"
    :account="account"
    @success="handleUpdatePlanSuccess"
  />

  <!-- Turnstile Verification Dialog -->
  <TurnstileDialog
    :visible="showTurnstileDialog"
    @update:visible="showTurnstileDialog = $event"
    @success="handleTurnstileSuccess"
    @cancel="showTurnstileDialog = false"
  />

  <!-- Switch Account Progress Popup (Independent Centered Dialog) -->
  <!-- 运行中禁止通过遮罩 / Esc 关闭，强制用户看到全流程；成功/失败时允许关闭 -->
  <el-dialog
    v-model="switchProgress.visible"
    :title="`Switch to ${switchProgress.accountName}`"
    width="460px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    :show-close="switchProgress.phase !== 'running'"
    align-center
    append-to-body
    class="switch-progress-dialog"
    @close="closeSwitchProgress"
  >
    <div class="switch-progress-body">
      <!-- 横向进度条：error → exception(红)，100% 且 success → success(绿)，其它 → 默认 -->
      <el-progress
        :percentage="switchProgress.percent"
        :status="switchProgress.phase === 'error'
          ? 'exception'
          : (switchProgress.phase === 'success' ? 'success' : undefined)"
        :stroke-width="12"
        striped
        :striped-flow="switchProgress.phase === 'running'"
      />
      <!-- Current stage description -->
      <div
        class="switch-progress-label"
        :class="{ 'is-error': switchProgress.phase === 'error' }"
      >
        {{ switchProgress.label || 'Waiting for backend to start...' }}
      </div>
      <!-- 7 步 checklist -->
      <div class="switch-progress-steps">
        <div
          v-for="(step, idx) in SWITCH_STEP_DEFS"
          :key="step.key"
          class="switch-progress-step"
          :class="`status-${getStepStatus(idx)}`"
        >
          <el-icon v-if="getStepStatus(idx) === 'done'" class="step-icon">
            <CircleCheck />
          </el-icon>
          <el-icon v-else-if="getStepStatus(idx) === 'running'" class="step-icon is-spin">
            <Loading />
          </el-icon>
          <el-icon v-else-if="getStepStatus(idx) === 'error'" class="step-icon">
            <CircleClose />
          </el-icon>
          <el-icon v-else class="step-icon">
            <CircleCheck />
          </el-icon>
          <span class="step-label">{{ step.label }}</span>
        </div>
      </div>
    </div>
    <template #footer>
      <!-- running 期间不给任何按钮，强制用户等待后端；非 running 显示关闭 -->
      <el-button
        v-if="switchProgress.phase !== 'running'"
        :type="switchProgress.phase === 'error' ? 'danger' : 'primary'"
        @click="closeSwitchProgress"
      >
        Close
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, reactive, onBeforeUnmount } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  Document,
  RefreshRight,
  Edit,
  Delete,
  Key,
  Clock,
  Calendar,
  Folder,
  User,
  Trophy,
  TrendCharts,
  Link,
  Switch,
  DataAnalysis,
  Setting,
  UserFilled,
  Money,
  Sell,
  Refresh,
  CircleCheck,
  CircleClose,
  Loading,
} from '@element-plus/icons-vue';
import type { Account } from '@/types';
import { apiService, accountApi } from '@/api';
import { useAccountsStore, useUIStore, useSettingsStore } from '@/store';
import UpdateSeatsResultDialog from '@/components/UpdateSeatsResultDialog.vue';
import CreditHistoryDialog from '@/components/CreditHistoryDialog.vue';
import AnalyticsDialog from '@/components/AnalyticsDialog.vue';
import TeamSettingsDialog from '@/components/TeamSettingsDialog.vue';
import TeamManagementDialog from '@/components/TeamManagementDialog.vue';
import AutoRefillDialog from '@/components/AutoRefillDialog.vue';
import UpdatePlanDialog from '@/components/UpdatePlanDialog.vue';
import TurnstileDialog from '@/components/TurnstileDialog.vue';
import dayjs from 'dayjs';
import { maskEmail } from '@/utils/privacy';

const props = defineProps<{
  account: Account;
  isSelected: boolean;
  currentEmail?: string;
}>();

const emit = defineEmits<{
  select: [value: boolean];
  update: [account: Account];
}>();

const accountsStore = useAccountsStore();
const uiStore = useUIStore();
const settingsStore = useSettingsStore();

// 是否为当前激活账号
const isCurrent = computed(() => {
  return props.currentEmail && props.account.email === props.currentEmail;
});

// 显示的邮箱（根据隐私模式）
const displayEmail = computed(() => {
  if (settingsStore.settings?.privacyMode) {
    return maskEmail(props.account.email);
  }
  return props.account.email;
});

// Get tag color (priority given to global tag color)
function getTagColor(tagName: string): string | null {
  // 优先使用全局标签的颜色
  const globalTag = settingsStore.tags.find(t => t.name === tagName);
  if (globalTag?.color) {
    return globalTag.color;
  }
  // 回退到账号保存的颜色
  if (!props.account.tagColors) return null;
  const tagWithColor = props.account.tagColors.find(t => t.name === tagName);
  return tagWithColor?.color || null;
}

// Parse color into RGB values
function parseColor(color: string): { r: number; g: number; b: number; a: number } | null {
  // 解析RGBA格式
  const rgbaMatch = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
  if (rgbaMatch) {
    return {
      r: parseInt(rgbaMatch[1]),
      g: parseInt(rgbaMatch[2]),
      b: parseInt(rgbaMatch[3]),
      a: parseFloat(rgbaMatch[4] || '1')
    };
  }
  
  // 解析HEX格式
  if (color.startsWith('#')) {
    const hex = color.slice(1);
    if (hex.length === 6) {
      return {
        r: parseInt(hex.slice(0, 2), 16),
        g: parseInt(hex.slice(2, 4), 16),
        b: parseInt(hex.slice(4, 6), 16),
        a: 1
      };
    } else if (hex.length === 8) {
      return {
        r: parseInt(hex.slice(0, 2), 16),
        g: parseInt(hex.slice(2, 4), 16),
        b: parseInt(hex.slice(4, 6), 16),
        a: parseInt(hex.slice(6, 8), 16) / 255
      };
    }
  }
  
  return null;
}

// Get tag style
function getTagStyle(tagName: string): Record<string, string> {
  const color = getTagColor(tagName);
  
  // If no color is found, return default style (instead of an empty object)
  if (!color) {
    return {
      backgroundColor: 'rgba(64, 158, 255, 0.1)',
      borderColor: 'rgba(64, 158, 255, 0.3)',
      color: 'rgba(64, 158, 255, 1)',
      border: '1px solid rgba(64, 158, 255, 0.3)'
    };
  }
  
  const parsed = parseColor(color);
  if (parsed) {
    const { r, g, b, a } = parsed;
    // 背景色使用低透明度
    const bgAlpha = Math.min(a * 0.2, 0.3);
    // Use slightly higher transparency for the border
    const borderAlpha = Math.min(a * 0.5, 0.6);
    return {
      backgroundColor: `rgba(${r}, ${g}, ${b}, ${bgAlpha})`,
      borderColor: `rgba(${r}, ${g}, ${b}, ${borderAlpha})`,
      color: `rgba(${r}, ${g}, ${b}, ${Math.max(a, 0.8)})`,
      border: `1px solid rgba(${r}, ${g}, ${b}, ${borderAlpha})`
    };
  }
  
  // If color parsing fails, try using the color value directly
  return {
    backgroundColor: color,
    borderColor: color,
    color: '#fff'
  };
}

// Whether there are colored tags
const hasColoredTag = computed(() => {
  // Check if any of the account's tags have a color (global or account level)
  return props.account.tags.some(tagName => getTagColor(tagName) !== null);
});

// 获取第一个带颜色标签的颜色作为卡片边框色
const primaryTagColor = computed(() => {
  // 遍历账号的标签，找到第一个有颜色的
  for (const tagName of props.account.tags) {
    const color = getTagColor(tagName);
    if (color) return color;
  }
  return null;
});

// Card border style
const cardBorderStyle = computed(() => {
  const color = primaryTagColor.value;
  if (!color) return {};
  
  const parsed = parseColor(color);
  if (parsed) {
    const { r, g, b, a } = parsed;
    // Border transparency
    const borderAlpha = Math.min(a * 0.6, 0.8);
    // 发光效果透明度
    const glowAlpha = Math.min(a * 0.2, 0.3);
    return {
      '--tag-border-color': `rgba(${r}, ${g}, ${b}, ${borderAlpha})`,
      '--tag-glow-color': `rgba(${r}, ${g}, ${b}, ${glowAlpha})`,
      borderColor: `rgba(${r}, ${g}, ${b}, ${borderAlpha})`,
      boxShadow: `0 0 12px rgba(${r}, ${g}, ${b}, ${glowAlpha}), 0 2px 8px rgba(0, 0, 0, 0.06)`
    };
  }
  return {};
});

const isGettingBilling = ref(false);
const isLoadingCreditHistory = ref(false);
const isRefreshing = ref(false);
const isGettingTrialLink = ref(false);
const deletingUser = ref(false);
const isSwitching = ref(false);

// ==================== 切号进度弹窗状态 ====================
// 与后端 switch_account_commands.rs 的 SwitchProgressPayload 对齐
// phase: 'running' | 'success' | 'error' | 'idle'（idle 仅前端用，代表弹窗未激活）
type SwitchProgressPhase = 'idle' | 'running' | 'success' | 'error';

interface SwitchProgressEventPayload {
  step: string;
  label: string;
  percent: number;
  phase: 'running' | 'success' | 'error';
}

interface SwitchProgressState {
  visible: boolean;
  step: string;
  label: string;
  percent: number;
  phase: SwitchProgressPhase;
  accountName: string;
}

// 显式泛型声明，避免 TS 在后续赋值 phase='running' 后做 literal narrowing
// 使得 `switchProgress.phase !== 'error'` 被错判为永远为 true 的比较。
const switchProgress = reactive<SwitchProgressState>({
  visible: false,
  step: '',
  label: '',
  percent: 0,
  phase: 'idle',
  accountName: '',
});

// Step definition: The order is consistent with the step key emitted by the backend
// The frontend looks up the position of the current step in the sequence by key to render the checklist status
const SWITCH_STEP_DEFS: ReadonlyArray<{ key: string; label: string }> = [
  { key: 'preparing', label: 'Preparing account info' },
  { key: 'fetch_access', label: 'Fetching access_token' },
  { key: 'fetch_auth', label: 'Fetching one-time auth_token' },
  { key: 'auto_patch', label: 'Checking seamless switch patch' },
  { key: 'reset_mid', label: 'Resetting machine ID' },
  { key: 'callback', label: 'Triggering client login' },
  { key: 'finalize', label: 'Saving account state' },
];

// 当前步骤索引（-1 表示尚未开始 / 未命中已知 step）
const currentStepIndex = computed(() => {
  if (switchProgress.step === 'done') return SWITCH_STEP_DEFS.length;
  return SWITCH_STEP_DEFS.findIndex(s => s.key === switchProgress.step);
});

// 每一步在 UI 上的状态：已完成 ✓ / 当前 spinner / 失败 ✗ / 待办灰
function getStepStatus(idx: number): 'done' | 'running' | 'error' | 'pending' {
  const cur = currentStepIndex.value;
  if (cur === -1) return 'pending';
  if (idx < cur) return 'done';
  if (idx === cur) {
    if (switchProgress.phase === 'error') return 'error';
    if (switchProgress.phase === 'success') return 'done';
    return 'running';
  }
  return 'pending';
}

// Tauri 事件监听句柄；每次点击切号时新建，弹窗关闭或卸载时释放
let switchProgressUnlisten: UnlistenFn | null = null;

// 卸载组件时必须释放 listener，避免"路由切换 → 组件销毁 → 事件继续触发"导致内存泄漏
onBeforeUnmount(async () => {
  if (switchProgressUnlisten) {
    switchProgressUnlisten();
    switchProgressUnlisten = null;
  }
});

// 手动关闭弹窗（仅在非 running 阶段允许）
function closeSwitchProgress() {
  if (switchProgress.phase === 'running') return;
  switchProgress.visible = false;
  switchProgress.phase = 'idle';
  if (switchProgressUnlisten) {
    switchProgressUnlisten();
    switchProgressUnlisten = null;
  }
}

const isResettingCredits = ref(false);
const isUpdatingPlan = ref(false);
const billingData = ref<any>(null);
const showCreditHistoryDialog = ref(false);
const showSeatsResultDialog = ref(false);
const showAnalyticsDialog = ref(false);
const showTeamSettingsDialog = ref(false);
const showTeamManagementDialog = ref(false);
const showAutoRefillDialog = ref(false);
const showUpdatePlanDialog = ref(false);
const showTurnstileDialog = ref(false);
const pendingTurnstileToken = ref('');
const seatsResultData = ref<any>(null);

// Determine if it is a paid plan (not Free)
const isPaidPlan = computed(() => {
  const planName = props.account.plan_name?.toLowerCase();
  return planName && planName !== 'free';
});

const statusClass = computed(() => {
  // 只有付费计划且 subscription_active 为 false 时才显示未激活
  if (isPaidPlan.value && props.account.subscription_active === false) return 'status-subscription-inactive';
  // Check account disabled status
  if (props.account.is_disabled) return 'status-disabled';
  if (props.account.status === 'active') return 'status-active';
  if (props.account.status === 'inactive') return 'status-inactive';
  return 'status-error';
});

const statusText = computed(() => {
  // 只有付费计划且 subscription_active 为 false 时才显示未激活
  if (isPaidPlan.value && props.account.subscription_active === false) return 'Inactive';
  // Check account disabled status
  if (props.account.is_disabled) return 'Disabled';
  if (props.account.status === 'active') return 'Active';
  if (props.account.status === 'inactive') return 'Offline';
  return 'Error';
});

// Whether it is in quota percentage mode (billing_strategy === 2, i.e., QUOTA)
const isQuotaMode = computed(() => props.account.billing_strategy === 2);

// 是否有配额数据可展示（QUOTA 模式或 CREDITS 模式）
const hasQuotaData = computed(() => {
  if (isQuotaMode.value) {
    return props.account.daily_quota_remaining_percent !== undefined 
        || props.account.weekly_quota_remaining_percent !== undefined;
  }
  return !!props.account.total_quota;
});

// 配额百分比（仅 CREDITS 模式使用，表示已用占比）
const quotaPercentage = computed(() => {
  if (!props.account.total_quota || !props.account.used_quota) return 0;
  return Math.min(Math.round((props.account.used_quota / props.account.total_quota) * 100), 100);
});

// 配额颜色（仅 CREDITS 模式使用）
const quotaColor = computed(() => {
  const percentage = quotaPercentage.value;
  if (percentage < 50) return '#10b981';  // 绿色
  if (percentage < 80) return '#f59e0b';  // 橙色
  return '#ef4444';  // 红色
});

// 日配额剩余百分比的颜色（QUOTA 模式）
const dailyQuotaColor = computed(() => {
  const remaining = props.account.daily_quota_remaining_percent ?? 0;
  if (remaining > 50) return '#10b981';
  if (remaining > 20) return '#f59e0b';
  return '#ef4444';
});

// Color for remaining weekly quota percentage (QUOTA mode)
const weeklyQuotaColor = computed(() => {
  const remaining = props.account.weekly_quota_remaining_percent ?? 0;
  if (remaining > 50) return '#10b981';
  if (remaining > 20) return '#f59e0b';
  return '#ef4444';
});

// 格式化配额重置时间
function formatResetTime(unixTimestamp: number | undefined): string {
  if (!unixTimestamp || unixTimestamp <= 0) return '';
  return dayjs.unix(unixTimestamp).format('MM-DD HH:mm');
}

// 刷新按钮提示文本
const refreshButtonTooltip = computed(() => {
  if (!props.account.token_expires_at) {
    return 'Refresh Token';
  }
  const isExpired = dayjs(props.account.token_expires_at).isBefore(dayjs());
  return isExpired ? 'Refresh Token (Expired)' : 'Refresh account info (Token valid)';
});

// 订阅到期日期格式化
const formattedExpiryDate = computed(() => {
  if (!props.account.subscription_expires_at) return '';
  return dayjs(props.account.subscription_expires_at).format('YYYY-MM-DD HH:mm');
});

// Calculate days until expiry
const daysUntilExpiry = computed(() => {
  if (!props.account.subscription_expires_at) return null;
  const now = dayjs();
  const expiry = dayjs(props.account.subscription_expires_at);
  return expiry.diff(now, 'day');
});

// 到期文本
const expiryText = computed(() => {
  const days = daysUntilExpiry.value;
  if (days === null) return '';
  if (days < 0) return 'Expired';
  if (days === 0) return 'Expires today';
  if (days === 1) return 'Expires tomorrow';
  if (days <= 7) return `Expires in ${days} days`;
  return `${days} days left`;
});

// 到期样式类
const expiryClass = computed(() => {
  const days = daysUntilExpiry.value;
  if (days === null) return '';
  if (days < 0) return 'expired';
  if (days <= 7) return 'expiring-soon';
  return 'normal';
});

// Token过期类型
const tokenExpiryType = computed(() => {
  if (!props.account.token_expires_at) return 'info';
  const expiry = dayjs(props.account.token_expires_at);
  const now = dayjs();
  const minutesUntilExpiry = expiry.diff(now, 'minutes');
  const hoursUntilExpiry = expiry.diff(now, 'hours');
  
  if (minutesUntilExpiry < 0) return 'danger';
  if (minutesUntilExpiry < 60) return 'warning';  // 小于1小时显示警告
  if (hoursUntilExpiry <= 24) return 'warning';   // 小于24小时显示警告
  return 'success';
});

// Token过期提示
const tokenExpiryTooltip = computed(() => {
  if (!props.account.token_expires_at) return 'No Token';
  const expiry = dayjs(props.account.token_expires_at);
  const now = dayjs();
  const minutesUntilExpiry = expiry.diff(now, 'minutes');
  const hoursUntilExpiry = expiry.diff(now, 'hours');
  const daysUntilExpiry = expiry.diff(now, 'days');
  
  if (minutesUntilExpiry < 0) return 'Expired';
  if (minutesUntilExpiry === 0) return 'Expiring soon (<1min)';
  if (minutesUntilExpiry < 5) return `Expiring in ${minutesUntilExpiry} min`;
  if (minutesUntilExpiry < 60) return `Expires in ${minutesUntilExpiry} min`;
  if (hoursUntilExpiry < 24) return `Expires in ${hoursUntilExpiry} hours`;
  if (daysUntilExpiry <= 7) return `Expires in ${daysUntilExpiry} days`;
  return `Valid (${daysUntilExpiry} days)`;
});

function formatDate(date: string) {
  return dayjs(date).format('MM-DD HH:mm');
}

// 格式化配额（除以100并显示两位小数）
function formatQuota(num: number | undefined | null) {
  if (!num) return '0.00';
  return (num / 100).toFixed(2);
}

function handleSelect(value: boolean) {
  emit('select', value);
}

// 点击卡片空白区域触发选择
function handleCardClick(event: MouseEvent) {
  const target = event.target as HTMLElement;
  
  // 检查是否点击了交互元素
  const isInteractive = target.closest('button, a, input, .el-checkbox, .el-button, .el-tag, .el-tooltip, .el-icon');
  
  if (!isInteractive) {
    // 点击空白区域，切换选择状态
    emit('select', !props.isSelected);
  }
}

async function copyEmail() {
  try {
    await navigator.clipboard.writeText(props.account.email);
    ElMessage.success('Email copied');
  } catch (error) {
    ElMessage.error('Copy failed');
  }
}

async function handleGetBilling() {
  isGettingBilling.value = true;
  try {
    const result = await apiService.getBilling(props.account.id);
    if (result.success) {
      // 显示账单对话框而不是仅仅提示
      uiStore.openBillingDialog(props.account.id);
      // 把账单数据传递给对话框（可以通过store或事件）
      billingData.value = result;
    } else {
      ElMessage.error('Failed to query billing');
    }
  } catch (error) {
    ElMessage.error(`Operation failed: ${error}`);
  } finally {
    isGettingBilling.value = false;
  }
}

async function handleShowCreditHistory() {
  showCreditHistoryDialog.value = true;
}

async function handleRefreshToken() {
  isRefreshing.value = true;
  try {
    // 注：Devin 账号与 Firebase 账号统一走 apiService.refreshToken()，后端已按
    // auth_provider 自动分流（Devin 走 auth1_token 换新 session_token + enrich，
    // Firebase 走 refresh_token/sign_in），并统一响应 use_lightweight_api 设置、
    // 写 OperationLog。无需在前端额外区分。

    // Check if the token is expired
    const isTokenExpired = !props.account.token_expires_at || 
                          dayjs(props.account.token_expires_at).isBefore(dayjs());
    
    if (isTokenExpired) {
      // Token has expired, perform token refresh operation
      const result = await apiService.refreshToken(props.account.id);
      if (result.success) {
        // Show more detailed success message
        const message = result.message || 'Token refreshed successfully';
        if (result.old_expires_at && result.old_expires_at !== 'unknown') {
          ElMessage.success({
            message: `${message}\nOld expiry: ${new Date(result.old_expires_at).toLocaleString()}\nNew expiry: ${result.expires_at ? new Date(result.expires_at).toLocaleString() : 'unknown'}`,
            duration: 3000,
            showClose: true
          });
        } else {
          ElMessage.success(message);
        }
        // Token刷新成功后，重新获取账号信息（包括新的token）
        try {
          const updatedAccount = await accountApi.getAccount(props.account.id);
          
          // 合并额外的信息
          if (result.token) {
            updatedAccount.token = result.token;
          }
          if (result.expires_at) {
            updatedAccount.token_expires_at = result.expires_at;
          }
          if (result.plan_name) {
            updatedAccount.plan_name = result.plan_name;
          }
          if (result.used_quota !== undefined) {
            updatedAccount.used_quota = result.used_quota;
          }
          if (result.total_quota !== undefined) {
            updatedAccount.total_quota = result.total_quota;
          }
          if (result.subscription_expires_at) {
            updatedAccount.subscription_expires_at = result.subscription_expires_at;
          }
          if (result.is_disabled !== undefined) {
            updatedAccount.is_disabled = result.is_disabled;
          }
          
          // 直接更新store中的账号数据，确保立即同步
          await accountsStore.updateAccount(updatedAccount);
          emit('update', updatedAccount);
        } catch (error) {
          console.error('获取账号信息失败:', error);
          // 如果无法获取最新账号信息，使用现有数据更新
          const updatedAccount = { ...props.account, status: 'active' as const };
          if (result.token) {
            updatedAccount.token = result.token;
          }
          if (result.expires_at) {
            updatedAccount.token_expires_at = result.expires_at;
          }
          if (result.is_disabled !== undefined) {
            updatedAccount.is_disabled = result.is_disabled;
          }
          await accountsStore.updateAccount(updatedAccount);
          emit('update', updatedAccount);
        }
      } else {
        ElMessage.error('Token refresh failed');
      }
    } else {
      // Token仍然有效，只刷新账号信息
      const result = await apiService.getCurrentUser(props.account.id);
      if (result.success && result.user_info) {
        ElMessage.success({
          message: `Account info updated\nToken valid until: ${new Date(props.account.token_expires_at!).toLocaleString()}`,
          duration: 2500,
          showClose: true
        });
        
        // 更新账号信息
        const updatedAccount = { ...props.account, status: 'active' as const };
        
        // 更新用户基本信息（包含api_key和禁用状态）
        if (result.user_info.user?.api_key) {
          updatedAccount.windsurf_api_key = result.user_info.user.api_key;
        }
        // 更新账户禁用状态
        if (result.user_info.user?.disable_codeium !== undefined) {
          updatedAccount.is_disabled = result.user_info.user.disable_codeium;
        }
        
        // Update plan information
        if (result.user_info.plan?.plan_name) {
          updatedAccount.plan_name = result.user_info.plan.plan_name;
        }
        // 从 plan 中读取 billing_strategy
        if (result.user_info.plan?.billing_strategy !== undefined) {
          updatedAccount.billing_strategy = result.user_info.plan.billing_strategy;
        }
        
        // 合并 plan_status 中的新配额字段（避免覆盖后端已保存的数据）
        if (result.plan_status) {
          if (result.plan_status.billing_strategy !== undefined) {
            updatedAccount.billing_strategy = result.plan_status.billing_strategy;
          }
          if (result.plan_status.daily_quota_remaining_percent !== undefined) {
            updatedAccount.daily_quota_remaining_percent = result.plan_status.daily_quota_remaining_percent;
          }
          if (result.plan_status.weekly_quota_remaining_percent !== undefined) {
            updatedAccount.weekly_quota_remaining_percent = result.plan_status.weekly_quota_remaining_percent;
          }
          if (result.plan_status.daily_quota_reset_at_unix !== undefined) {
            updatedAccount.daily_quota_reset_at_unix = result.plan_status.daily_quota_reset_at_unix;
          }
          if (result.plan_status.weekly_quota_reset_at_unix !== undefined) {
            updatedAccount.weekly_quota_reset_at_unix = result.plan_status.weekly_quota_reset_at_unix;
          }
          if (result.plan_status.overage_balance_micros !== undefined) {
            updatedAccount.overage_balance_micros = result.plan_status.overage_balance_micros;
          }
        }
        
        // 更新配额信息
        if (result.user_info.subscription) {
          if (result.user_info.subscription.used_quota !== undefined) {
            updatedAccount.used_quota = result.user_info.subscription.used_quota;
          }
          if (result.user_info.subscription.quota !== undefined) {
            updatedAccount.total_quota = result.user_info.subscription.quota;
          }
          if (result.user_info.subscription.expires_at) {
            const expiresTimestamp = result.user_info.subscription.expires_at;
            const expiresDate = dayjs.unix(expiresTimestamp);
            updatedAccount.subscription_expires_at = expiresDate.toISOString();
          }
          // 更新订阅激活状态
          if (result.user_info.subscription.subscription_active !== undefined) {
            updatedAccount.subscription_active = result.user_info.subscription.subscription_active;
          }
        }
        
        updatedAccount.last_quota_update = dayjs().toISOString();
        // 保存到后端数据库
        await accountsStore.updateAccount(updatedAccount);
        emit('update', updatedAccount);
      } else {
        // 显示详细错误信息
        const statusCode = result.status_code;
        const errorMsg = result.error || 'Unknown error';
        if (statusCode === 401) {
          // Token 实际已失效，尝试自动刷新
          console.log('[AccountCard] Token expired (401), attempting auto-refresh...');
          const refreshResult = await apiService.refreshToken(props.account.id);
          if (refreshResult.success) {
            ElMessage.success({
              message: 'Token auto-refreshed, please retry operation',
              duration: 3000,
              showClose: true
            });
// Update account information
            const updatedAccount = { ...props.account, status: 'active' as const };
            if (refreshResult.token) {
              updatedAccount.token = refreshResult.token;
            }
            if (refreshResult.expires_at) {
              updatedAccount.token_expires_at = refreshResult.expires_at;
            }
            await accountsStore.updateAccount(updatedAccount);
            emit('update', updatedAccount);
          } else {
            ElMessage.error({
              message: `Token expired and refresh failed\nMay need to re-login`,
              duration: 5000,
              showClose: true
            });
            // 更新账户状态为错误
            const errorAccount = { ...props.account, status: 'error' as const };
            await accountsStore.updateAccount(errorAccount);
            emit('update', errorAccount);
          }
        } else {
          ElMessage.error({
            message: `Failed to get account info (${statusCode || 'unknown'})\n${errorMsg}`,
            duration: 5000,
            showClose: true
          });
          // 更新账户状态为错误
          const errorAccount = { ...props.account, status: 'error' as const };
          await accountsStore.updateAccount(errorAccount);
          emit('update', errorAccount);
        }
      }
    }
  } catch (error) {
    ElMessage.error(`Operation failed: ${error}`);
  } finally {
    isRefreshing.value = false;
  }
}

async function handleLogin() {
  try {
    const result = await apiService.loginAccount(props.account.id);
    if (result.success) {
      ElMessage.success('Login successful');
      
      // 重新从后端获取完整的账号数据（包括新的 token 和 refresh_token）
      try {
        const updatedAccount = await accountApi.getAccount(props.account.id);
        
        // 合并返回的额外信息
        if (result.expires_at) {
          updatedAccount.token_expires_at = result.expires_at;
        }
        if (result.plan_name) {
          updatedAccount.plan_name = result.plan_name;
        }
        if (result.used_quota !== undefined) {
          updatedAccount.used_quota = result.used_quota;
        }
        if (result.total_quota !== undefined) {
          updatedAccount.total_quota = result.total_quota;
        }
        if (result.subscription_expires_at) {
          updatedAccount.subscription_expires_at = result.subscription_expires_at;
        }
        if (result.is_disabled !== undefined) {
          updatedAccount.is_disabled = result.is_disabled;
        }
        updatedAccount.last_quota_update = dayjs().toISOString();
        
        // Update account data in the store
        await accountsStore.updateAccount(updatedAccount);
        emit('update', updatedAccount);
      } catch (error) {
        console.error('获取账号信息失败:', error);
        // 如果获取失败，使用基本更新
        const updatedAccount = { 
          ...props.account, 
          status: 'active' as const,
          token_expires_at: result.expires_at,
          last_login_at: dayjs().toISOString()
        };
        if (result.plan_name) updatedAccount.plan_name = result.plan_name;
        if (result.used_quota !== undefined) updatedAccount.used_quota = result.used_quota;
        if (result.total_quota !== undefined) updatedAccount.total_quota = result.total_quota;
        if (result.subscription_expires_at) updatedAccount.subscription_expires_at = result.subscription_expires_at;
        emit('update', updatedAccount);
      }
    } else {
      ElMessage.error('Login failed');
    }
  } catch (error) {
    ElMessage.error(`Login failed: ${error}`);
  }
}

function handleEdit() {
  uiStore.openEditAccountDialog(props.account.id);
}

function handleAccountInfo() {
  // Directly open the account information dialog
  uiStore.openAccountInfoDialog(props.account.id);
}

function handleShowAnalytics() {
  // 显示使用分析对话框
  showAnalyticsDialog.value = true;
}

function handleTeamSettings() {
  // Show team settings dialog
  showTeamSettingsDialog.value = true;
}

function handleTeamManagement() {
  showTeamManagementDialog.value = true;
}

function handleAutoRefill() {
  // Show auto-refill settings dialog
  showAutoRefillDialog.value = true;
}

// 批量重置团队成员积分
async function handleBatchResetTeamCredits() {
  isResettingCredits.value = true;
  try {
    // Step 1: Get team member list
    const membersResult = await invoke<any>('get_team_members', {
      id: props.account.id,
      groupId: null
    });
    
    if (!membersResult.success) {
      ElMessage.error(membersResult.error || 'Failed to get team members');
      return;
    }
    
    const data = membersResult.data || {};
    let users = data.subMesssage_1 || [];
    if (users && !Array.isArray(users)) {
      users = [users];
    }
    let userRoles = data.subMesssage_2 || [];
    if (userRoles && !Array.isArray(userRoles)) {
      userRoles = [userRoles];
    }
    
    // 构建成员列表并排除自己
    interface TeamMember {
      api_key: string;
      name: string;
      email: string;
    }
    const otherMembers: TeamMember[] = [];
    const currentEmail = props.account.email?.toLowerCase();
    
    for (const user of users) {
      const apiKey = user.string_1 || '';
      const name = user.string_2 || '';
      const email = user.string_3 || '';
      const teamStatus = user.int_8 || 0;
      
      // Only add approved members and exclude self (current account)
      if (teamStatus !== 1 && email.toLowerCase() !== currentEmail) {
        otherMembers.push({ api_key: apiKey, name, email });
      }
    }
    
    if (otherMembers.length === 0) {
      ElMessage.warning('No team members to reset');
      return;
    }
    
    // Confirm operation
    try {
      await ElMessageBox.confirm(
        `确定要重置 ${otherMembers.length} 位团队成员的积分吗？\n此操作将移除并重新邀请这些成员。`,
        '批量重置团队积分',
        {
          confirmButtonText: '确定',
          cancelButtonText: '取消',
          type: 'warning'
        }
      );
    } catch {
      return;
    }
    
    let successCount = 0;
    let failCount = 0;
    
    // Step 2: 遍历所有其他成员执行重置
    for (const member of otherMembers) {
      try {
        // 移除成员
        const removeResult = await invoke<any>('remove_team_member', {
          id: props.account.id,
          memberApiKey: member.api_key
        });
        
        if (!removeResult.success) {
          console.error(`移除成员 ${member.name} 失败:`, removeResult.error);
          failCount++;
          continue;
        }
        
        // Re-invite
        const inviteResult = await invoke<any>('invite_team_members', {
          id: props.account.id,
          users: [{ name: member.name, email: member.email }]
        });
        
        if (!inviteResult.success) {
          console.error(`邀请成员 ${member.name} 失败:`, inviteResult.error);
          failCount++;
          continue;
        }
        
        // 自动接受邀请（如果邮箱在管理器中）
        try {
          const accounts = await invoke<any[]>('get_all_accounts');
          const matchedAccount = accounts.find((acc: any) => 
            acc.email?.toLowerCase() === member.email.toLowerCase()
          );
          
          if (matchedAccount) {
            await invoke<any>('accept_invitation', {
              id: matchedAccount.id,
              approvalId: ''
            });
          }
        } catch (e) {
          console.log(`Auto-accept invitation failed (${member.email}):`, e);
        }
        
        successCount++;
      } catch (error) {
        console.error(`处理成员 ${member.name} 时出错:`, error);
        failCount++;
      }
    }
    
    // 显示结果
    if (failCount === 0) {
      ElMessage.success(`Successfully reset credits for ${successCount} team members`);
    } else {
      ElMessage.warning(`Reset complete: ${successCount} succeeded, ${failCount} failed`);
    }
    
    // 刷新账户信息
    accountsStore.refreshAccountToken(props.account);
  } catch (error: any) {
    ElMessage.error('Batch reset team credits failed: ' + error.toString());
  } finally {
    isResettingCredits.value = false;
  }
}

// 更换订阅
function handleUpdatePlan() {
  showUpdatePlanDialog.value = true;
}

// 更换订阅成功回调
function handleUpdatePlanSuccess() {
  accountsStore.refreshAccountToken(props.account);
}

// 获取试用绑卡链接 - 所有 trial 签约都需要 Turnstile 验证
//
// Windsurf 后端 `SubscribeToPlan` 对 `start_trial=true` 的所有计划（Pro/Max/Teams/Devin…）
// 都强制 captcha 校验（错误码 `failed_precondition` + `captcha required for trial signup`）。
// 早期只有 Pro 走 trial、代码里按 tier 分流弹 Turnstile 是历史遗留，现统一成：
// - `startTrial=true` → 必须先过 Turnstile 拿 token 再请求
// - `startTrial=false`（直接付费签约）→ 后端不要求 captcha，直接发请求
function handleGetTrialLink() {
  const startTrial = settingsStore.settings?.startTrial ?? true;

  if (startTrial) {
    showTurnstileDialog.value = true;
  } else {
    // Direct paid signup, no captcha required
    handleTurnstileSuccess('');
  }
}

// Turnstile 验证成功后的处理
async function handleTurnstileSuccess(turnstileToken: string) {
  pendingTurnstileToken.value = turnstileToken;
  showTurnstileDialog.value = false;
  
  isGettingTrialLink.value = true;
  try {
    // Read subscription parameters from settings
    const teamsTier = settingsStore.settings?.subscriptionPlan ?? 2; // 默认 Pro
    const paymentPeriod = settingsStore.settings?.paymentPeriod ?? 1; // 默认月付
    // 团队/企业类计划需要团队名称，个人计划不设置
    const teamTiers = [1, 3, 4, 5, 7, 10, 11, 12, 14, 15];
    const teamName = teamTiers.includes(teamsTier) ? (settingsStore.settings?.teamName || undefined) : undefined;
    const seatCount = settingsStore.settings?.seatCount ?? 1;
    const startTrial = settingsStore.settings?.startTrial ?? true;
    
    // 检查是否启用了自动打开支付页面
    const autoOpen = settingsStore.settings?.autoOpenPaymentLinkInWebview || false;
    const autoFill = settingsStore.settings?.autoFillPaymentForm || false;
    const autoSubmit = settingsStore.settings?.autoSubmitPaymentForm || false;
    
    // 如果启用了自动打开，使用增强的API
    if (autoOpen) {
      // Get the latest account data from the store to ensure the token is up-to-date
      const latestAccount = accountsStore.accounts.find(a => a.id === props.account.id);
      const account = latestAccount || props.account;
      
      if (!account.token) {
        ElMessage.warning('Please refresh token first');
        isGettingTrialLink.value = false;
        return;
      }
      
      // Use the enhanced payment API
      const { getTrialPaymentLink, autoFillPaymentForm } = await import('@/utils/cardGenerator');
      
      const result = await getTrialPaymentLink(
        account.nickname || account.email,
        account.token,
        true, // 自动打开窗口
        teamsTier,
        paymentPeriod,
        startTrial,
        teamName,
        teamTiers.includes(teamsTier) ? seatCount : undefined, // Team/Enterprise plans require seats
        turnstileToken || undefined // trial 签约时所有计划均需 Turnstile token
      );
      
      if (result.success && result.window_opened) {
        ElMessage.success('Payment window opened in Chrome incognito mode');
        
        // 如果启用了自动填写表单
        if (autoFill && result.virtual_card && result.window_label) {
          // 立即注入填写脚本（只等待1秒让窗口加载）
          setTimeout(async () => {
            try {
              console.log('开始自动填写表单，窗口标签:', result.window_label);
              
              // Inject form filling code via Tauri command
              await autoFillPaymentForm(result.window_label, result.virtual_card);
              
              ElMessage.success('Auto-filling virtual card info...');
              
              // If auto-submit is enabled, inject the auto-submit script
              if (autoSubmit) {
                console.log('准备自动提交表单...');
                await invoke('inject_auto_submit_script', { 
                  windowLabel: result.window_label 
                });
                ElMessage.warning('Auto-submit enabled, please observe payment flow');
              }
              
              // 根据设置决定是否显示虚拟卡信息
              const showCardInfo = settingsStore.settings?.showVirtualCardInfo || false;
              if (showCardInfo) {
                ElMessageBox.alert(
                  `<div style="text-align: left; font-family: monospace;">
                    <p><strong>卡号:</strong> ${result.virtual_card.card_number}</p>
                    <p><strong>有效期:</strong> ${result.virtual_card.expiry_date}</p>
                    <p><strong>CVC:</strong> ${result.virtual_card.cvv}</p>
                    <p><strong>姓名:</strong> ${result.virtual_card.cardholder_name}</p>
                    <p><strong>地址:</strong> ${result.virtual_card.billing_address.street_address}</p>
                    <p><strong>城市:</strong> ${result.virtual_card.billing_address.city}, ${result.virtual_card.billing_address.state} ${result.virtual_card.billing_address.postal_code}</p>
                  </div>`,
                  '虚拟卡信息（仅用于测试）',
                  {
                    dangerouslyUseHTMLString: true,
                    confirmButtonText: '确定',
                    type: 'warning',
                  }
                );
              }
            } catch (fillError) {
              console.error('自动填写表单失败:', fillError);
            }
          }, 1000); // 固定1秒延迟，不再使用pageDelay
        }
      } else {
        ElMessage.error(result.error || 'Failed to open payment window');
      }
    } else {
      // 使用原有的API
      const result = await apiService.getTrialPaymentLink(
        props.account.id, 
        teamsTier,
        paymentPeriod,
        startTrial,
        teamName,
        teamTiers.includes(teamsTier) ? seatCount : undefined,
        turnstileToken || undefined
      );

      if (result.success && result.stripe_url) {
        // 复制链接到剪贴板
        try {
          await navigator.clipboard.writeText(result.stripe_url);
          ElMessage.success('Stripe payment link copied to clipboard');

          // 获取设置
          const autoOpen = settingsStore.settings?.autoOpenBrowser ?? true;
          const browserMode = settingsStore.settings?.browserMode ?? 'incognito';
          const isIncognito = browserMode === 'incognito';
          const modeText = isIncognito ? 'incognito mode' : 'normal mode';
          const openCommand = isIncognito ? 'open_external_link_incognito' : 'open_external_link';

          if (autoOpen) {
            // 自动打开浏览器
            try {
              await invoke(openCommand, { url: result.stripe_url });
              ElMessage.success(`已在浏览器${modeText}中打开`);
            } catch (err) {
              ElMessage.error('Failed to open browser, please open link manually');
              console.error('打开链接失败:', err);
            }
          } else {
            // 询问是否在浏览器中打开
            ElMessageBox.confirm(
              `Link copied to clipboard, open in browser ${modeText}?`,
              '打开链接',
              {
                confirmButtonText: '打开',
                cancelButtonText: '取消',
                type: 'info',
              }
            ).then(async () => {
              try {
                await invoke(openCommand, { url: result.stripe_url });
                ElMessage.success(`已在浏览器${modeText}中打开`);
              } catch (err) {
                ElMessage.error('Failed to open browser, please open link manually');
                console.error('打开链接失败:', err);
              }
            }).catch(() => {
              // 用户取消，不做任何操作
            });
          }
        } catch (clipboardError) {
          // 如果复制失败，直接显示链接
          const browserMode = settingsStore.settings?.browserMode ?? 'incognito';
          const isIncognito = browserMode === 'incognito';
          const modeText = isIncognito ? 'incognito mode' : 'normal mode';
          const openCommand = isIncognito ? 'open_external_link_incognito' : 'open_external_link';
          
          ElMessageBox.alert(
            `<div style="word-break: break-all;">${result.stripe_url}</div>`,
            'Stripe Payment Link',
            {
              dangerouslyUseHTMLString: true,
              confirmButtonText: `Open in ${modeText}`,
            }
          ).then(async () => {
            try {
              await invoke(openCommand, { url: result.stripe_url });
              ElMessage.success(`已在浏览器${modeText}中打开`);
            } catch (err) {
              ElMessage.error('Failed to open browser, please open link manually');
              console.error('打开链接失败:', err);
            }
          });
        }
      } else {
        ElMessage.error(result.error || 'Failed to get payment link');
      }
    }
  } catch (error) {
    ElMessage.error(`Failed to get payment link: ${error}`);
  } finally {
    isGettingTrialLink.value = false;
  }
}

async function handleDelete() {
  try {
    await ElMessageBox.confirm(
      `Delete account ${props.account.nickname} (${props.account.email})?`,
      'Confirm Delete',
      {
        confirmButtonText: '删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );
    
    await accountsStore.deleteAccount(props.account.id);
    ElMessage.success('Account deleted successfully');
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`Delete failed: ${error}`);
    }
  }
}

async function handleDeleteWindsurfUser() {
  try {
    await ElMessageBox.confirm(
      `确定要删除 Windsurf 用户 ${props.account.email} 吗？\n\n⚠️ 此操作将从 Windsurf 服务器上删除该用户账号！`,
      'Delete Windsurf User',
      {
        confirmButtonText: 'Confirm Delete',
        cancelButtonText: '取消',
        type: 'error',
      }
    );
    
    deletingUser.value = true;
    const result = await invoke('delete_windsurf_user', { id: props.account.id }) as any;
    
    if (result.success) {
      ElMessage.success('Windsurf 用户已删除');
    } else {
      ElMessage.error(result.error || '删除失败');
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`Delete failed: ${error}`);
    }
  } finally {
    deletingUser.value = false;
  }
}

async function handleSwitchAccount() {
  const isSeamless = settingsStore.settings?.seamlessSwitchEnabled === true;
  const displayName = props.account.nickname || props.account.email;
  
  const confirmMessage = isSeamless
    ? `确定要无感切换到账号 ${displayName} 吗？\n\n此操作将：\n• 自动登录并重置机器ID\n• 无需重启客户端\n• 保持当前工作状态`
    : `确定要切换到账号 ${displayName} 吗？\n\n此操作将：\n• 自动检测客户端路径并启用无感换号\n• 自动登录并重置机器ID`;

  // 先处理确认：用户取消直接返回，不触碰进度状态
  try {
    await ElMessageBox.confirm(
      confirmMessage,
      isSeamless ? '无感切号' : '切换账号确认',
      {
        confirmButtonText: '确定切换',
        cancelButtonText: '取消',
        type: 'info',
      }
    );
  } catch {
    return; // 用户取消
  }

  // 初始化进度弹窗状态（running 态，percent=0 等待后端第一次 emit）
  switchProgress.visible = true;
  switchProgress.accountName = displayName;
  switchProgress.step = '';
  switchProgress.label = '等待后端开始...';
  switchProgress.percent = 0;
  switchProgress.phase = 'running';

  // 注册 Tauri event listener —— 必须在 invoke 之前，否则会错过后端首个 5% "preparing" 事件
  //
  // 防御性释放旧 listener：防止上一次切号异常退出时遗留下来的 handler 再次触发。
  // 注：后端 emit 是全局广播，所有正在监听的 AccountCard 都会收到同一事件；
  // 但因为我们只在点击切号的瞬间注册、完成后立即注销，正常使用不会出现双监听。
  if (switchProgressUnlisten) {
    switchProgressUnlisten();
    switchProgressUnlisten = null;
  }
  switchProgressUnlisten = await listen<SwitchProgressEventPayload>('switch-progress', (e) => {
    const p = e.payload;
    switchProgress.step = p.step;
    switchProgress.label = p.label;
    switchProgress.percent = p.percent;
    switchProgress.phase = p.phase;
  });

  isSwitching.value = true;
  try {
    const result = await apiService.switchAccount(props.account.id);

    if (result.success) {
      // 兜底同步最终态：若事件晚于 invoke 返回未抵达，手动置为 success
      // 类型断言绕开 TS 对 Vue reactive 属性的 literal narrowing（赋值后 TS 错认为 phase 永远为 'running'）
      const currentPhase = switchProgress.phase as SwitchProgressPhase;
      if (currentPhase !== 'error') {
        switchProgress.percent = 100;
        switchProgress.phase = 'success';
        switchProgress.step = 'done';
        switchProgress.label = result.message || '切换完成';
      }

      ElMessage.success({
        message: result.message || '已成功切换账号',
        duration: 5000,
        showClose: true
      });

      if (result.auto_enabled_seamless) {
        await settingsStore.loadSettings();
        ElMessage.info({
          message: '已自动启用无感换号，后续切号将更加流畅',
          duration: 5000,
          showClose: true
        });
      }

      if (!result.seamless_patch_active) {
        if (result.machine_id_reset === false) {
          ElMessage.warning({
            message: '提示：机器ID未重置（可能需要管理员权限），但账号切换已成功',
            duration: 6000,
            showClose: true
          });
        }
        if (result.auth_token) {
          ElMessage.info({
            message: '如果客户端未自动登录，请确保客户端已打开',
            duration: 5000,
            showClose: true
          });
        }
      }

      const updatedAccount = {
        ...props.account,
        status: 'active' as const,
        last_login_at: dayjs().toISOString()
      };
      emit('update', updatedAccount);

      // 成功后延迟自动关闭进度弹窗，给用户一个可感知的"完成"反馈
      setTimeout(() => {
        if (switchProgress.phase === 'success') {
          closeSwitchProgress();
        }
      }, 1200);
    } else {
      // 业务失败（success=false）：保持弹窗打开、切到 error 态，让用户看到失败点后手动关闭
      const currentPhase = switchProgress.phase as SwitchProgressPhase;
      if (currentPhase !== 'error') {
        switchProgress.phase = 'error';
        switchProgress.label = result.error || '切换账号失败';
      }
      ElMessage.error(result.error || '切换账号失败');
    }
  } catch (error) {
    // invoke 级别异常（很少见），同样反馈到弹窗上
    if (error !== 'cancel') {
      switchProgress.phase = 'error';
      switchProgress.label = `切换账号失败: ${error}`;
      ElMessage.error(`切换账号失败: ${error}`);
    }
  } finally {
    isSwitching.value = false;
  }
}

</script>

<style scoped>
.account-card {
  background: white;
  border-radius: 12px;
  padding: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border: 2px solid transparent;
  width: 100%;
  box-sizing: border-box;
  position: relative;
  cursor: pointer;
  user-select: none;
}

.account-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  border-radius: 10px;
  box-shadow: inset 0 0 0 2px rgba(0, 0, 0, 0);
  opacity: 0;
  transition: opacity 0.3s ease, box-shadow 0.3s ease;
  pointer-events: none;
}

/* 悬浮状态 - 优雅上浮 */
.account-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(0, 0, 0, 0.1);
  z-index: 1;
}

.account-card:hover::before {
  opacity: 1;
  box-shadow: inset 0 0 0 1px rgba(100, 116, 139, 0.15);
}

/* 选中状态 - 科技蓝 (Selected) */
.account-card.selected {
  border-color: transparent;
  background: linear-gradient(135deg, #f0f7ff 0%, #e0effe 100%);
  box-shadow: 
    0 4px 12px rgba(59, 130, 246, 0.15),
    0 0 0 1px rgba(59, 130, 246, 0.1);
}

.account-card.selected::before {
  box-shadow: 
    inset 0 0 0 2px rgba(59, 130, 246, 0.6),
    inset 0 0 12px rgba(59, 130, 246, 0.1);
  opacity: 1;
}

/* 当前激活账号 - 琥珀金 (Using) */
.account-card.current {
  background: linear-gradient(135deg, #fffbeb 0%, #fef3c7 100%);
  box-shadow: 
    0 4px 12px rgba(245, 158, 11, 0.15),
    0 0 0 1px rgba(245, 158, 11, 0.1);
}

.account-card.current::before {
  box-shadow: 
    inset 0 0 0 2px rgba(245, 158, 11, 0.6),
    inset 0 0 12px rgba(245, 158, 11, 0.15);
  opacity: 1;
}

.account-card.current:hover,
.account-card.selected:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
}

/* 已禁用账户 - 红色警告 */
.account-card.is-disabled {
  background: linear-gradient(135deg, #fef2f2 0%, #fee2e2 100%);
  box-shadow: 
    0 4px 12px rgba(239, 68, 68, 0.15),
    0 0 0 1px rgba(239, 68, 68, 0.1);
}

.account-card.is-disabled::before {
  box-shadow: 
    inset 0 0 0 2px rgba(239, 68, 68, 0.5),
    inset 0 0 12px rgba(239, 68, 68, 0.1);
  opacity: 1;
}

.account-card.is-disabled:hover {
  box-shadow: 0 8px 24px rgba(239, 68, 68, 0.2);
}

/* 订阅未激活 - 灰色样式 */
.account-card.subscription-inactive {
  background: linear-gradient(135deg, #f8fafc 0%, #e2e8f0 100%);
  box-shadow: 
    0 4px 12px rgba(100, 116, 139, 0.15),
    0 0 0 1px rgba(100, 116, 139, 0.1);
  filter: grayscale(30%);
}

.account-card.subscription-inactive::before {
  box-shadow: 
    inset 0 0 0 2px rgba(100, 116, 139, 0.3),
    inset 0 0 12px rgba(100, 116, 139, 0.1);
  opacity: 1;
}

.account-card.subscription-inactive:hover {
  box-shadow: 0 8px 24px rgba(100, 116, 139, 0.2);
  filter: grayscale(20%);
}

/* 拖拽手柄样式 */
.drag-handle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  margin-right: 6px;
  cursor: grab;
  color: #94a3b8;
  border-radius: 4px;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.drag-handle:hover {
  color: #3b82f6;
  background: rgba(59, 130, 246, 0.1);
}

.drag-handle:active {
  cursor: grabbing;
  color: #2563eb;
  background: rgba(59, 130, 246, 0.2);
}

:root.dark .drag-handle {
  color: #64748b;
}

:root.dark .drag-handle:hover {
  color: #60a5fa;
  background: rgba(96, 165, 250, 0.15);
}

.card-header {
  display: flex;
  align-items: center;
  margin-bottom: 4px;
  gap: 6px;
  padding-bottom: 4px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.05);
}

.account-info {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  display: flex;
  align-items: center;
  gap: 6px;
}

.email {
  font-size: 14px;
  font-weight: 600;
  color: #1e293b;
  cursor: pointer;
  transition: color 0.2s;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.email:hover {
  color: #3b82f6;
}

.nickname-tag {
  flex-shrink: 0;
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
  background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%) !important;
  color: #92400e !important;
  border: none !important;
  font-weight: 600;
  font-size: 10px;
  padding: 1px 6px !important;
  border-radius: 10px;
  box-shadow: 0 1px 3px rgba(146, 64, 14, 0.15);
  letter-spacing: 0.3px;
  margin-left: auto;
  margin-right: 4px;
  opacity: 0.9;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 3px;
  font-size: 10px;
  padding: 2px 6px;
  border-radius: 10px;
  background: rgba(0, 0, 0, 0.02);
  transition: all 0.2s ease;
}

.status-indicator:hover {
  background: rgba(0, 0, 0, 0.04);
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  position: relative;
  transition: all 0.3s ease;
}

.status-dot::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 100%;
  height: 100%;
  border-radius: 50%;
  opacity: 0.3;
}

.status-active .status-dot {
  background: #10b981;
  box-shadow: 0 0 0 2px rgba(16, 185, 129, 0.2);
}

.status-active .status-dot::after {
  animation: status-pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  background: #10b981;
}

.status-inactive .status-dot {
  background: #94a3b8;
}

.status-error .status-dot {
  background: #ef4444;
  box-shadow: 0 0 0 2px rgba(239, 68, 68, 0.2);
}

.status-disabled .status-dot {
  background: #f59e0b;
  box-shadow: 0 0 0 2px rgba(245, 158, 11, 0.2);
}

.status-text {
  font-weight: 500;
  color: #64748b;
}

.status-active .status-text {
  color: #10b981;
  font-weight: 600;
}

.status-error .status-text {
  color: #ef4444;
  font-weight: 600;
}

.status-disabled .status-text {
  color: #f59e0b;
  font-weight: 600;
}

.status-subscription-inactive .status-text {
  color: #64748b;
  font-weight: 600;
}

.status-subscription-inactive .status-dot {
  background: #64748b;
  box-shadow: 0 0 0 2px rgba(100, 116, 139, 0.2);
}

@keyframes status-pulse {
  0%, 100% {
    transform: translate(-50%, -50%) scale(1);
    opacity: 0.3;
  }
  50% {
    transform: translate(-50%, -50%) scale(2);
    opacity: 0;
  }
}

.card-body {
  margin-top: 4px;
}

/* 配额部分样式 */
.quota-section {
  background: linear-gradient(135deg, #f6f8fb 0%, #f0f3f8 100%);
  border-radius: 8px;
  padding: 6px;
  margin-bottom: 4px;
  border: 1px solid rgba(0, 0, 0, 0.04);
  transition: all 0.3s ease;
}

.quota-section:hover {
  background: linear-gradient(135deg, #f0f3f8 0%, #e8ecf3 100%);
}

.quota-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
  gap: 6px;
}

.quota-header-left {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.quota-header-right {
  display: flex;
  align-items: baseline;
  gap: 2px;
  margin-left: auto;
  font-family: 'Segoe UI', system-ui, sans-serif;
}

.plan-tag {
  color: white !important;
  border: none !important;
  font-weight: 700;
  font-size: 10.5px;
  padding: 2px 8px;
  height: auto;
  min-height: 20px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 3px;
  white-space: nowrap;
  border-radius: 6px;
  box-shadow: 
    0 2px 4px rgba(0, 0, 0, 0.1),
    0 1px 2px rgba(0, 0, 0, 0.06),
    inset 0 1px 0 rgba(255, 255, 255, 0.2),
    inset 0 -1px 0 rgba(0, 0, 0, 0.1);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  letter-spacing: 0.3px;
  position: relative;
  overflow: hidden;
  text-shadow: 0 1px 1px rgba(0, 0, 0, 0.1);
}

.plan-tag::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.15) 0%, rgba(255, 255, 255, 0) 60%, rgba(0, 0, 0, 0.05) 100%);
  pointer-events: none;
}

.plan-tag:hover {
  transform: translateY(-1px) scale(1.02);
  box-shadow: 
    0 4px 8px rgba(0, 0, 0, 0.15),
    0 2px 4px rgba(0, 0, 0, 0.1),
    inset 0 1px 0 rgba(255, 255, 255, 0.3);
  filter: brightness(1.05);
}

.plan-tag :deep(.el-tag__content) {
  display: inline-flex !important;
  flex-direction: row !important;
  align-items: center !important;
  gap: 3px !important;
}

.plan-tag .el-icon {
  font-size: 11px;
  margin-right: 1px;
  filter: drop-shadow(0 1px 1px rgba(0, 0, 0, 0.1));
  display: inline-flex !important;
}

/* 套餐颜色层级：Free < Trial < Pro < Teams < Enterprise */

/* Free套餐 - 银灰色，简约 */
.plan-tag.plan-free {
  background: linear-gradient(135deg, #9ca3af 0%, #6b7280 100%) !important;
  border: 1px solid rgba(255, 255, 255, 0.1) !important;
}

/* Trial套餐 - 琥珀金，醒目 */
.plan-tag.plan-trial {
  background: linear-gradient(135deg, #f59e0b 0%, #d97706 100%) !important;
  box-shadow: 0 2px 6px rgba(245, 158, 11, 0.25), inset 0 1px 0 rgba(255, 255, 255, 0.3) !important;
}

/* Pro套餐 - 科技蓝，专业 */
.plan-tag.plan-pro {
  background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%) !important;
  box-shadow: 0 2px 6px rgba(59, 130, 246, 0.25), inset 0 1px 0 rgba(255, 255, 255, 0.3) !important;
}

/* Teams套餐 - 翡翠绿，活力 */
.plan-tag.plan-teams {
  background: linear-gradient(135deg, #10b981 0%, #059669 100%) !important;
  box-shadow: 0 2px 6px rgba(16, 185, 129, 0.25), inset 0 1px 0 rgba(255, 255, 255, 0.3) !important;
}

/* Enterprise套餐 - 幻彩紫，尊贵 */
.plan-tag.plan-enterprise {
  background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 50%, #6d28d9 100%) !important;
  box-shadow: 0 2px 8px rgba(124, 58, 237, 0.35), inset 0 1px 0 rgba(255, 255, 255, 0.3) !important;
}

/* Enterprise流光动画 */
.plan-tag.plan-enterprise::after {
  content: '';
  position: absolute;
  top: -50%;
  left: -50%;
  width: 200%;
  height: 200%;
  background: linear-gradient(
    to right,
    rgba(255, 255, 255, 0) 0%,
    rgba(255, 255, 255, 0.15) 50%,
    rgba(255, 255, 255, 0) 100%
  );
  transform: rotate(30deg);
  animation: shimmer 3s infinite;
  pointer-events: none;
}

@keyframes shimmer {
  0% { transform: translateX(-100%) rotate(30deg); }
  100% { transform: translateX(100%) rotate(30deg); }
}

/* 默认颜色（其他套餐） */
.plan-tag:not(.plan-free):not(.plan-trial):not(.plan-pro):not(.plan-teams):not(.plan-enterprise) {
  background: linear-gradient(135deg, #6366f1 0%, #4f46e5 100%) !important;
}

.quota-used {
  font-size: 14px;
  font-weight: 700;
  color: #475569;
  letter-spacing: 0.3px;
}

.quota-separator {
  font-size: 12px;
  font-weight: 400;
  color: #94a3b8;
  margin: 0 1px;
}

.quota-total {
  font-size: 12px;
  font-weight: 600;
  color: #64748b;
  letter-spacing: 0.3px;
}

.quota-progress {
  position: relative;
  display: flex;
  align-items: center;
  gap: 8px;
}

.quota-progress .el-progress {
  flex: 1;
}

.quota-progress :deep(.el-progress-bar__outer) {
  background-color: #e2e8f0;
  height: 6px !important;
}

.quota-progress :deep(.el-progress-bar__inner) {
  border-radius: 10px;
  transition: all 0.3s ease;
  background-image: linear-gradient(90deg, var(--color-start) 0%, var(--color-end) 100%);
}

/* 为不同百分比设置不同的渐变色 */
.quota-progress :deep(.el-progress-bar__inner[style*="#10b981"]) {
  --color-start: #10b981;
  --color-end: #34d399;
}

.quota-progress :deep(.el-progress-bar__inner[style*="#f59e0b"]) {
  --color-start: #f59e0b;
  --color-end: #fbbf24;
}

.quota-progress :deep(.el-progress-bar__inner[style*="#ef4444"]) {
  --color-start: #ef4444;
  --color-end: #f87171;
}

.quota-percentage {
  min-width: 36px;
  text-align: right;
  font-size: 11px;
  font-weight: 600;
  color: #475569;
}

/* QUOTA 模式：日/周配额百分比行样式 */
.quota-percent-row {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-bottom: 2px;
}

.quota-percent-label {
  font-size: 11px;
  font-weight: 600;
  color: #64748b;
  min-width: 36px;
  flex-shrink: 0;
}

.quota-percent-bar {
  flex: 1;
}

.quota-percent-bar :deep(.el-progress-bar__outer) {
  background-color: #e2e8f0;
  height: 6px !important;
}

.quota-percent-bar :deep(.el-progress-bar__inner) {
  border-radius: 10px;
  transition: all 0.3s ease;
}

.quota-percent-value {
  min-width: 36px;
  text-align: right;
  font-size: 11px;
  font-weight: 600;
  color: #475569;
  flex-shrink: 0;
}

.quota-reset-time {
  font-size: 10px;
  color: #94a3b8;
  flex-shrink: 0;
  font-family: 'Segoe UI', system-ui, sans-serif;
}

/* 配额区块内的订阅到期时间样式 */
.quota-expiry {
  display: flex;
  align-items: center;
  gap: 3px;
  margin-top: 4px;
  padding-top: 4px;
  border-top: 1px solid rgba(0, 0, 0, 0.06);
  font-size: 10px;
}

.quota-expiry .expiry-icon {
  color: #64748b;
  font-size: 12px;
}

.quota-expiry .expiry-label {
  color: #64748b;
  font-weight: 500;
}

.quota-expiry .expiry-date {
  color: #475569;
  font-weight: 500;
  font-family: 'Segoe UI', system-ui, sans-serif;
}

.quota-expiry .expiry-badge {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 10px;
  margin-left: auto;
  white-space: nowrap;
}

.quota-expiry .expiry-badge.normal {
  color: #10b981;
  background-color: rgba(16, 185, 129, 0.1);
}

.quota-expiry .expiry-badge.expiring-soon {
  color: #f59e0b;
  background-color: rgba(245, 158, 11, 0.15);
  animation: gentle-pulse 2s ease-in-out infinite;
}

.quota-expiry .expiry-badge.expired {
  color: #ef4444;
  background-color: rgba(239, 68, 68, 0.15);
  font-weight: 700;
}

@keyframes gentle-pulse {
  0%, 100% {
    opacity: 1;
    transform: scale(1);
  }
  50% {
    opacity: 0.85;
    transform: scale(1.02);
  }
}

.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 2px;
  margin-bottom: 2px;
}

/* 自定义颜色标签 */
.custom-tag {
  border-radius: 4px !important;
  font-weight: 500 !important;
  font-size: 10px !important;
  padding: 0 6px !important;
  height: 18px !important;
  line-height: 18px !important;
  transition: all 0.2s ease !important;
}

.custom-tag:hover {
  transform: scale(1.05);
  filter: brightness(1.1);
}

/* 带颜色标签的卡片边框效果 */
.account-card.has-tag-color {
  border-width: 2px;
  border-style: solid;
}

.account-card.has-tag-color:hover {
  filter: brightness(1.02);
}

.account-card.has-tag-color::before {
  opacity: 0.5;
}

/* 信息标签组 */
.info-tags {
  display: flex;
  justify-content: space-evenly;
  align-items: center;
  gap: 6px;
  margin-top: 4px;
  margin-bottom: 2px;
}

.info-tag {
  font-size: 10px !important;
  padding: 0 10px !important;
  height: 20px !important;
  border-radius: 4px !important;
  display: inline-flex !important;
  flex-direction: row !important;
  align-items: center !important;
  gap: 4px !important;
  flex: 1;
  justify-content: center !important;
  min-width: 0;
  font-weight: 500;
}

.info-tag :deep(.el-tag__content) {
  display: inline-flex !important;
  flex-direction: row !important;
  align-items: center !important;
  gap: 4px !important;
}

.info-tag .el-icon {
  font-size: 10px !important;
  margin-right: 1px;
  display: inline-flex !important;
}

.info-tag span {
  font-size: 10px;
  white-space: nowrap;
  text-overflow: ellipsis;
  overflow: hidden;
}

/* 分组标签 - 浅蓝色 */
.group-tag {
  background: rgba(59, 130, 246, 0.1) !important;
  color: #64748b !important;
  border: 1px solid rgba(59, 130, 246, 0.15) !important;
}

/* 创建时间标签 - 浅紫色 */
.create-tag {
  background: rgba(168, 85, 247, 0.1) !important;
  color: #64748b !important;
  border: 1px solid rgba(168, 85, 247, 0.15) !important;
}

/* Token标签根据过期时间变色 */
.token-tag {
  border: none !important;
}

.token-tag.el-tag--success {
  background: rgba(16, 185, 129, 0.1) !important;
  color: #10b981 !important;
  border: 1px solid rgba(16, 185, 129, 0.15) !important;
}

.token-tag.el-tag--warning {
  background: rgba(245, 158, 11, 0.1) !important;
  color: #f59e0b !important;
  border: 1px solid rgba(245, 158, 11, 0.15) !important;
}

.token-tag.el-tag--danger {
  background: rgba(239, 68, 68, 0.1) !important;
  color: #ef4444 !important;
  border: 1px solid rgba(239, 68, 68, 0.15) !important;
}

.token-tag.el-tag--info {
  background: rgba(107, 114, 128, 0.1) !important;
  color: #6b7280 !important;
  border: 1px solid rgba(107, 114, 128, 0.15) !important;
}

.card-actions {
  padding: 5px 8px;
  border-top: 1px solid #ebeef5;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.action-buttons {
  display: flex;
  gap: 4px;
  justify-content: space-evenly;
  flex-wrap: nowrap;
  align-items: center;
  padding: 2px;
  height: 28px;
}

.action-buttons .el-button {
  flex: 0 0 auto;
  margin: 0;
}

.action-buttons .el-button.el-button--small.is-circle {
  width: 26px;
  height: 26px;
  padding: 0;
  min-width: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 调整图标大小 */
.action-buttons .el-button .el-icon {
  font-size: 13px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

/* 暗色主题支持 */
:root.dark .account-card {
  background: #1e1e1e;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
}

:root.dark .account-card::before {
  box-shadow: inset 0 0 0 2px rgba(255, 255, 255, 0.05);
}

:root.dark .account-card:hover {
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.6);
  background: #252525;
}

:root.dark .account-card:hover::before {
  opacity: 1;
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.1);
}

/* 暗色主题 - 选中状态 (Selected - Tech Blue) */
:root.dark .account-card.selected {
  border-color: transparent;
  background: linear-gradient(135deg, #1e293b 0%, #172554 100%);
  box-shadow: 
    0 4px 12px rgba(30, 64, 175, 0.3),
    0 0 0 1px rgba(59, 130, 246, 0.3);
}

:root.dark .account-card.selected::before {
  box-shadow: 
    inset 0 0 0 1px rgba(96, 165, 250, 0.5),
    inset 0 0 20px rgba(37, 99, 235, 0.2);
  opacity: 1;
}

/* 暗色主题 - 当前账号 (Current - Amber Gold) */
:root.dark .account-card.current {
  background: linear-gradient(135deg, #2a2515 0%, #453a10 100%);
  box-shadow: 
    0 4px 12px rgba(180, 83, 9, 0.25),
    0 0 0 1px rgba(245, 158, 11, 0.3);
}

:root.dark .account-card.current::before {
  box-shadow: 
    inset 0 0 0 1px rgba(251, 191, 36, 0.5),
    inset 0 0 20px rgba(217, 119, 6, 0.2);
  opacity: 1;
}

:root.dark .account-card.current:hover,
:root.dark .account-card.selected:hover {
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
  filter: brightness(1.1);
}

/* 暗色主题 - 已禁用账户 (Disabled - Warning Red) */
:root.dark .account-card.is-disabled {
  background: linear-gradient(135deg, #2a1515 0%, #451a1a 100%);
  box-shadow: 
    0 4px 12px rgba(185, 28, 28, 0.25),
    0 0 0 1px rgba(239, 68, 68, 0.3);
}

:root.dark .account-card.is-disabled::before {
  box-shadow: 
    inset 0 0 0 1px rgba(248, 113, 113, 0.5),
    inset 0 0 20px rgba(220, 38, 38, 0.2);
  opacity: 1;
}

:root.dark .account-card.is-disabled:hover {
  box-shadow: 0 8px 24px rgba(185, 28, 28, 0.4);
  filter: brightness(1.1);
}

/* 暗色主题 - 订阅未激活 (灰色) */
:root.dark .account-card.subscription-inactive {
  background: linear-gradient(135deg, #1e293b 0%, #334155 100%);
  box-shadow: 
    0 4px 12px rgba(71, 85, 105, 0.25),
    0 0 0 1px rgba(100, 116, 139, 0.3);
  filter: grayscale(30%);
}

:root.dark .account-card.subscription-inactive::before {
  box-shadow: 
    inset 0 0 0 1px rgba(148, 163, 184, 0.3),
    inset 0 0 20px rgba(100, 116, 139, 0.2);
  opacity: 1;
}

:root.dark .account-card.subscription-inactive:hover {
  box-shadow: 0 8px 24px rgba(71, 85, 105, 0.4);
  filter: grayscale(20%) brightness(1.1);
}

:root.dark .card-header {
  border-bottom-color: rgba(255, 255, 255, 0.08);
}

:root.dark .email {
  color: #f1f5f9;
}

:root.dark .nickname-tag {
  background: linear-gradient(135deg, rgba(251, 191, 36, 0.2) 0%, rgba(245, 158, 11, 0.25) 100%) !important;
  color: #fbbf24 !important;
  box-shadow: 0 1px 3px rgba(251, 191, 36, 0.2);
  opacity: 0.9;
}

:root.dark .status-indicator {
  background: rgba(255, 255, 255, 0.03);
}

:root.dark .status-indicator:hover {
  background: rgba(255, 255, 255, 0.05);
}

:root.dark .status-text {
  color: #94a3b8;
}

:root.dark .status-active .status-text {
  color: #34d399;
}

:root.dark .status-error .status-text {
  color: #f87171;
}

:root.dark .status-active .status-dot {
  background: #34d399;
  box-shadow: 0 0 0 2px rgba(52, 211, 153, 0.25);
}

:root.dark .status-active .status-dot::after {
  background: #34d399;
}

:root.dark .status-error .status-dot {
  background: #f87171;
  box-shadow: 0 0 0 2px rgba(248, 113, 113, 0.25);
}

:root.dark .status-disabled .status-text {
  color: #fbbf24;
}

:root.dark .status-disabled .status-dot {
  background: #fbbf24;
  box-shadow: 0 0 0 2px rgba(251, 191, 36, 0.25);
}

:root.dark .status-subscription-inactive .status-text {
  color: #94a3b8;
}

:root.dark .status-subscription-inactive .status-dot {
  background: #94a3b8;
  box-shadow: 0 0 0 2px rgba(148, 163, 184, 0.25);
}

/* 暗色主题下的配额样式 */
:root.dark .quota-section {
  background: linear-gradient(135deg, #2a2a2a 0%, #252525 100%);
  border: 1px solid rgba(255, 255, 255, 0.08);
}

:root.dark .quota-section:hover {
  background: linear-gradient(135deg, #2f2f2f 0%, #2a2a2a 100%);
}

/* 暗色主题下的套餐标签 */
:root.dark .plan-tag {
  box-shadow: 
    0 2px 6px rgba(0, 0, 0, 0.3),
    0 1px 3px rgba(0, 0, 0, 0.2),
    inset 0 1px 0 rgba(255, 255, 255, 0.15),
    inset 0 -1px 0 rgba(0, 0, 0, 0.2);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

:root.dark .plan-tag::before {
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.1) 0%, rgba(255, 255, 255, 0) 60%, rgba(0, 0, 0, 0.1) 100%);
}

:root.dark .plan-tag:hover {
  box-shadow: 
    0 4px 10px rgba(0, 0, 0, 0.4),
    0 2px 5px rgba(0, 0, 0, 0.25),
    inset 0 1px 0 rgba(255, 255, 255, 0.2);
  filter: brightness(1.1);
}

/* 暗色主题套餐颜色 */
:root.dark .plan-tag.plan-free {
  background: linear-gradient(135deg, #4b5563 0%, #6b7280 100%) !important;
  border: 1px solid rgba(255, 255, 255, 0.08) !important;
}

:root.dark .plan-tag.plan-trial {
  background: linear-gradient(135deg, #b45309 0%, #d97706 100%) !important;
  box-shadow: 0 2px 6px rgba(180, 83, 9, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.15) !important;
}

:root.dark .plan-tag.plan-pro {
  background: linear-gradient(135deg, #1e40af 0%, #2563eb 100%) !important;
  box-shadow: 0 2px 6px rgba(30, 64, 175, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.15) !important;
}

:root.dark .plan-tag.plan-teams {
  background: linear-gradient(135deg, #047857 0%, #059669 100%) !important;
  box-shadow: 0 2px 6px rgba(4, 120, 87, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.15) !important;
}

:root.dark .plan-tag.plan-enterprise {
  background: linear-gradient(135deg, #6d28d9 0%, #7c3aed 50%, #8b5cf6 100%) !important;
  box-shadow: 0 2px 10px rgba(109, 40, 217, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.15) !important;
}

:root.dark .plan-tag:not(.plan-free):not(.plan-trial):not(.plan-pro):not(.plan-teams):not(.plan-enterprise) {
  background: linear-gradient(135deg, #4338ca 0%, #4f46e5 100%) !important;
  box-shadow: 0 2px 6px rgba(67, 56, 202, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.15) !important;
}

:root.dark .quota-used {
  color: #cbd5e1;
}

:root.dark .quota-separator {
  color: #64748b;
}

:root.dark .quota-total {
  color: #94a3b8;
}

:root.dark .quota-percentage {
  color: #cbd5e1;
}

:root.dark .quota-progress :deep(.el-progress-bar__outer) {
  background-color: #374151;
}

:root.dark .quota-percent-label {
  color: #94a3b8;
}

:root.dark .quota-percent-value {
  color: #cbd5e1;
}

:root.dark .quota-percent-bar :deep(.el-progress-bar__outer) {
  background-color: #374151;
}

:root.dark .quota-reset-time {
  color: #64748b;
}

/* 暗色主题下的订阅到期时间样式 */
:root.dark .quota-expiry {
  border-top-color: rgba(255, 255, 255, 0.08);
}

:root.dark .quota-expiry .expiry-icon {
  color: #94a3b8;
}

:root.dark .quota-expiry .expiry-label {
  color: #94a3b8;
}

:root.dark .quota-expiry .expiry-date {
  color: #cbd5e1;
}

/* 深色模式下的功能按钮样式 */
:root.dark .card-actions {
  border-top-color: rgba(255, 255, 255, 0.08);
}

:root.dark .action-buttons .el-button.is-circle {
  background-color: #262729;
  border-color: #4c4d4f;
  color: #cfd3dc;
}

:root.dark .action-buttons .el-button.is-circle:hover {
  background-color: #303133;
  border-color: #5a5b5d;
  color: #409eff;
}

:root.dark .action-buttons .el-button.is-circle:active {
  background-color: #1a1a1c;
  border-color: #409eff;
}

:root.dark .action-buttons .el-button--danger {
  background-color: transparent;
  border-color: rgba(245, 108, 108, 0.3);
  color: #f56c6c;
}

:root.dark .action-buttons .el-button--danger:hover {
  background-color: rgba(245, 108, 108, 0.1);
  border-color: rgba(245, 108, 108, 0.5);
  color: #ff6b6b;
}

:root.dark .quota-expiry .expiry-badge.normal {
  color: #34d399;
  background-color: rgba(52, 211, 153, 0.15);
}

:root.dark .quota-expiry .expiry-badge.expiring-soon {
  color: #fbbf24;
  background-color: rgba(251, 191, 36, 0.2);
}

:root.dark .quota-expiry .expiry-badge.expired {
  color: #f87171;
  background-color: rgba(248, 113, 113, 0.2);
}

/* 暗色主题下的信息标签 */
:root.dark .info-tags {
  gap: 4px;
}

:root.dark .group-tag {
  background: rgba(59, 130, 246, 0.1) !important;
  color: #60a5fa !important;
  border: 1px solid rgba(59, 130, 246, 0.15) !important;
}

:root.dark .create-tag {
  background: rgba(168, 85, 247, 0.1) !important;
  color: #c084fc !important;
  border: 1px solid rgba(168, 85, 247, 0.15) !important;
}

:root.dark .token-tag.el-tag--success {
  background: rgba(16, 185, 129, 0.1) !important;
  color: #34d399 !important;
  border: 1px solid rgba(16, 185, 129, 0.15) !important;
}

:root.dark .token-tag.el-tag--warning {
  background: rgba(245, 158, 11, 0.1) !important;
  color: #fbbf24 !important;
  border: 1px solid rgba(245, 158, 11, 0.15) !important;
}

:root.dark .token-tag.el-tag--danger {
  background: rgba(239, 68, 68, 0.1) !important;
  color: #f87171 !important;
  border: 1px solid rgba(239, 68, 68, 0.15) !important;
}

:root.dark .token-tag.el-tag--info {
  background: rgba(107, 114, 128, 0.1) !important;
  color: #9ca3af !important;
  border: 1px solid rgba(107, 114, 128, 0.15) !important;
}

/* ==================== 切号进度弹窗 ==================== */
.switch-progress-body {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 4px 0 8px;
}

.switch-progress-label {
  font-size: 13px;
  color: #606266;
  min-height: 20px;
  line-height: 1.5;
}

.switch-progress-label.is-error {
  color: #F56C6C;
  font-weight: 500;
}

.switch-progress-steps {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 12px;
  background-color: rgba(0, 0, 0, 0.02);
  border: 1px solid rgba(0, 0, 0, 0.06);
  border-radius: 6px;
}

.switch-progress-step {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  line-height: 1.6;
  color: #C0C4CC;
  transition: color 0.2s ease;
}

.switch-progress-step .step-icon {
  font-size: 14px;
  flex-shrink: 0;
}

.switch-progress-step.status-done {
  color: #67C23A;
}
.switch-progress-step.status-done .step-icon {
  color: #67C23A;
}

.switch-progress-step.status-running {
  color: #303133;
  font-weight: 500;
}
.switch-progress-step.status-running .step-icon {
  color: #409EFF;
}

.switch-progress-step.status-error {
  color: #F56C6C;
  font-weight: 500;
}
.switch-progress-step.status-error .step-icon {
  color: #F56C6C;
}

.switch-progress-step.status-pending .step-icon {
  color: #DCDFE6;
}

/* running 步骤图标的 spinner 动画 */
.switch-progress-step .step-icon.is-spin {
  animation: switch-progress-spin 1s linear infinite;
}

@keyframes switch-progress-spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* 深色模式适配 */
:root.dark .switch-progress-label {
  color: #cfd3dc;
}

:root.dark .switch-progress-steps {
  background-color: rgba(255, 255, 255, 0.03);
  border-color: rgba(255, 255, 255, 0.08);
}

:root.dark .switch-progress-step {
  color: #7a8394;
}

:root.dark .switch-progress-step.status-running {
  color: #e2e8f0;
}

:root.dark .switch-progress-step.status-pending .step-icon {
  color: #4b                       