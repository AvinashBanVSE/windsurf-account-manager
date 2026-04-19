<template>
  <el-dialog
    v-model="uiStore.showAddAccountDialog"
    title="Add Account"
    width="500px"
    :close-on-click-modal="false"
  >
    <el-form
      ref="formRef"
      :model="formData"
      :rules="currentRules"
      label-width="100px"
      autocomplete="off"
    >
      <!-- Add method switch: compact card grid (2 columns, single row layout, auto single column on narrow screen)
           desc shown in native title attribute, displayed on hover. -->
      <el-form-item label="Add Method">
        <div class="mode-grid" role="radiogroup" aria-label="Add Method">
          <div
            v-for="opt in modeOptions"
            :key="opt.value"
            class="mode-card"
            :class="{ 'is-active': addMode === opt.value }"
            :title="opt.desc"
            role="radio"
            :aria-checked="addMode === opt.value"
            tabindex="0"
            @click="selectMode(opt.value)"
            @keydown.enter.prevent="selectMode(opt.value)"
            @keydown.space.prevent="selectMode(opt.value)"
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
            <el-icon v-if="addMode === opt.value" class="mode-card__check">
              <Check />
            </el-icon>
          </div>
        </div>
      </el-form-item>

      <!-- Smart mode: Enter email+password, auto identify Firebase / Devin -->
      <template v-if="addMode === 'smart'">
        <el-alert
          type="success"
          :closable="false"
          show-icon
          style="margin-bottom: 18px;"
        >
          <template #title>
            <span style="font-size: 12px;">
              Enter email and password, system will automatically identify <strong>Firebase</strong> / <strong>Devin Auth1</strong> account
            </span>
          </template>
        </el-alert>
        <el-form-item label="Email" prop="email">
          <el-input
            v-model="formData.email"
            placeholder="Enter email"
            :prefix-icon="Message"
            autocomplete="off"
          />
        </el-form-item>
        <el-form-item label="Password" prop="password">
          <el-input
            v-model="formData.password"
            type="password"
            placeholder="Enter password"
            :prefix-icon="Lock"
            show-password
            autocomplete="new-password"
          />
        </el-form-item>
      </template>

      <!-- Email Password mode (Legacy Firebase) -->
      <template v-else-if="addMode === 'password'">
        <el-form-item label="Email" prop="email">
          <el-input
            v-model="formData.email"
            placeholder="Enter email"
            :prefix-icon="Message"
            autocomplete="off"
          />
        </el-form-item>
        
        <el-form-item label="Password" prop="password">
          <el-input
            v-model="formData.password"
            type="password"
            placeholder="Enter password"
            :prefix-icon="Lock"
            show-password
            autocomplete="new-password"
          />
        </el-form-item>
      </template>

      <!-- Refresh Token mode -->
      <template v-else-if="addMode === 'refresh_token'">
        <el-form-item label="Refresh Token" prop="refreshToken">
          <el-input
            v-model="formData.refreshToken"
            type="textarea"
            :rows="3"
            placeholder="Enter Refresh Token"
          />
        </el-form-item>
      </template>

      <!-- Devin Session Token mode: Paste devin-session-token$... to import -->
      <template v-else-if="addMode === 'devin_session'">
        <el-alert
          type="warning"
          :closable="false"
          show-icon
          style="margin-bottom: 18px;"
        >
          <template #title>
            <span style="font-size: 12px;">
              Paste the complete <code>devin-session-token$...</code> session_token,
              system will automatically query email / quota / api_key via GetCurrentUser.
              Applicable when migrating tokens from browser localStorage/cookie after login.
            </span>
          </template>
        </el-alert>
        <el-form-item label="Session Token" prop="sessionToken">
          <el-input
            v-model="formData.sessionToken"
            type="textarea"
            :rows="3"
            placeholder="Paste complete devin-session-token$... token"
          />
        </el-form-item>
      </template>

      <!-- Devin 邮箱验证码模式：两步流程，按 flow 区分 login / signup -->
      <template v-else-if="addMode === 'devin_email_code'">
        <!-- 顶部说明：按 flow 动态文案 -->
        <el-alert
          v-if="devinEmailCodeFlow === 'signup'"
          type="warning"
          :closable="false"
          show-icon
          style="margin-bottom: 16px;"
        >
          <template #title>
            <span style="font-size: 12px;">
              <strong>Register new Devin account</strong>: Create new account via email code, requires setting password and name.
              <strong>This flow will create a new account</strong>.
            </span>
          </template>
        </el-alert>
        <el-alert
          v-else
          type="info"
          :closable="false"
          show-icon
          style="margin-bottom: 16px;"
        >
          <template #title>
            <span style="font-size: 12px;">
              For existing Devin accounts without password (SSO migration / forgot password / Google・GitHub login):
              Login and add via email code. <strong>This flow will NOT create a new account</strong>.
            </span>
          </template>
        </el-alert>

        <el-steps :active="devinEmailCodeStep" finish-status="success" simple style="margin-bottom: 20px;">
          <el-step title="Send Code" />
          <el-step :title="devinEmailCodeFlow === 'signup' ? 'Complete Registration' : 'Enter Code'" />
        </el-steps>

        <!-- Step 0：输入邮箱 -->
        <template v-if="devinEmailCodeStep === 0">
          <el-form-item label="Email" prop="email">
            <el-input
              v-model="formData.email"
              placeholder="Enter Devin account email"
              :prefix-icon="Message"
              autocomplete="off"
            />
          </el-form-item>
        </template>

        <!-- Step 1：输入验证码（signup flow 额外要求新密码 + 姓名） -->
        <template v-else>
          <el-alert
            v-if="devinEmailCodeFlow === 'signup'"
            type="warning"
            :closable="false"
            show-icon
            style="margin-bottom: 16px;"
          >
            <template #title>
              <span style="font-size: 12px;">
                <strong>Register new account</strong>: Verification code sent to {{ formData.email }},
                enter code and set password/name to complete registration
              </span>
            </template>
          </el-alert>
          <el-alert v-else type="success" :closable="false" show-icon style="margin-bottom: 16px;">
            Verification code sent to: {{ formData.email }}
          </el-alert>

          <el-form-item label="Email">
            <el-input :model-value="formData.email" disabled />
          </el-form-item>
          <el-form-item label="Code" prop="devinEmailCodeOtp">
            <el-input
              v-model="formData.devinEmailCodeOtp"
              placeholder="Enter 6-digit code from email"
              maxlength="10"
            />
          </el-form-item>

          <!-- signup flow 专属字段 -->
          <template v-if="devinEmailCodeFlow === 'signup'">
            <el-form-item label="New Password" prop="devinEmailCodePassword">
              <el-input
                v-model="formData.devinEmailCodePassword"
                type="password"
                placeholder="Set new password, at least 6 characters"
                :prefix-icon="Lock"
                show-password
                autocomplete="new-password"
              />
            </el-form-item>
            <el-form-item label="Name" prop="devinEmailCodeName">
              <el-input
                v-model="formData.devinEmailCodeName"
                placeholder="Enter display name (empty uses email prefix)"
                :prefix-icon="User"
              />
            </el-form-item>
          </template>
        </template>
      </template>

      <!-- Devin 账密模式（新 Devin Session 体系） -->
      <template v-else>
        <el-alert
          type="info"
          :closable="false"
          show-icon
          style="margin-bottom: 18px;"
        >
          <template #title>
            <span style="font-size: 12px;">
              通过 Devin Session 新体系登录（<code>/_devin-auth/password/login</code> +
              <code>WindsurfPostAuth</code>），无 Google API Key 限制、无需 Token 刷新
            </span>
          </template>
        </el-alert>

        <el-form-item label="Email" prop="email">
          <el-input
            v-model="formData.email"
            placeholder="Enter Devin account email"
            :prefix-icon="Message"
            autocomplete="off"
          />
        </el-form-item>

        <el-form-item label="Password" prop="password">
          <el-input
            v-model="formData.password"
            type="password"
            placeholder="Enter Devin password"
            :prefix-icon="Lock"
            show-password
            autocomplete="new-password"
          />
        </el-form-item>
      </template>
      
      <el-form-item label="Nickname" prop="nickname">
        <el-input
          v-model="formData.nickname"
          placeholder="Leave empty to use email username"
          :prefix-icon="User"
        />
      </el-form-item>
      
      <el-form-item label="Group">
        <el-select
          v-model="formData.group"
          placeholder="Select group"
          clearable
        >
          <el-option
            v-for="group in settingsStore.groups"
            :key="group"
            :label="group"
            :value="group"
          />
        </el-select>
      </el-form-item>
      
      <el-form-item label="Tags">
        <el-select
          v-model="formData.tags"
          multiple
          filterable
          allow-create
          placeholder="Enter or select tags"
          style="width: 100%"
        >
          <el-option
            v-for="tag in settingsStore.tags"
            :key="tag.name"
            :label="tag.name"
            :value="tag.name"
          >
            <span :style="getTagOptionStyle(tag.color)">{{ tag.name }}</span>
          </el-option>
        </el-select>
      </el-form-item>
    </el-form>
    
