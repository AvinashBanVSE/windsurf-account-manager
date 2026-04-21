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

      <!-- Devin Email Code Mode: Two-step process, distinguish login/signup by flow -->
      <template v-else-if="addMode === 'devin_email_code'">
        <!-- Top description: Dynamic text based on flow -->
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

        <!-- Step 0: Enter email -->
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

        <!-- Step 1: Enter verification code (signup flow requires new password + name) -->
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

          <!-- signup flow exclusive fields -->
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

      <!-- Devin Credentials Mode (New Devin Session System) -->
      <template v-else>
        <el-alert
          type="info"
          :closable="false"
          show-icon
          style="margin-bottom: 18px;"
        >
          <template #title>
            <span style="font-size: 12px;">
              Login via the new Devin Session system (<code>/_devin-auth/password/login</code> +
              <code>WindsurfPostAuth</code>), no Google API Key limit, no token refresh needed
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

      <!-- Devin Email Code Mode: Dynamic button based on step -->
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

      <!-- Other modes: Unified "Confirm" button -->
      <el-button v-else type="primary" @click="handleSubmit" :loading="loading">
        Confirm
      </el-button>
    </template>

      <!-- Other modes: Unified "Confirm" button -->
      <el-button v-else type="primary" @click="handleSubmit" :loading="loading">
        Confirm
      </el-button>
        <el-button type="primary" @click="handleSubmit" :loading="loading">
          {{ devinEmailCodeStep === 0
              ? 'Send Code'
              : (devinEmailCodeFlow === 'signup' ? 'Complete Registration' : 'Complete Add') }}
        </el-button>
      </template>

      <!-- Other modes: Unified "Confirm" button -->
      <el-button v-else type="primary" @click="handleSubmit" :loading="loading">
        Confirm
      </el-button>
    </template>

      <!-- Other modes: Unified "Confirm" button -->
      <el-button v-else type="primary" @click="handleSubmit" :loading="loading">
        Confirm
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

// Two-step state for Devin email code login (exclusive to mode === 'devin_email_code')
// step 0: Enter email + send code; step 1: Enter code + complete login/registration
const devinEmailCodeStep = ref<0 | 1>(0);
// email_verification_token returned from /email/start, for subsequent /email/complete
const devinEmailCodeEmailToken = ref('');
// Code sub-flow: login=login to existing passwordless account; signup=register new account
// - Default to 'login' when devin_email_code is selected directly from the main radio entry
// - Automatically set to 'signup' when entering from the quick button dispatched by smart detection's not_found
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
  group: 'Default Group',
  tags: [] as string[]
});

/**
 * Metadata for add method options
 *
 * Ordered by "Recommendation + Genre Aggregation":
 * 1) smart Smart Detect (Recommended, Pinned)
 * 2) Devin series: Credentials / Email Code / session_token (New system, daily driver)
 * 3) Firebase series: Email Password / Refresh Token (Legacy system, compatible with old accounts)
 *
 * Each item carries all visual data required for card rendering (icon, title, tag, one-line description).
 * When adding a new mode, just append an item to this array, and the template grid will render automatically.
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
 * Switch add method
 *
 * Called by the template on card click; directly writes to `addMode` and reuses the original `handleModeChange`
 * cleanup logic (resetting code step / email_token / flow, etc.), ensuring behavior is identical to the el-radio version.
 */
function selectMode(value: string) {
  if (addMode.value === value) return;
  addMode.value = value as typeof addMode.value;
  handleModeChange();
}

// Validation rules for email/password mode
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
  ]\n};

// Validation rules for Refresh Token mode
const refreshTokenRules: FormRules = {
  refreshToken: [
    { required: true, message: 'Please enter Refresh Token', trigger: 'blur' },
    { min: 10, message: 'Invalid Refresh Token format', trigger: 'blur' }
  ],
  nickname: [
    { max: 20, message: 'Nickname can be up to 20 characters', trigger: 'blur' }
  ]
};

// Validation rules for Devin credentials mode (same as passwordRules)
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

// Validation rules for Devin email code mode: grouped by step
// step 0 validates only email, step 1 validates only verification code
// (To prevent users from entering the code before it has been sent)
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
// Step 1 signup sub-flow: verification code + new password (at least 6 chars) + name (optional)
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