<template #footer>
      <el-button @click="handleClose">Cancel</el-button>

      <!-- Devin 邮箱验证码模式：按 step 动态按钮 -->
      <template v-if="addMode === 'devin_email_code'">
        <el-button v-if="devinEmailCodeStep === 1" @click="devinEmailCodeStep = 0" :disabled="loading">
          Back
        </el-button>
        <el-button type="primary" @click="handleSubmit" :loading="loading">
          {{ devinEmailCodeStep === 0
              ? 'Send Code'
              : (devinEmailCodeFlow === 'signup' ? 'Complete Registration' : 'Complete Add') }}
        </el-button>
      </template>

      <!-- 其他模式：统一"确定"按钮 -->
      <el-button v-else type="primary" @click="handleSubmit" :loading="loading">
        Confirm
      </el-button>
    </template>

      <!-- 其他模式：统一“确定”按钮 -->
      <el-button v-else type="primary" @click="handleSubmit" :loading="loading">
        确定
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, nextTick } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import type { FormInstance, FormRules } from 'element-plus';
import { Message, Lock, User, MagicStick, Refresh, Connection, Check } from '@element-plus/icons-vue';
import { useAccountsStore, useSettingsStore, useUIStore } from '@/store';
import { apiService, accountApi, devinApi } from '@/api';
import type { WindsurfOrg, LoginMethodSniffResult } from '@/types';
import { invoke } from '@tauri-apps/api/core';

const accountsStore = useAccountsStore();
const settingsStore = useSettingsStore();
const uiStore = useUIStore();

const formRef = ref<FormInstance>();
const loading = ref(false);
const addMode = ref<'smart' | 'password' | 'refresh_token' | 'devin' | 'devin_session' | 'devin_email_code'>('smart');

// Devin 邮箱验证码登录的两步状态（mode === 'devin_email_code' 专属）
// step 0：输入邮箱 + 发送验证码；step 1：输入验证码 + 完成登录/注册
const devinEmailCodeStep = ref<0 | 1>(0);
// /email/start 返回的 email_verification_token，用于后续 /email/complete
const devinEmailCodeEmailToken = ref('');
// 验证码子流程：login=登录已有无密码账号；signup=注册新账号
// - 从 radio 主入口直接选 devin_email_code 时默认 'login'
// - 从智能识别 not_found 分派快捷按钮进入时自动设为 'signup'
const devinEmailCodeFlow = ref<'login' | 'signup'>('login');

const formData = reactive({
  email: '',
  password: '',
  refreshToken: '',
  sessionToken: '',
  devinEmailCodeOtp: '',
  devinEmailCodePassword: '',
  devinEmailCodeName: '',
  nickname: '',
  group: '默认分组',
  tags: [] as string[]
});

/**
 * 添加方式选项的元数据
 *
 * 顺序按「推荐度 + 流派聚合」排列：
 * 1) smart 智能识别（推荐，置顶）
 * 2) Devin 系：账密 / 邮箱验证码 / session_token（新体系，日常主力）
 * 3) Firebase 系：邮箱密码 / Refresh Token（传统体系，兼容老账号）
 *
 * 每项承载卡片渲染所需的全部视觉数据（图标、标题、标签、一句话说明）。
 * 新增模式时只需在此数组里追加一条，模板网格自动同步渲染。
 */
const modeOptions = [
  {
    value: 'smart',
    title: 'Smart Detect',
    desc: 'Enter email and password, system will automatically select best login method',
    icon: MagicStick,
    tag: 'Recommended',
    tagType: 'primary' as const,
  },
  {
    value: 'devin',
    title: 'Devin Credentials',
    desc: 'Login with Devin account credentials',
    icon: User,
    tag: 'New',
    tagType: 'success' as const,
  },
  {
    value: 'devin_email_code',
    title: 'Devin Email Code',
    desc: 'Login or register with code for SSO/passwordless account',
    icon: Message,
    tag: 'Passwordless',
    tagType: 'info' as const,
  },
  {
    value: 'devin_session',
    title: 'Devin Session Token',
    desc: 'Paste devin-session-token$... to migrate directly',
    icon: Connection,
    tag: 'Migrate',
    tagType: 'warning' as const,
  },
  {
    value: 'password',
    title: 'Email Password',
    desc: 'Traditional Firebase account login',
    icon: Lock,
    tag: '',
    tagType: 'info' as const,
  },
  {
    value: 'refresh_token',
    title: 'Refresh Token',
    desc: 'Paste Firebase refresh_token to import',
    icon: Refresh,
    tag: '',
    tagType: 'info' as const,
  },
] as const;

/**
 * 切换添加方式
 *
 * 卡片点击时由模板调用；内部直接写入 `addMode` 并复用原有的 `handleModeChange`
 * 清理逻辑（重置验证码 step / email_token / flow 等），保证与 el-radio 版本行为完全一致。
 */
function selectMode(value: string) {
  if (addMode.value === value) return;
  addMode.value = value as typeof addMode.value;
  handleModeChange();
}

// 邮箱密码模式的验证规则
const passwordRules: FormRules = {
  email: [
    { required: true, message: 'Please enter email', trigger: 'blur' },
    { type: 'email', message: 'Please enter a valid email address', trigger: 'blur' }
  ],
  password: [
    { required: true, message: 'Please enter password', trigger: 'blur' },
    { min: 6, message: 'Password must be at least 6 characters', trigger: 'blur' }
  ],
  nickname: [
    { max: 20, message: 'Nickname can be up to 20 characters', trigger: 'blur' }
  ]
};

// Refresh Token 模式的验证规则
const refreshTokenRules: FormRules = {
  refreshToken: [
    { required: true, message: 'Please enter Refresh Token', trigger: 'blur' },
    { min: 10, message: 'Invalid Refresh Token format', trigger: 'blur' }
  ],
  nickname: [
    { max: 20, message: 'Nickname can be up to 20 characters', trigger: 'blur' }
  ]
};

// Devin 账密模式的验证规则（与 passwordRules 一致）
const devinRules: FormRules = {
  email: [
    { required: true, message: 'Please enter Devin account email', trigger: 'blur' },
    { type: 'email', message: 'Please enter a valid email address', trigger: 'blur' }
  ],
  password: [
    { required: true, message: 'Please enter Devin password', trigger: 'blur' },
    { min: 6, message: 'Password must be at least 6 characters', trigger: 'blur' }
  ],
  nickname: [
    { max: 20, message: 'Nickname can be up to 20 characters', trigger: 'blur' }
  ]
};

// Devin 邮箱验证码模式的验证规则：按 step 分组
// step 0 只校验 email，step 1 只校验 验证码
// （避免在需要发验证码的阶段反骨用户填验证码）
const devinEmailCodeStep0Rules: FormRules = {
  email: [
    { required: true, message: 'Please enter email', trigger: 'blur' },
    { type: 'email', message: 'Please enter a valid email address', trigger: 'blur' }
  ],
  nickname: [
    { max: 20, message: 'Nickname can be up to 20 characters', trigger: 'blur' }
  ]
};
const devinEmailCodeStep1Rules: FormRules = {
  devinEmailCodeOtp: [
    { required: true, message: 'Please enter verification code', trigger: 'blur' },
    { min: 4, message: 'Invalid verification code length', trigger: 'blur' }
  ],
  nickname: [
    { max: 20, message: 'Nickname can be up to 20 characters', trigger: 'blur' }
  ]
};
// Step 1 注册子流程：验证码 + 新密码 (至少 6 位) + 姓名 (可选)
const devinEmailCodeStep1SignupRules: FormRules = {
  devinEmailCodeOtp: [
    { required: true, message: 'Please enter verification code', trigger: 'blur' },
    { min: 4, message: 'Invalid verification code length', trigger: 'blur' }
  ],
  devinEmailCodePassword: [
    { required: true, message: 'Please set new password', trigger: 'blur' },
    { min: 6, message: 'Password must be at least 6 characters', trigger: 'blur' }
  ],
  devinEmailCodeName: [
    { max: 50, message: 'Name can be up to 50 characters', trigger: 'blur' }
  ],
  nickname: [
    { max: 20, message: 'Nickname can be up to 20 characters', trigger: 'blur' }
  ]
};