// Validation rules for Devin Session Token mode
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

// Select validation rules based on mode
const currentRules = computed(() => {
  // Smart mode reuses email/password rules (also requires email + password)
  if (addMode.value === 'smart' || addMode.value === 'password') return passwordRules;
  if (addMode.value === 'refresh_token') return refreshTokenRules;
  if (addMode.value === 'devin_session') return devinSessionRules;
  if (addMode.value === 'devin_email_code') {
    if (devinEmailCodeStep.value === 0) return devinEmailCodeStep0Rules;
    // Step 1 diverges by flow: login only requires code, signup additionally requires new password + name
    return devinEmailCodeFlow.value === 'signup' ? devinEmailCodeStep1SignupRules : devinEmailCodeStep1Rules;
  }
  return devinRules;
});

// Reset form when switching mode
function handleModeChange() {
  formRef.value?.resetFields();
  // Reset states exclusive to Devin email code mode
  devinEmailCodeStep.value = 0;
  devinEmailCodeEmailToken.value = '';
  devinEmailCodeFlow.value = 'login';
  formData.devinEmailCodeOtp = '';
  formData.devinEmailCodePassword = '';
  formData.devinEmailCodeName = '';
}

// Get tag option style
function getTagOptionStyle(color: string): Record<string, string> {
  if (!color) return {};
  
  let r = 0, g = 0, b = 0;
  let parsed = false;
  
  // Parse rgba or rgb format
  if (color.startsWith('rgba') || color.startsWith('rgb')) {
    const match = color.match(/rgba?\\((\\d+),\\s*(\\d+),\\s*(\\d+)/);
    if (match) {
      r = parseInt(match[1]);
      g = parseInt(match[2]);
      b = parseInt(match[3]);
      parsed = true;
    }
  } 
  // Parse HEX format
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
        // Refresh Token mode
        const trimmedToken = formData.refreshToken.trim();
        const trimmedNickname = formData.nickname.trim() || undefined;
        
        if (!trimmedToken) {
          ElMessage.error('Refresh Token cannot be empty');
          loading.value = false;
          return;
        }
        
        // Call backend API to add account
        const result = await invoke<any>('add_account_by_refresh_token', {
          refreshToken: trimmedToken,
          nickname: trimmedNickname,
          tags: formData.tags,
          group: formData.group || 'Default Group'
        });
        
        if (result.success) {
          ElMessage.success(`Account ${result.email} added successfully`);
          // Refresh account list
          await accountsStore.loadAccounts();
          handleClose();
        } else {
          ElMessage.error(result.error || 'Failed to add account');
        }
      } else if (addMode.value === 'devin') {
        // Devin credentials mode
        await handleDevinSubmit();
      } else if (addMode.value === 'devin_session') {
        // Direct import with Devin Session Token
        await handleDevinSessionSubmit();
      } else if (addMode.value === 'devin_email_code') {
        // Devin email code (two-step process) - dispatched by step + flow
        if (devinEmailCodeStep.value === 0) {
          await sendDevinEmailCode();
        } else if (devinEmailCodeFlow.value === 'signup') {
          await completeDevinEmailCodeRegister();
        } else {
          await completeDevinEmailCodeLogin();
        }
      } else if (addMode.value === 'smart') {
        // Smart detection mode: sniff first, then dispatch
        await handleSmartSubmit();
      } else {
        // Email/password mode (old Firebase)
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
 * Firebase email/password login flow (extracted from original 'password' branch)
 *
 * Called directly by 'password' mode, and also reused by 'smart' mode when sniff result is firebase
 */
async function handleFirebaseSubmit() {
  const trimmedEmail = formData.email.trim();
  const trimmedPassword = formData.password.trim();
  const trimmedNickname = formData.nickname.trim() || trimmedEmail.split('@')[0];

  if (!trimmedPassword) {
    ElMessage.error('Password cannot be empty or only contain spaces');
    return;
  }

  // Add account
  const newAccount = await accountsStore.addAccount({
    email: trimmedEmail,
    password: trimmedPassword,
    nickname: trimmedNickname,
    tags: formData.tags,
    group: formData.group || 'Default Group'
  });

  ElMessage.success('Account added successfully, getting account info...');

  // Automatically log in and get account details
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
 * Smart detection mode: first sniff if the account belongs to Firebase / Devin, then dispatch automatically
 *
 * Backend `sniff_login_method` concurrently calls detection endpoints on both sides, returns `recommended` field:
 * - firebase: use `handleFirebaseSubmit`
 * - devin:    use `handleDevinSubmit`
 * - sso / no_password / not_found / blocked: show dialog to guide user
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
      // Enterprise SSO account: some organizations still allow email code login, provide a quick button to try
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\\n\\nYou can try to login with email code. If you still cannot receive code, use "Refresh Token" mode.`,
          'Enterprise SSO Account',
          {
            type: 'info',
            confirmButtonText: 'Login with Email Code',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend();
      } catch {
        // User cancelled, do nothing
      }
      break;
case 'no_password':
      // Passwordless account: This is the main scenario for "email code login"
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\\n\\nThis account can login with email code, no password required. Send code now?`,
          'Account No Password',
          {
            type: 'warning',
            confirmButtonText: 'Send Code',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend();
      } catch {
        // User cancelled
      }
      break;
    case 'not_found':
      // Account doesn't exist on either side: go directly to "email code registration" flow (mode=signup)
      // No more alert asking user to register elsewhere, do it in one step
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\\n\\nThis email is not registered with Devin. Register new account with email code now? Password required in next step.`,
          'Account Not Found',
          {
            type: 'warning',
            confirmButtonText: 'Register Now',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend('signup');
      } catch {
        // User cancelled
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
 * Direct import flow for Devin Session Token
 *
 * User only needs to paste `devin-session-token$...` to create an account,
 * backend automatically calls GetCurrentUser to look up email / api_key / quota and save to DB.
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
 * Complete flow for Devin credentials login
 *
 * 1. Call addAccountByLogin
 * 2. If it returns requires_org_selection=true, show organization selection dialog
 * 3. After user selection, call addAccountWithOrg to complete creation
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

  // Branch 1: Requires organization selection
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

  // Branch 2: Direct success
  if (result.success) {
    ElMessage.success(`Devin account ${result.email} added successfully`);
    await accountsStore.loadAccounts();
    handleClose();
  } else {
    ElMessage.error(result.message || 'Devin login failed');
  }
}

/**
 * Multi-organization selection dialog
 *
 * Implemented with ElMessageBox for minimal dependency, returns the user's selected org_id or null (if cancelled)
 */
async function promptOrgSelection(orgs: WindsurfOrg[]): Promise<string | null> {
  // Build options HTML (Element Plus's MessageBox supports dangerouslyUseHTMLString)
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

/** Escape HTML to prevent XSS */
function escapeHtml(s: string): string {
  return (s || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/\"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

  // Add account
  const newAccount = await accountsStore.addAccount({
    email: trimmedEmail,
    password: trimmedPassword,
    nickname: trimmedNickname,
    tags: formData.tags,
    group: formData.group || 'Default Group'
  });

  ElMessage.success('Account added successfully, getting account info...');

  // Automatically log in and get account details
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
 * Smart detection mode: first sniff if the account belongs to Firebase / Devin, then dispatch automatically
 *
 * Backend `sniff_login_method` concurrently calls detection endpoints on both sides, returns `recommended` field:
 * - firebase: use `handleFirebaseSubmit`
 * - devin:    use `handleDevinSubmit`
 * - sso / no_password / not_found / blocked: show dialog to guide user
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
      // Enterprise SSO account: some organizations still allow email code login, provide a quick button to try
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\\n\\nYou can try to login with email code. If you still cannot receive code, use "Refresh Token" mode.`,
          'Enterprise SSO Account',
          {
            type: 'info',
            confirmButtonText: 'Login with Email Code',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend();
      } catch {
        // User cancelled, do nothing
      }
      break;
case 'no_password':
      // Passwordless account: This is the main scenario for "email code login"
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\\n\\nThis account can login with email code, no password required. Send code now?`,
          'Account No Password',
          {
            type: 'warning',
            confirmButtonText: 'Send Code',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend();
      } catch {
        // User cancelled
      }
      break;
    case 'not_found':
      // Account doesn't exist on either side: go directly to "email code registration" flow (mode=signup)
      // No more alert asking user to register elsewhere, do it in one step
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\\n\\nThis email is not registered with Devin. Register new account with email code now? Password required in next step.`,
          'Account Not Found',
          {
            type: 'warning',
            confirmButtonText: 'Register Now',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend('signup');
      } catch {
        // User cancelled
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
      // Enterprise SSO account: some organizations still allow email code login, provide a quick button to try
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\\n\\nYou can try to login with email code. If you still cannot receive code, use "Refresh Token" mode.`,
          'Enterprise SSO Account',
          {
            type: 'info',
            confirmButtonText: 'Login with Email Code',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend();
      } catch {
        // User cancelled, do nothing
      }
      break;
case 'no_password':
      // Passwordless account: This is the main scenario for "email code login"
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\\n\\nThis account can login with email code, no password required. Send code now?`,
          'Account No Password',
          {
            type: 'warning',
            confirmButtonText: 'Send Code',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend();
      } catch {
        // User cancelled
      }
      break;
    case 'not_found':
      // Account doesn't exist on either side: go directly to "email code registration" flow (mode=signup)
      // No more alert asking user to register elsewhere, do it in one step
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\\n\\nThis email is not registered with Devin. Register new account with email code now? Password required in next step.`,
          'Account Not Found',
          {
            type: 'warning',
            confirmButtonText: 'Register Now',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend('signup');
      } catch {
        // User cancelled
      }
      break;
case 'no_password':
      // Passwordless account: This is the main scenario for "email code login"
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\\n\\nThis account can login with email code, no password required. Send code now?`,
          'Account No Password',
          {
            type: 'warning',
            confirmButtonText: 'Send Code',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend();
      } catch {
        // User cancelled
      }
      break;
    case 'not_found':
      // Account doesn't exist on either side: go directly to "email code registration" flow (mode=signup)
      // No more alert asking user to register elsewhere, do it in one step
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\\n\\nThis email is not registered with Devin. Register new account with email code now? Password required in next step.`,
          'Account Not Found',
          {
            type: 'warning',
            confirmButtonText: 'Register Now',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend('signup');
      } catch {
        // User cancelled
      }
      break;
case 'not_found':
      // Account doesn't exist on either side: go directly to "email code registration" flow (mode=signup)
      // No more alert asking user to register elsewhere, do it in one step
      try {
        await ElMessageBox.confirm(
          `${sniff.reason}\\n\\nThis email is not registered with Devin. Register new account with email code now? Password required in next step.`,
          'Account Not Found',
          {
            type: 'warning',
            confirmButtonText: 'Register Now',
            cancelButtonText: 'Got it',
          }
        );
        await switchToEmailCodeModeAndSend('signup');
      } catch {
        // User cancelled
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
 * Direct import flow for Devin Session Token
 *
 * User only needs to paste `devin-session-token$...` to create an account,
 * backend automatically calls GetCurrentUser to look up email / api_key / quota and save to DB.
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
 * Complete flow for Devin credentials login
 *
 * 1. Call addAccountByLogin
 * 2. If it returns requires_org_selection=true, show organization selection dialog
 * 3. After user selection, call addAccountWithOrg to complete creation
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

  // Branch 1: Requires organization selection
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

  // Branch 2: Direct success
  if (result.success) {
    ElMessage.success(`Devin account ${result.email} added successfully`);
    await accountsStore.loadAccounts();
    handleClose();
  } else {
    ElMessage.error(result.message || 'Devin login failed');
  }
}

/**
 * Multi-organization selection dialog
 *
 * Implemented with ElMessageBox for minimal dependency, returns the user's selected org_id or null (if cancelled)
 */
async function promptOrgSelection(orgs: WindsurfOrg[]): Promise<string | null> {
  // Build options HTML (Element Plus's MessageBox supports dangerouslyUseHTMLString)
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

/** Escape HTML to prevent XSS */
function escapeHtml(s: string): string {
  return (s || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/\"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

/**
 * Switch to "Devin Email Code" mode and send code automatically
 *
 * Provides a shortcut guide when smart detection dispatch fails: keeps the user's entered email,
 * automatically switches addMode, resets to step=0, sends the code immediately, and proceeds to step=1 to await user input.
 *
 * - `flow = 'login'` (default): Login to existing passwordless account (used by no_password / sso dispatch)
 * - `flow = 'signup'`: Register a new account (used by not_found dispatch)
 *
 * The outer handleSmartSubmit has already set `loading = true` within the validate callback, so this function doesn't need to manage it.
 */
async function switchToEmailCodeModeAndSend(flow: 'login' | 'signup' = 'login') {
  addMode.value = 'devin_email_code';
  devinEmailCodeFlow.value = flow;
  devinEmailCodeStep.value = 0;
  devinEmailCodeEmailToken.value = '';
  formData.devinEmailCodeOtp = '';
  formData.devinEmailCodePassword = '';
  formData.devinEmailCodeName = '';
  // formData.email is kept, not cleared

  // Send verification code after mode switch (to avoid validate being triggered by currentRules switch glitch)
  await nextTick();
  await sendDevinEmailCode();
}

/**
 * Devin Email Code - Step 1: Call /email/start to send verification code
 *
 * - login flow: `mode=login` - only valid for existing accounts, server will not create a new account
 * - signup flow: `mode=signup` - server sends registration code to email, a new account is created during the subsequent `/email/complete`
 *
 * On success, update to step=1 and show the code input screen.
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
    // When login flow encounters "account not found" from server, guide user to switch to signup flow and retry automatically
    // Covers three scenarios:
    // 1) Directly selecting devin_email_code from radio entry but entering an unregistered email
    // 2) The no_password / sso determination from sniff_login_method is inconsistent with /email/start
    // 3) Account was just deleted/migrated, CheckUserLoginMethod still has cache but /email/start is updated
    if (mode === 'login' && /no account found/i.test(errMsg)) {
      try {
        await ElMessageBox.confirm(
          `Server determined this email is not registered with Devin:\\n${errMsg}\\n\\nSwitch to "Email Code Registration" to create new account? Password required in next step.`,
          'Account Not Found',
          {
            type: 'warning',
            confirmButtonText: 'Switch to Register',
            cancelButtonText: 'Got it',
          }
        );
        // Recurse once after switching flow; signup mode won't return "No account found", so no infinite loop
        devinEmailCodeFlow.value = 'signup';
        await sendDevinEmailCode();
      } catch {
        // User cancelled: remain at step=0, show original error to allow user to correct email or switch mode
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
      ElMessage.error('Backend did not return email_verification_token, cannot continue');
      return;
    }
    devinEmailCodeEmailToken.value = resp.email_verification_token;
    devinEmailCodeStep.value = 1;
    const hint = mode === 'signup' ? 'Registration verification code sent to' : 'Verification code sent to';
    ElMessage.success(`${hint} ${trimmedEmail}`);
  } catch (e: any) {
    const errMsg = String(e?.message || e || '');
    // When login flow encounters "account not found" from server, guide user to switch to signup flow and retry automatically
    // Covers three scenarios:
    // 1) Directly selecting devin_email_code from radio entry but entering an unregistered email
    // 2) The no_password / sso determination from sniff_login_method is inconsistent with /email/start
    // 3) Account was just deleted/migrated, CheckUserLoginMethod still has cache but /email/start is updated
    if (mode === 'login' && /no account found/i.test(errMsg)) {
      try {
        await ElMessageBox.confirm(
          `Server determined this email is not registered with Devin:\\n${errMsg}\\n\\nSwitch to "Email Code Registration" to create new account? Password required in next step.`,
          'Account Not Found',
          {
            type: 'warning',
            confirmButtonText: 'Switch to Register',
            cancelButtonText: 'Got it',
          }
        );
        // Recurse once after switching flow; signup mode won't return "No account found", so no infinite loop
        devinEmailCodeFlow.value = 'signup';
        await sendDevinEmailCode();
      } catch {
        // User cancelled: remain at step=0, show original error to allow user to correct email or switch mode
        ElMessage.info('Cancelled. Please verify email is correct, or use another add method.');
      }
      return;
    }
    ElMessage.error(`Failed to send verification code: ${errMsg}`);
  }
}

/**
 * Devin Email Code Login - Step 2: Submit code to complete login and create account
 *
 * - Devin accounts without a password must use "/email/complete mode=login", the backend command is
 *   `add_account_by_devin_email_login` (which automatically completes WindsurfPostAuth + enrich)
 * - Multi-org scenarios reuse `promptOrgSelection` + `addAccountWithOrg` (consistent with handleDevinSubmit)
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

  // Branch 1: Requires organization selection
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

  // Branch 2: Direct success
  if (result.success) {
    ElMessage.success(`Devin account ${result.email || trimmedEmail} added successfully`);
    await accountsStore.loadAccounts();
    handleClose();
  } else {
    ElMessage.error(result.message || 'Email code login failed');
  }
}

/**
 * Devin Email Code Registration - Step 2: Submit code + new password + name to complete registration and create account
 *
 * - Calls backend `add_account_by_devin_register` (which automatically completes registration + WindsurfPostAuth + enrich)
 * - Multi-org scenarios reuse `promptOrgSelection` + `addAccountWithOrg` (the original password from the registration flow is saved to the account card's password field on the second write)
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

  // Branch 1: Requires organization selection
  if (result.requires_org_selection && result.auth1_token && result.orgs) {
    const chosenOrg = await promptOrgSelection(result.orgs);
    if (!chosenOrg) {
      ElMessage.info('Multi-org selection cancelled');
      return;
    }

    // The registration flow saves the original password along with the second org selection, making it easier to display the password on the account card
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

  // Branch 2: Direct registration success
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
  
  // Reset form data
  formData.email = '';
  formData.password = '';
  formData.refreshToken = '';
  formData.sessionToken = '';
  formData.devinEmailCodeOtp = '';
  formData.devinEmailCodePassword = '';
  formData.devinEmailCodeName = '';
  formData.nickname = '';
  formData.group = 'Default Group';
  formData.tags = [];
  addMode.value = 'smart';
  // Devin email code mode state
  devinEmailCodeStep.value = 0;
  devinEmailCodeEmailToken.value = '';
  devinEmailCodeFlow.value = 'login';
}
</script>

<style scoped>
/* ==================== Add Method Card Grid (Compact) ==================== * 
 * Single-line layout: icon + title(flex 1, can be omitted) + tag(optional) + check(only when selected)
 * Description text is only shown as a native tooltip (see template `:title="opt.desc"`),
 * does not take up vertical space.
 */

/* Outer 2-column grid, automatically collapses to single column on narrow screens */
.mode-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 8px;
  width: 100%;
}

/* Single card: single-line flex, short version 34px */
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

/* Mouse hover: light primary border + very light background */
.mode-card:hover {
  border-color: var(--el-color-primary-light-3);
  background-color: var(--el-color-primary-light-9);
}

/* Keyboard focus state */
.mode-card:focus-visible {
  box-shadow: 0 0 0 2px var(--el-color-primary-light-5);
}

/* Selected state: primary border + light primary background + outer ring */
.mode-card.is-active {
  border-color: var(--el-color-primary);
  background-color: var(--el-color-primary-light-9);
  box-shadow: 0 0 0 2px var(--el-color-primary-light-7);
}

/* Icon: compact version aligned with text baseline */
.mode-card__icon {
  flex-shrink: 0;
  font-size: 18px;
  color: var(--el-color-primary);
  width: 18px;
  height: 18px;
}

/* Title: occupies remaining space, single line with ellipsis; font size 13 to avoid frequent ellipsis in 2 columns ~220px */
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

/* Tag: no shrink, follows title */
.mode-card__tag {
  flex-shrink: 0;
}

/* Selected checkmark: inline on the far right, parallel to tag; no longer use absolute to avoid overlapping text in compact height */
.mode-card__check {
  flex-shrink: 0;
  font-size: 14px;
  color: var(--el-color-primary);
}

/* Narrow screen fallback: single column in small windows to prevent title + tag from stretching the card */
@media (max-width: 520px) {
  .mode-grid {
    grid-template-columns: 1fr;
  }
}
</style>