// Devin Session Token 模式的验证规则
const devinSessionRules: FormRules = {
  sessionToken: [
    { required: true, message: 'Please paste Devin session_token', trigger: 'blur' },
    {
      validator: (_rule, value: string, callback) => {
        const trimmed = (value || '').trim();
        if (!trimmed) return callback(new Error('Please paste Devin session_token'));
        if (!trimmed.startsWith('devin-session-token$')) {
          return callback(new Error('session_token must start with devin-session-token$ prefix'));
        }
        callback();
      },
      trigger: 'blur',
    },
  ],
  nickname: [
    { max: 20, message: 'Nickname can be up to 20 characters', trigger: 'blur' }
  ]
};

// 根据模式选择验证规则
const currentRules = computed(() => {
  // 智能模式复用邮箱密码规则（同样需要 email + password）
  if (addMode.value === 'smart' || addMode.value === 'password') return passwordRules;
  if (addMode.value === 'refresh_token') return refreshTokenRules;
  if (addMode.value === 'devin_session') return devinSessionRules;
  if (addMode.value === 'devin_email_code') {
    if (devinEmailCodeStep.value === 0) return devinEmailCodeStep0Rules;
    // Step 1 按 flow 分流：login 仅验证码，signup 额外要求新密码 + 姓名
    return devinEmailCodeFlow.value === 'signup' ? devinEmailCodeStep1SignupRules : devinEmailCodeStep1Rules;
  }
  return devinRules;
});

// 切换模式时重置表单
function handleModeChange() {
  formRef.value?.resetFields();
  // Devin 邮箱验证码模式专属状态重置
  devinEmailCodeStep.value = 0;
  devinEmailCodeEmailToken.value = '';
  devinEmailCodeFlow.value = 'login';
  formData.devinEmailCodeOtp = '';
  formData.devinEmailCodePassword = '';
  formData.devinEmailCodeName = '';
}

// 获取标签选项样式
function getTagOptionStyle(color: string): Record<string, string> {
  if (!color) return {};
  
  let r = 0, g = 0, b = 0;
  let parsed = false;
  
  // 解析 rgba 或 rgb 格式
  if (color.startsWith('rgba') || color.startsWith('rgb')) {
    const match = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
    if (match) {
      r = parseInt(match[1]);
      g = parseInt(match[2]);
      b = parseInt(match[3]);
      parsed = true;
    }
  } 
  // 解析 HEX 格式
  if (!parsed && color.startsWith('#')) {
    const hex = color.slice(1);
    if (hex.length >= 6) {
      r = parseInt(hex.slice(0, 2), 16);
      g = parseInt(hex.slice(2, 4), 16);
      b = parseInt(hex.slice(4, 6), 16);
      parsed = true;
    }
  }
  
  if (!parsed) return {};
  
  return {
    color: `rgb(${r}, ${g}, ${b})`,
    fontWeight: '500'
  };
}

async function handleSubmit() {
  if (!formRef.value) return;
  
  await formRef.value.validate(async (valid) => {
    if (!valid) return;
    
    loading.value = true;
    try {
      if (addMode.value === 'refresh_token') {
        // Refresh Token 模式
        const trimmedToken = formData.refreshToken.trim();
        const trimmedNickname = formData.nickname.trim() || undefined;
        
        if (!trimmedToken) {
          ElMessage.error('Refresh Token cannot be empty');
          loading.value = false;
          return;
        }
        
        // 调用后端接口添加账号
        const result = await invoke<any>('add_account_by_refresh_token', {
          refreshToken: trimmedToken,
          nickname: trimmedNickname,
          tags: formData.tags,
          group: formData.group || '默认分组'
        });
        
        if (result.success) {
          ElMessage.success(`Account ${result.email} added successfully`);
          // 刷新账号列表
          await accountsStore.loadAccounts();
          handleClose();
        } else {
          ElMessage.error(result.error || 'Failed to add account');
        }
      } else if (addMode.value === 'devin') {
        // Devin 账密模式
        await handleDevinSubmit();
      } else if (addMode.value === 'devin_session') {
        // Devin Session Token 直接迁入
        await handleDevinSessionSubmit();
      } else if (addMode.value === 'devin_email_code') {
        // Devin 邮箱验证码（两步流程）—— 按 step + flow 分派
        if (devinEmailCodeStep.value === 0) {
          await sendDevinEmailCode();
        } else if (devinEmailCodeFlow.value === 'signup') {
          await completeDevinEmailCodeRegister();
        } else {
          await completeDevinEmailCodeLogin();
        }
      } else if (addMode.value === 'smart') {
        // 智能识别模式：先嗅探再分派
        await handleSmartSubmit();
      } else {
        // 邮箱密码模式（旧 Firebase）
        await handleFirebaseSubmit();
      }
    } catch (error) {
      ElMessage.error(`Failed to add account: ${error}`);
    } finally {
      loading.value = false;
    }
  });
}

/**
 * Firebase 邮箱密码登录流程（原 'password' 分支抽取）
 *
 * 供 'password' 模式直接调用，也被 'smart' 模式在嗅探结果为 firebase 时复用
 */
async function handleFirebaseSubmit() {
  const trimmedEmail = formData.email.trim();
  const trimmedPassword = formData.password.trim();
  const trimmedNickname = formData.nickname.trim() || trimmedEmail.split('@')[0];

  if (!trimmedPassword) {
    ElMessage.error('Password cannot be empty or only contain spaces');
    return;
  }

  // 添加账号
  const newAccount = await accountsStore.addAccount({
    email: trimmedEmail,
    password: trimmedPassword,
    nickname: trimmedNickname,
    tags: formData.tags,
    group: formData.group || 'Default Group'
  });

  ElMessage.success('Account added successfully, getting account info...');

  // 自动登录并获取账号详细信息
  try {
    const loginResult = await apiService.loginAccount(newAccount.id);

    if (loginResult.success) {
      const latestAccount = await accountApi.getAccount(newAccount.id);
      await accountsStore.updateAccount(latestAccount);
      ElMessage.success('Account info updated');
    } else {
      ElMessage.warning('Account added, but login failed. Please refresh manually');
    }
  } catch (infoError) {
    console.error('Failed to get account info:', infoError);
    ElMessage.warning('Account added, but failed to get details. Please refresh manually');
  }

  handleClose();
}

/**
 * 智能识别模式：先嗅探账号属于 Firebase / Devin 哪一派，再自动分派
 *
 * 后端 `sniff_login_method` 并发调两侧探测端点，返回 `recommended` 字段：
 * - firebase：走 `handleFirebaseSubmit`
 * - devin：　走 `handleDevinSubmit`
 * - sso / no_password / not_found / blocked：弹对话框指引用户处理
 */
async function handleSmartSubmit() {
  const trimmedEmail = formData.email.trim();
  const trimmedPassword = formData.password.trim();

  if (!trimmedEmail || !trimmedPassword) {
    ElMessage.error('Email and password cannot be empty');
    return;
  }

  ElMessage.info('Detecting account type...');

  let sniff: LoginMethodSniffResult;
  try {
    sniff = await devinApi.sniffLoginMethod(trimmedEmail);
  } catch (e) {
    ElMessage.error(`Failed to detect login method: ${e}`);
    return;
  }

  switch (sniff.recommended) {
    case 'firebase':
      ElMessage.success('Detected as Firebase account, logging in...');
      await handleFirebaseSubmit();
      break;
    case 'devin':
      ElMessage.success('Detected as Devin account, logging in...');
      await handleDevinSubmit();
      break;
    case 'sso':
      // 企业 SSO 账号：有些组织仍允许邮箱验证码登录，提供快捷按钮尝试
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\n\nYou can try to login with email code. If you still cannot receive code, use "Refresh Token" mode.`,
          'Enterprise SSO Account',
          {
            type: 'info',
            confirmButtonText: 'Login with Email Code',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend();
      } catch {
        // 用户取消，不做任何处理
      }
      break;
case 'no_password':
      // 无密码账号：正是"邮箱验证码登录"的主场景
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\n\nThis account can login with email code, no password required. Send code now?`,
          'Account No Password',
          {
            type: 'warning',
            confirmButtonText: 'Send Code',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend();
      } catch {
        // 用户取消
      }
      break;
    case 'not_found':
      // 账号两侧都不存在：直接走"邮箱验证码注册"流程（mode=signup）
      // 不再弹 alert 要用户去别处注册，一步到位
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\n\nThis email is not registered with Devin. Register new account with email code now? Password required in next step.`,
          'Account Not Found',
          {
            type: 'warning',
            confirmButtonText: 'Register Now',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend('signup');
      } catch {
        // 用户取消
      }
      break;
    case 'blocked':
      await ElMessageBox.alert(
        `${sniff.reason}`,
        'Account Blocked',
        { type: 'error', confirmButtonText: 'Got it' }
      ).catch(() => {});
      break;
    default:
      ElMessage.error(`Unknown detection result: ${sniff.recommended}`);
  }
}

/**
 * Devin Session Token 直接迁入流程
 *
 * 用户仅需粘贴 `devin-session-token$...` 即可建号，
 * 后端自动调 GetCurrentUser 反查 email / api_key / 配额 并落库。
 */
async function handleDevinSessionSubmit() {
  const trimmedToken = formData.sessionToken.trim();
  const trimmedNickname = formData.nickname.trim() || undefined;

  if (!trimmedToken) {
    ElMessage.error('Session Token cannot be empty');
    return;
  }
  if (!trimmedToken.startsWith('devin-session-token$')) {
    ElMessage.error('session_token must start with devin-session-token$ prefix');
    return;
  }

  ElMessage.info('Looking up Devin account info...');
  const result = await devinApi.addAccountBySessionToken({
    sessionToken: trimmedToken,
    nickname: trimmedNickname,
    tags: formData.tags,
    group: formData.group || 'Default Group',
  });

  if (result.success) {
    ElMessage.success(`Devin account ${result.email} imported successfully via session_token`);
    await accountsStore.loadAccounts();
    handleClose();
  } else {
    ElMessage.error(result.message || 'Session Token import failed');
  }
}

/**
 * Devin 账密登录的完整流程
 *
 * 1. 调用 addAccountByLogin
 * 2. 若返回 requires_org_selection=true，弹出组织选择对话框
 * 3. 用户选择后调用 addAccountWithOrg 完成创建
 */
async function handleDevinSubmit() {
  const trimmedEmail = formData.email.trim();
  const trimmedPassword = formData.password.trim();
  const trimmedNickname = formData.nickname.trim() || undefined;

  if (!trimmedEmail || !trimmedPassword) {
    ElMessage.error('Email and password cannot be empty');
    return;
  }

  const result = await devinApi.addAccountByLogin({
    email: trimmedEmail,
    password: trimmedPassword,
    nickname: trimmedNickname,
    tags: formData.tags,
    group: formData.group || 'Default Group',
  });

  // 分支 1：需要选择组织
  if (result.requires_org_selection && result.auth1_token && result.orgs) {
    const chosenOrg = await promptOrgSelection(result.orgs);
    if (!chosenOrg) {
      ElMessage.info('Multi-org selection cancelled');
      return;
    }

    const confirmResult = await devinApi.addAccountWithOrg({
      email: trimmedEmail,
      auth1Token: result.auth1_token,
      orgId: chosenOrg,
      nickname: trimmedNickname,
      tags: formData.tags,
      group: formData.group || 'Default Group',
    });

    if (confirmResult.success) {
      ElMessage.success(`Devin account ${trimmedEmail} added successfully`);
      await accountsStore.loadAccounts();
      handleClose();
    } else {
      ElMessage.error(confirmResult.message || 'Failed to create account after org selection');
    }
    return;
  }

  // 分支 2：直接成功
  if (result.success) {
    ElMessage.success(`Devin account ${result.email} added successfully`);
    await accountsStore.loadAccounts();
    handleClose();
  } else {
    ElMessage.error(result.message || 'Devin login failed');
  }
}

/**
 * 多组织选择对话框
 *
 * 使用 ElMessageBox 以最小依赖实现，返回用户选择的 org_id 或 null（取消）
 */
async function promptOrgSelection(orgs: WindsurfOrg[]): Promise<string | null> {
  // 构建选项 HTML（Element Plus 的 MessageBox 支持 dangerouslyUseHTMLString）
  const optionsHtml = orgs
    .map(
      (org, i) => `
        <div style="margin: 8px 0;">
          <label style="display: flex; align-items: center; cursor: pointer;">
            <input type="radio" name="devin-org" value="${escapeHtml(org.id)}" ${i === 0 ? 'checked' : ''} style="margin-right: 8px;" />
            <div>
              <div style="font-weight: 600;">${escapeHtml(org.name) || '(unnamed organization)'}</div>
              <div style="font-size: 11px; color: #909399; font-family: monospace;">${escapeHtml(org.id)}</div>
            </div>
          </label>
        </div>
      `
    )
    .join('');

  try {
    await ElMessageBox({
      title: `This account belongs to ${orgs.length} organizations, please select`,
      message: `<div id="devin-org-picker">${optionsHtml}</div>`,
      dangerouslyUseHTMLString: true,
      showCancelButton: true,
      confirmButtonText: 'Select this org',
      cancelButtonText: 'Cancel',
      closeOnClickModal: false,
    });

    const checked = document.querySelector<HTMLInputElement>(
      '#devin-org-picker input[name="devin-org"]:checked'
    );
    return checked ? checked.value : null;
  } catch {
    return null;
  }
}

/** 转义 HTML 以避免 XSS */
function escapeHtml(s: string): string {
  return (s || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

/**
 * 切换到「Devin 邮箱验证码」模式并自动发送验证码
 *
 * 供智能识别分派失败时的快捷引导：保留用户已输入的邮箱，
 * 自动切换 addMode、重置 step=0、马上发送验证码，进入 step=1等待用户输入。
 *
 * - `flow = 'login'`（默认）：登录已有无密码账号（no_password / sso 分派使用）
 * - `flow = 'signup'`：注册新账号（not_found 分派使用）
 *
 * 外层 handleSmartSubmit 已在 validate 回调内 `loading = true`，本函数无需再管理。
 */
async function switchToEmailCodeModeAndSend(flow: 'login' | 'signup' = 'login') {
  addMode.value = 'devin_email_code';
  devinEmailCodeFlow.value = flow;
  devinEmailCodeStep.value = 0;
  devinEmailCodeEmailToken.value = '';
  formData.devinEmailCodeOtp = '';
  formData.devinEmailCodePassword = '';
  formData.devinEmailCodeName = '';
  // formData.email 保留，不清空

  // 等模式切换后再发验证码（避免 currentRules 切换时的瑕疵触发 validate）
  await nextTick();
  await sendDevinEmailCode();
}

/**
 * Devin 邮箱验证码—— 第 1 步：调 /email/start 发送验证码
 *
 * - login flow：`mode=login` —— 仅对已存在账号有效，服务端不会创建新账号
 * - signup flow：`mode=signup` —— 服务端向邮箱发送注册验证码，后续 `/email/complete` 时创建新账号
 *
 * 成功后更新 step=1，进入验证码输入屏。
 */
async function sendDevinEmailCode() {
  const trimmedEmail = formData.email.trim();
  if (!trimmedEmail) {
    ElMessage.error('Email cannot be empty');
    return;
  }

  const mode = devinEmailCodeFlow.value === 'signup' ? 'signup' : 'login';
  try {
    const resp = await devinApi.emailStart(trimmedEmail, mode, 'Windsurf');
    if (!resp || !resp.email_verification_token) {
      ElMessage.error('Backend did not return email_verification_token, cannot continue');
      return;
    }
    devinEmailCodeEmailToken.value = resp.email_verification_token;
    devinEmailCodeStep.value = 1;
    const hint = mode === 'signup' ? 'Registration verification code sent to' : 'Verification code sent to';
    ElMessage.success(`${hint} ${trimmedEmail}`);
  } catch (e: any) {
    const errMsg = String(e?.message || e || '');
    // login flow 遇到服务端"账号不存在"判定时，引导用户改为 signup flow 并自动重试
    // 覆盖三种场景：
    // 1) radio 主入口直选 devin_email_code 但输入了未注册邮箱
    // 2) sniff_login_method 给出的 no_password / sso 判定与 /email/start 不一致
    // 3) 账号刚被删除/迁移，CheckUserLoginMethod 仍有缓存但 /email/start 已同步
    if (mode === 'login' && /no account found/i.test(errMsg)) {
      try {
        await ElMessageBox.confirm(
          `Server determined this email is not registered with Devin:\n${errMsg}\n\nSwitch to "Email Code Registration" to create new account? Password required in next step.`,
          'Account Not Found',
          {
            type: 'warning',
            confirmButtonText: 'Switch to Register',
            cancelButtonText: 'Got it',
          }
        );
        // 切 flow 后递归一次；signup mode 不会再返回 No account found，不会无限循环
        devinEmailCodeFlow.value = 'signup';
        await sendDevinEmailCode();
      } catch {
        // 用户取消：保持在 step=0，提示原始错误以便用户修正邮箱或切换模式
        ElMessage.info('Cancelled. Please verify email is correct, or use another add method.');
      }
      return;
    }
    ElMessage.error(`Failed to send verification code: ${errMsg}`);
  }
}

  const mode = devinEmailCodeFlow.value === 'signup' ? 'signup' : 'login';
  try {
    const resp = await devinApi.emailStart(trimmedEmail, mode, 'Windsurf');
    if (!resp || !resp.email_verification_token) {
      ElMessage.error('后端未返回 email_verification_token，无法继续');
      return;
    }
    devinEmailCodeEmailToken.value = resp.email_verification_token;
    devinEmailCodeStep.value = 1;
    const hint = mode === 'signup' ? '注册验证码已发送至' : '验证码已发送至';
    ElMessage.success(`${hint} ${trimmedEmail}`);
  } catch (e: any) {
    const errMsg = String(e?.message || e || '');
    // login flow 遇到服务端“账号不存在”判定时，引导用户改为 signup flow 并自动重试
    // 覆盖三种场景：
    // 1) radio 主入口直选 devin_email_code 但输入了未注册邮箱
    // 2) sniff_login_method 给出的 no_password / sso 判定与 /email/start 不一致
    // 3) 账号刚被删除/迁移，CheckUserLoginMethod 仍有缓存但 /email/start 已同步
    if (mode === 'login' && /no account found/i.test(errMsg)) {
      try {
        await ElMessageBox.confirm(
          `服务端判定此邮箱尚未注册 Devin 账号：\n${errMsg}\n\n是否改为「邮箱验证码注册」创建新账号？下一步需要设置密码。`,
          '账号不存在',
          {
            type: 'warning',
            confirmButtonText: '改为注册',
            cancelButtonText: '我知道了',
          }
        );
        // 切 flow 后递归一次；signup mode 不会再返回 No account found，不会无限循环
        devinEmailCodeFlow.value = 'signup';
        await sendDevinEmailCode();
      } catch {
        // 用户取消：保持在 step=0，提示原始错误以便用户修正邮箱或切换模式
        ElMessage.info('已取消。请确认邮箱是否正确，或改用其它添加方式。');
      }
      return;
    }
    ElMessage.error(`发送验证码失败：${errMsg}`);
  }
}

/**
 * Devin 邮箱验证码登录—— 第 2 步：提交验证码，完成登录并建账号
 *
 * - 未设密码的 Devin 账号要走「/email/complete mode=login」，后端命令为
 *   `add_account_by_devin_email_login`（内部自动完成 WindsurfPostAuth + enrich）
 * - 多组织场景复用 `promptOrgSelection` + `addAccountWithOrg`（与 handleDevinSubmit 同步一致）
 */
async function completeDevinEmailCodeLogin() {
  const trimmedEmail = formData.email.trim();
  const otp = formData.devinEmailCodeOtp.trim();
  const trimmedNickname = formData.nickname.trim() || undefined;

  if (!otp) {
    ElMessage.error('Please enter verification code');
    return;
  }
  if (!devinEmailCodeEmailToken.value) {
    ElMessage.error('Session state abnormal, please go back and resend code');
    return;
  }

  const result = await devinApi.addAccountByEmailLogin({
    email: trimmedEmail,
    emailVerificationToken: devinEmailCodeEmailToken.value,
    code: otp,
    nickname: trimmedNickname,
    tags: formData.tags,
    group: formData.group || 'Default Group',
  });

  // 分支 1：需要选择组织
  if (result.requires_org_selection && result.auth1_token && result.orgs) {
    const chosenOrg = await promptOrgSelection(result.orgs);
    if (!chosenOrg) {
      ElMessage.info('Multi-org selection cancelled');
      return;
    }

    const confirmResult = await devinApi.addAccountWithOrg({
      email: trimmedEmail,
      auth1Token: result.auth1_token,
      orgId: chosenOrg,
      nickname: trimmedNickname,
      tags: formData.tags,
      group: formData.group || 'Default Group',
    });

    if (confirmResult.success) {
      ElMessage.success(`Devin account ${trimmedEmail} added successfully`);
      await accountsStore.loadAccounts();
      handleClose();
    } else {
      ElMessage.error(confirmResult.message || 'Failed to create account after org selection');
    }
    return;
  }

  // 分支 2：直接成功
  if (result.success) {
    ElMessage.success(`Devin account ${result.email || trimmedEmail} added successfully`);
    await accountsStore.loadAccounts();
    handleClose();
  } else {
    ElMessage.error(result.message || 'Email code login failed');
  }
}

/**
 * Devin 邮箱验证码注册—— 第 2 步：提交验证码 + 新密码 + 姓名，完成注册并建账号
 *
 * - 调用后端 `add_account_by_devin_register`（内部自动完成注册 + WindsurfPostAuth + enrich）
 * - 多组织场景复用 `promptOrgSelection` + `addAccountWithOrg`（注册流程的原始密码会随二次写入账号卡的 password 字段）
 */
async function completeDevinEmailCodeRegister() {
  const trimmedEmail = formData.email.trim();
  const otp = formData.devinEmailCodeOtp.trim();
  const newPassword = formData.devinEmailCodePassword.trim();
  const displayName =
    formData.devinEmailCodeName.trim() || trimmedEmail.split('@')[0] || 'Devin User';
  const trimmedNickname = formData.nickname.trim() || undefined;

  if (!otp) {
    ElMessage.error('Please enter verification code');
    return;
  }
  if (!newPassword) {
    ElMessage.error('Please set new password');
    return;
  }
  if (!devinEmailCodeEmailToken.value) {
    ElMessage.error('Session state abnormal, please go back and resend code');
    return;
  }

  const result = await devinApi.addAccountByRegister({
    email: trimmedEmail,
    emailVerificationToken: devinEmailCodeEmailToken.value,
    code: otp,
    password: newPassword,
    name: displayName,
    nickname: trimmedNickname,
    tags: formData.tags,
    group: formData.group || 'Default Group',
  });

  // 分支 1：需要选择组织
  if (result.requires_org_selection && result.auth1_token && result.orgs) {
    const chosenOrg = await promptOrgSelection(result.orgs);
    if (!chosenOrg) {
      ElMessage.info('Multi-org selection cancelled');
      return;
    }

    // 注册流程将原始密码随二次选组织入库，便于账号卡回显密码
    const confirmResult = await devinApi.addAccountWithOrg({
      email: trimmedEmail,
      auth1Token: result.auth1_token,
      orgId: chosenOrg,
      nickname: trimmedNickname,
      tags: formData.tags,
      group: formData.group || 'Default Group',
      password: newPassword,
    });

    if (confirmResult.success) {
      ElMessage.success(`Devin account ${trimmedEmail} registered successfully`);
      await accountsStore.loadAccounts();
      handleClose();
    } else {
      ElMessage.error(confirmResult.message || 'Failed to register account after org selection');
    }
    return;
  }

  // 分支 2：直接注册成功
  if (result.success) {
    ElMessage.success(`Devin account ${result.email || trimmedEmail} registered successfully`);
    await accountsStore.loadAccounts();
    handleClose();
  } else {
    ElMessage.error(result.message || 'Email code registration failed');
  }
}

function handleClose() {
  uiStore.closeAddAccountDialog();
  formRef.value?.resetFields();
  
  // 重置表单数据
  formData.email = '';
  formData.password = '';
  formData.refreshToken = '';
  formData.sessionToken = '';
  formData.devinEmailCodeOtp = '';
  formData.devinEmailCodePassword = '';
  formData.devinEmailCodeName = '';
  formData.nickname = '';
  formData.group = '默认分组';
  formData.tags = [];
  addMode.value = 'smart';
  // Devin 邮箱验证码模式状态
  devinEmailCodeStep.value = 0;
  devinEmailCodeEmailToken.value = '';
  devinEmailCodeFlow.value = 'login';
}
</script>

<style scoped>
/* ==================== 添加方式卡片网格（紧凑版） ====================
 * 单行布局：icon + title(flex 1 可省略) + tag(可选) + check(仅选中时)
 * 说明文本仅以原生 tooltip 呈现（见模板 `:title="opt.desc"`），
 * 不占用纵向空间。
 */

/* 外层 2 列网格，窄屏自动降为单列 */
.mode-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
  width: 100%;
}

/* 单张卡片：单行 flex，矮版 34px */
.mode-card {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border: 1.5px solid var(--el-border-color);
  border-radius: 6px;
  background-color: var(--el-bg-color);
  cursor: pointer;
  transition: border-color 0.2s ease, background-color 0.2s ease, box-shadow 0.2s ease;
  user-select: none;
  outline: none;
  min-height: 34px;
}

/* 鼠标悬停：浅主色边框 + 极浅背景 */
.mode-card:hover {
  border-color: var(--el-color-primary-light-3);
  background-color: var(--el-color-primary-light-9);
}

/* 键盘 focus 态 */
.mode-card:focus-visible {
  box-shadow: 0 0 0 2px var(--el-color-primary-light-5);
}

/* 选中态：主色边框 + 浅主色背景 + 外环 */
.mode-card.is-active {
  border-color: var(--el-color-primary);
  background-color: var(--el-color-primary-light-9);
  box-shadow: 0 0 0 2px var(--el-color-primary-light-7);
}

/* 图标：紧凑版与文字基线对齐 */
.mode-card__icon {
  flex-shrink: 0;
  font-size: 18px;
  color: var(--el-color-primary);
  width: 18px;
  height: 18px;
}

/* 标题：占用剩余空间单行省略；字号 13 避免在 2 列 ~220px 下频繁省略 */
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

/* 标签：不收缩，跟在标题后 */
.mode-card__tag {
  flex-shrink: 0;
}

/* 选中勾选：内联放在最右，与 tag 并列；不再用 absolute 避免在紧凑高度下压到文字 */
.mode-card__check {
  flex-shrink: 0;
  font-size: 14px;
  color: var(--el-color-primary);
}

/* 窄屏降级：小窗下单列，避免标题 + 标签撑穷卡片 */
@media (max-width: 520px) {
  .mode-grid {
    grid-template-columns: 1fr;
  }
}
</style>
