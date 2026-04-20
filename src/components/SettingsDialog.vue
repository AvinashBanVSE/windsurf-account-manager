<template>
  <el-dialog
    v-model="uiStore.showSettingsDialog"
    title="Settings"
    width="700px"
  >
    <el-tabs v-model="activeTab" type="border-card">
      <!-- Basic Settings Tab -->
      <el-tab-pane label="Basic" name="basic">
        <el-form :model="settings" label-width="140px">
          <el-form-item label="Auto Refresh Token">
            <el-switch v-model="settings.auto_refresh_token" />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              When enabled, will auto-refresh token when expired
            </div>
          </el-form-item>
          
          <el-form-item label="Unlimited Concurrent Refresh" v-if="settings.auto_refresh_token">
            <el-switch v-model="settings.unlimitedConcurrentRefresh" />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              When enabled, all accounts refresh concurrently without limit to save time
            </div>
          </el-form-item>
          
          <!-- Seat Count Options - disabled in simple version
          <el-form-item label="Seat Count Options">
            <el-input
              v-model="seatCountOptionsInput"
              placeholder="e.g., 18, 19, 20"
              style="width: 200px;"
              @blur="parseSeatCountOptions"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              Seat count for rotation during credit reset, comma separated (e.g., 18, 19, 20)
            </div>
          </el-form-item>
          -->
          
          <el-form-item label="Retry Times">
            <el-input-number
              v-model="settings.retry_times"
              :min="1"
              :max="5"
              :step="1"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              Retry count when API call fails
            </div>
          </el-form-item>
          
          <el-form-item label="Concurrent Limit">
            <el-input-number
              v-model="settings.concurrent_limit"
              :min="1"
              :max="10"
              :step="1"
              :disabled="settings.unlimitedConcurrentRefresh"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              {{ settings.unlimitedConcurrentRefresh ? 'Unlimited concurrent refresh enabled, this setting does not affect auto refresh' : 'Max concurrent for batch operations' }}
            </div>
          </el-form-item>
          
          <el-form-item label="Theme">
            <el-radio-group v-model="settings.theme">
              <el-radio-button label="light">Light</el-radio-button>
              <el-radio-button label="dark">Dark</el-radio-button>
            </el-radio-group>
          </el-form-item>
          
          <el-form-item label="Show Detailed Results">
            <el-switch 
              v-model="settings.show_seats_result_dialog"
              active-text="On"
              inactive-text="Off"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              When enabled, shows detailed seat update results during credit reset
            </div>
          </el-form-item>
          
          <el-form-item label="Privacy Mode">
            <el-switch 
              v-model="settings.privacyMode"
              active-text="On"
              inactive-text="Off"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              When enabled, all emails display as random characters to protect privacy (for screenshots)
            </div>
          </el-form-item>
          
          <el-divider content-position="left">Network Maintenance</el-divider>
          
          <el-form-item label="Lightweight API">
            <el-switch 
              v-model="settings.useLightweightApi"
              active-text="On"
              inactive-text="Off"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              When enabled, uses GetPlanStatus to get quota info (faster); when off, uses GetCurrentUser (more complete data)
            </div>
          </el-form-item>
          
          <el-form-item label="Enable Proxy">
            <el-switch 
              v-model="settings.proxyEnabled"
              active-text="On"
              inactive-text="Off"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              When enabled, login and refresh Token Google API requests will go through proxy
            </div>
          </el-form-item>
          
          <el-form-item label="Proxy Address" v-if="settings.proxyEnabled">
            <el-input
              v-model="settings.proxyUrl"
              placeholder="http://127.0.0.1:7890"
              style="width: 280px;"
              clearable
            >
              <template #prefix>
                <el-icon><Connection /></el-icon>
              </template>
            </el-input>
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              Supports HTTP/HTTPS/SOCKS5 proxy, format: http://host:port or socks5://host:port
            </div>
          </el-form-item>
          
          <el-form-item label="Reset Network Connection">
            <el-button 
              type="warning" 
              @click="handleResetHttpClient"
              :loading="resettingHttp"
            >
              Reset HTTP Client
            </el-button>
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              When encountering continuous API request failures, click this button to reset network connection pool
            </div>
          </el-form-item>
        </el-form>
      </el-tab-pane>
      
      <!-- Payment Settings Tab -->
      <el-tab-pane label="Payment Settings" name="payment">
        <el-form :model="settings" label-width="140px">
          <el-divider content-position="left">Subscription Plan Settings</el-divider>
          
          <el-form-item label="Subscription Plan">
            <el-select v-model="settings.subscriptionPlan" style="width: 100%;">
              <el-option-group label="Windsurf Common">
                <el-option label="Pro" :value="2" />
                <el-option label="Max" :value="18" />
                <el-option label="Teams" :value="1" />
                <el-option label="Trial" :value="9" />
                <el-option label="Free" :value="0" />
              </el-option-group>
              <el-option-group label="Windsurf Ultimate">
                <el-option label="Pro Ultimate" :value="8" />
                <el-option label="Teams Ultimate" :value="7" />
              </el-option-group>
              <el-option-group label="Enterprise">
                <el-option label="Enterprise SaaS" :value="3" />
                <el-option label="Enterprise Self-Serve" :value="10" />
                <el-option label="Enterprise Self-Hosted" :value="5" />
                <el-option label="Enterprise SaaS Pooled" :value="11" />
                <el-option label="Hybrid" :value="4" />
              </el-option-group>
              <el-option-group label="Devin">
                <el-option label="Devin Pro" :value="16" />
                <el-option label="Devin Max" :value="17" />
                <el-option label="Devin Teams" :value="14" />
                <el-option label="Devin Teams V2" :value="15" />
                <el-option label="Devin Enterprise" :value="12" />
                <el-option label="Devin Free" :value="19" />
                <el-option label="Devin Trial" :value="20" />
              </el-option-group>
              <el-option-group label="Other">
                <el-option label="Waitlist Pro" :value="6" />
              </el-option-group>
            </el-select>
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              Select plan type to subscribe, Pro plan requires Turnstile verification
            </div>
          </el-form-item>
          
          <el-form-item label="Payment Period">
            <el-select v-model="settings.paymentPeriod" style="width: 100%;">
              <el-option label="Monthly" :value="1" />
              <el-option label="Yearly" :value="2" />
            </el-select>
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              Yearly payment usually has discount
            </div>
          </el-form-item>
          
          <el-form-item label="Start Trial">
            <el-switch 
              v-model="settings.startTrial"
              active-text="On"
              inactive-text="Off"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              Start with trial, Off means direct paid subscription
            </div>
          </el-form-item>
          
          <el-form-item label="Team Name" v-if="[1, 3, 4, 5, 7, 10, 11, 12, 14, 15].includes(settings.subscriptionPlan)">
            <el-input 
              v-model="settings.teamName" 
              placeholder="Enter team name (required for Teams plan)"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              Teams plan requires team name
            </div>
          </el-form-item>
          
          <el-form-item label="Seat Count" v-if="[1, 3, 4, 5, 7, 10, 11, 12, 14, 15].includes(settings.subscriptionPlan)">
            <el-input-number 
              v-model="settings.seatCount" 
              :min="1" 
              :max="1000"
              style="width: 100%;"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              Seat count for Teams plan
            </div>
          </el-form-item>
          
          <el-divider content-position="left">Payment Page Settings</el-divider>
          
          <el-form-item label="Auto Open Payment Page">
            <el-switch 
              v-model="settings.autoOpenPaymentLinkInWebview"
              active-text="On"
              inactive-text="Off"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              When enabled, automatically opens payment page in built-in browser when getting card binding link succeeds (privacy mode, no data saved)
            </div>
          </el-form-item>
          
          <el-divider content-position="left">External Browser Settings</el-divider>
          
          <el-form-item label="Auto Open External Browser">
            <el-switch 
              v-model="settings.autoOpenBrowser"
              active-text="On"
              inactive-text="Off"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              When enabled, automatically opens in external browser when getting card binding link (no need to click confirm)
            </div>
          </el-form-item>
          
          <el-form-item label="Browser Mode">
            <el-radio-group v-model="settings.browserMode">
              <el-radio-button label="incognito">Incognito Mode</el-radio-button>
              <el-radio-button label="normal">Normal Mode</el-radio-button>
            </el-radio-group>
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              Select mode for external browser (incognito is safer, recommended)
            </div>
          </el-form-item>
          
          <el-divider content-position="left">Auto Fill Settings</el-divider>
          
          <el-form-item label="Auto Fill Payment Form">
            <el-switch 
              v-model="settings.autoFillPaymentForm"
              active-text="On"
              inactive-text="Off"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              When enabled, automatically fills Stripe payment form with virtual card info (for testing only)
            </div>
          </el-form-item>
          
          <el-form-item label="Show Virtual Card Info">
            <el-switch 
              v-model="settings.showVirtualCardInfo"
              active-text="On"
              inactive-text="Off"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              When enabled, shows popup with generated virtual card info when auto-filling form
            </div>
          </el-form-item>
          
          <el-form-item label="Auto Submit Form">
            <el-switch 
              v-model="settings.autoSubmitPaymentForm"
              active-text="On"
              inactive-text="Off"
              :disabled="!settings.autoFillPaymentForm"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              After enabling, form will auto-submit after completion (use with caution)
            </div>
          </el-form-item>
          
          <el-form-item label="Payment Page Delay (seconds)">
            <el-input-number
              v-model="settings.paymentPageDelay"
              :min="1"
              :max="10"
              :step="1"
              :disabled="!settings.autoFillPaymentForm"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              Wait seconds before auto-filling form
            </div>
          </el-form-item>
          
          <el-form-item label="Custom Card BIN">
            <el-input
              v-model="settings.customCardBin"
              placeholder="Enter 4-12 digit number"
              maxlength="12"
              @input="validateCardBin"
            >
              <template #append>
                <el-button @click="resetCardBin">Reset Default</el-button>
              </template>
            </el-input>
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              Set virtual card prefix (BIN), must be 4-12 digits, default 626202
            </div>
          </el-form-item>
          
          <el-form-item label="Card BIN Range (optional)">
            <el-input
              v-model="settings.customCardBinRange"
              placeholder="e.g., 626200-626300"
              @input="validateCardBinRange"
            >
              <template #append>
                <el-button @click="clearCardBinRange">Clear</el-button>
              </template>
            </el-input>
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              After setting BIN range, will randomly select from range. Format: startBIN-endBIN
            </div>
          </el-form-item>
          
          <el-form-item label="Card Bind Retry Times">
            <el-input-number
              v-model="settings.cardBindRetryTimes"
              :min="0"
              :max="20"
              :step="1"
              controls-position="right"
            />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              Retry count after card bind failure, set 0 to not retry
            </div>
          </el-form-item>
          
          <el-divider content-position="left">Card BIN Pool Function</el-divider>
          
          <el-form-item label="Test Mode">
            <div style="display: flex; align-items: center; gap: 10px;">
              <el-switch v-model="settings.testModeEnabled" />
              <el-button 
                size="small" 
                type="warning" 
                @click="resetTestModeProgress"
                :disabled="!testModeProgress"
              >
                Reset Progress
              </el-button>
            </div>
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              After enabling, iterate through BIN range and collect successful BINs (pool count: {{ successBinCount }})
              <span v-if="testModeProgress" style="color: #67C23A;">
                <br/>Current Progress: {{ testModeProgress }}
              </span>
            </div>
          </el-form-item>
          
          <el-form-item label="Use Local BIN Pool">
            <el-switch v-model="settings.useLocalSuccessBins" :disabled="successBinCount === 0" />
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              After enabling, randomly get BIN from local success pool
            </div>
          </el-form-item>
          
          <el-form-item label="BIN Pool Management">
            <el-button-group>
              <el-button size="small" @click="viewSuccessBins" :disabled="successBinCount === 0">
                View BIN Pool
              </el-button>
              <el-button size="small" type="danger" @click="clearSuccessBins" :disabled="successBinCount === 0">
                Clear BIN Pool
              </el-button>
            </el-button-group>
          </el-form-item>
          
          <el-alert
            title="Important Notice"
            type="warning"
            :closable="false"
            show-icon
            style="margin-top: 20px;"
          >
            <template #default>
              <div style="font-size: 12px; line-height: 1.6;">
                <p>🔒 Built-in browser uses privacy mode, won't save browsing data</p>
                <p>⚠️ Virtual card generation is for testing only, don't use for real payments</p>
                <p>⚠️ Ensure compliance with Stripe terms when using this feature</p>
                <p>⚠️ Don't use virtual cards for fraud or illegal purposes</p>
              </div>
            </template>
          </el-alert>
        </el-form>
      </el-tab-pane>
      
      <!-- Seamless Switch Tab -->
      <el-tab-pane label="Seamless Switch" name="seamless">
        <el-form :model="settings" label-width="140px">
          <el-form-item label="Client Type">
            <el-select
              v-model="settings.windsurfClientType"
              style="width: 200px;"
              @change="handleClientTypeChange"
            >
              <el-option label="Windsurf" value="windsurf" />
              <el-option label="Windsurf - Next" value="windsurf-next" />
            </el-select>
          </el-form-item>
          
          <el-form-item label="Install Path">
            <el-input
              v-model="windsurfPath"
              placeholder="Click auto-detect or enter path"
              @blur="handlePathChange"
            >
              <template #append>
                <el-button-group>
                  <el-button @click="detectWindsurfPath" :loading="detectingPath">
                    Auto-detect
                  </el-button>
                  <el-button @click="browseWindsurfPath">
                    Browse
                  </el-button>
                </el-button-group>
              </template>
            </el-input>
            <div style="margin-top: 5px; color: #909399; font-size: 12px;">
              Manually enter path or auto-detect from Start menu for {{ settings.windsurfClientType === 'windsurf-next' ? 'Windsurf - Next' : 'Windsurf' }} install path
            </div>
          </el-form-item>
          
          <el-form-item label="Enable Seamless Switch">
            <el-switch 
              v-model="settings.seamlessSwitchEnabled"
              active-text="On"
              inactive-text="Off"
              :loading="patchLoading"
              @change="handleSeamlessSwitch"
              :disabled="!windsurfPath"
            />
          </el-form-item>
          
          <el-form-item label="Patch Status">
            <div class="patch-status-block">
              <!-- Summary tag + action buttons -->
              <div class="patch-status-header">
                <el-tag :type="patchSummary.type">{{ patchSummary.label }}</el-tag>
                <el-button
                  v-if="canUpgrade"
                  type="warning"
                  size="small"
                  :loading="patchLoading"
                  @click="handleUpgradePatch"
                >
                  Upgrade Patch
                </el-button>
                <el-button
                  v-if="windsurfPath"
                  size="small"
                  @click="checkPatchStatus"
                >
                  Re-detect
                </el-button>
              </div>
              <!-- Sub-item checklist (show when path exists and no IO error) -->
              <div
                v-if="windsurfPath && !patchStatus.error"
                class="patch-checklist"
              >
                <div
                  v-for="item in patchItems"
                  :key="item.key"
                  class="patch-checklist-item"
                  :class="{ 'is-applied': item.applied }"
                >
                  <el-icon v-if="item.applied" class="patch-checklist-icon is-applied">
                    <Check />
                  </el-icon>
                  <el-icon v-else class="patch-checklist-icon">
                    <Close />
                  </el-icon>
                  <span>{{ item.label }}</span>
                </div>
              </div>
            </div>
          </el-form-item>
          
          <el-alert
            title="Feature Description"
            type="info"
            :closable="false"
            show-icon
            style="margin-top: 20px;"
          >
            <template #default>
              <div style="font-size: 12px; line-height: 1.6;">
                <p>🚀 Seamless switch: Switch Windsurf accounts without user awareness</p>
                <p>⚠️ Note: Auto restart if client running when enable/disable, no restart if not running</p>
              </div>
            </template>
          </el-alert>
          
          <el-divider content-position="left">Windsurf Geek Feature</el-divider>
          
          <el-form-item label="Enable Geek Feature">
            <el-switch 
              v-model="settings.cunzhiEnabled"
              active-text="On"
              inactive-text="Off"
              :loading="cunzhiLoading"
              @change="handleCunzhiSwitch"
            />
          </el-form-item>
          
          <el-form-item label="Geek Feature Status">
            <el-tag v-if="cunzhiStatus.installed" type="success">Installed</el-tag>
            <el-tag v-else-if="cunzhiStatus.error" type="danger">{{ cunzhiStatus.error }}</el-tag>
            <el-tag v-else type="info">Not Installed</el-tag>
            <el-button 
              v-if="cunzhiStatus.installed" 
              size="small" 
              style="margin-left: 10px;"
              @click="checkCunzhiStatus"
            >
              Re-detect
            </el-button>
          </el-form-item>
          
          <el-alert
            title="Geek Feature Description"
            type="success"
            :closable="false"
            show-icon
            style="margin-top: 10px;"
          >
            <template #default>
              <div style="font-size: 12px; line-height: 1.6;">
                <p>💊 Geek Feature: Prevent AI from ending conversations automatically</p>
                <p>⚠️ Note: Restart Windsurf after enable/disable to take effect</p>
              </div>
            </template>
          </el-alert>
        </el-form>
      </el-tab-pane>
      
      <!-- Backup Settings Tab -->
      <el-tab-pane label="Backup Settings" name="backup">
        <el-form :model="settings" label-width="140px">
          <el-form-item label="Auto Backup">
            <el-switch v-model="settings.autoBackupEnabled" />
            <span style="margin-left: 10px; color: #909399; font-size: 12px;">
              When enabled, automatically backs up data at set intervals
            </span>
          </el-form-item>
          
          <el-form-item label="Backup Interval">
            <el-input-number
              v-model="settings.backupInterval"
              :min="1"
              :max="1440"
              :step="5"
              :disabled="!settings.autoBackupEnabled"
            />
            <span style="margin-left: 10px; color: #909399;">minutes</span>
          </el-form-item>
          
          <el-form-item label="Max Backups">
            <el-input-number
              v-model="settings.backupMaxCount"
              :min="1"
              :max="100"
            />
            <span style="margin-left: 10px; color: #909399;">copies (automatically deletes oldest when exceeded)</span>
          </el-form-item>
          
          <el-divider content-position="left">Manual Operations</el-divider>
          
          <el-form-item label="Backup Now">
            <el-button type="primary" @click="handleManualBackup" :loading="backupLoading">
              Create Backup
            </el-button>
          </el-form-item>
          
          <el-form-item label="Backup List">
            <el-button @click="handleShowBackups" :loading="loadingBackups">
              View Backups
            </el-button>
          </el-form-item>
          
          <el-alert type="info" :closable="false" style="margin-top: 15px;">
            <template #title>
              <span style="font-weight: bold;">Backup Instructions</span>
            </template>
            <template #default>
              <div style="line-height: 1.8;">
                <p>Backup files are stored in the <code>backups</code> folder in the app data directory</p>
                <p>Includes: account info, groups, tags, settings, etc.</p>
              </div>
            </template>
          </el-alert>
        </el-form>
        
        <!-- Backup List Dialog -->
        <el-dialog
          v-model="showBackupsDialog"
          title="Backup List"
          width="600px"
          append-to-body
        >
          <el-table :data="backupList" v-loading="loadingBackups" max-height="400">
            <el-table-column prop="name" label="File Name" />
            <el-table-column label="Size" width="100">
              <template #default="{ row }">
                {{ formatFileSize(row.size) }}
              </template>
            </el-table-column>
            <el-table-column label="Created Time" width="180">
              <template #default="{ row }">
                {{ formatBackupTime(row.name) }}
              </template>
            </el-table-column>
            <el-table-column label="Actions" width="120">
              <template #default="{ row }">
                <el-button type="primary" size="small" link @click="handleRestoreBackup(row)">
                  Restore
                </el-button>
                <el-button type="danger" size="small" link @click="handleDeleteBackup(row)">
                  Delete
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-dialog>
      </el-tab-pane>
    </el-tabs>
    
    <template #footer>
      <el-button @click="handleClose">Cancel</el-button>
      <el-button type="primary" @click="handleSave" :loading="loading">
        Save
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, watch, onMounted, computed } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Connection, Check, Close } from '@element-plus/icons-vue';
import { useSettingsStore, useUIStore } from '@/store';
import { invoke } from '@tauri-apps/api/core';
import { systemApi } from '@/api';

const settingsStore = useSettingsStore();
const uiStore = useUIStore();

const loading = ref(false);
const activeTab = ref('basic');  // Current active tab
const seatCountOptionsInput = ref('18, 19, 20');  // Seat count options input
const resettingHttp = ref(false);  // HTTP client resetting

// Parse seat count options
function parseSeatCountOptions() {
  const input = seatCountOptionsInput.value.trim();
  if (!input) {
    settings.seat_count_options = [18, 19, 20];
    seatCountOptionsInput.value = '18, 19, 20';
    return;
  }
  
  const numbers = input.split(/[,，\s]+/)
    .map(s => parseInt(s.trim(), 10))
    .filter(n => !isNaN(n) && n > 0);
  
  if (numbers.length === 0) {
    ElMessage.warning('Please enter valid seat numbers');
    settings.seat_count_options = [18, 19, 20];
    seatCountOptionsInput.value = '18, 19, 20';
  } else {
    settings.seat_count_options = numbers;
    seatCountOptionsInput.value = numbers.join(', ');
  }
}

const settings = reactive<{
  auto_refresh_token: boolean;
  seat_count_options: number[];
  retry_times: number;
  theme: string;
  concurrent_limit: number;
  show_seats_result_dialog: boolean;
  autoOpenPaymentLinkInWebview: boolean;
  autoFillPaymentForm: boolean;
  autoSubmitPaymentForm: boolean;
  paymentPageDelay: number;
  showVirtualCardInfo: boolean;
  customCardBin: string;
  customCardBinRange: string;
  cardBindRetryTimes: number;
  testModeEnabled: boolean;
  useLocalSuccessBins: boolean;
  seamlessSwitchEnabled: boolean;
  windsurfClientType: 'windsurf' | 'windsurf-next';
  windsurfPath: string | null;
  patchBackupPath: string | null;
  autoOpenBrowser: boolean;
  browserMode: 'incognito' | 'normal';
  privacyMode: boolean;
  unlimitedConcurrentRefresh: boolean;
  proxyEnabled: boolean;
  proxyUrl: string | null;
  useLightweightApi: boolean;
  subscriptionPlan: number;
  paymentPeriod: number;
  startTrial: boolean;
  teamName: string;
  seatCount: number;
  cunzhiEnabled: boolean;
  autoBackupEnabled: boolean;
  backupInterval: number;
  backupMaxCount: number;
}>({
  auto_refresh_token: true,
  seat_count_options: [18, 19, 20],
  retry_times: 2,
  theme: 'light',
  concurrent_limit: 5,
  show_seats_result_dialog: false,  // Default off
  autoOpenPaymentLinkInWebview: false,  // Default off auto open payment page
  autoFillPaymentForm: false,  // Default off auto fill form
  autoSubmitPaymentForm: false,  // Default off auto submit
  paymentPageDelay: 2,  // Default delay 2 seconds
  showVirtualCardInfo: false,  // Default off virtual card info popup
  customCardBin: '626202',  // Default card BIN
  customCardBinRange: '',  // Default not use BIN range
  cardBindRetryTimes: 5,  // Default 5 retry times
  testModeEnabled: false,  // Default off test mode
  useLocalSuccessBins: false,  // Default off use local BIN pool
  seamlessSwitchEnabled: false,  // Default off seamless switch
  windsurfClientType: 'windsurf',  // Default Windsurf client
  windsurfPath: null,  // Windsurf path
  patchBackupPath: null,  // Patch backup path
  autoOpenBrowser: true,  // Default auto open browser
  browserMode: 'incognito',  // Default incognito mode
  privacyMode: false,  // Default off privacy mode
  unlimitedConcurrentRefresh: false,  // Default off unlimited concurrent refresh
  proxyEnabled: false,  // Default off proxy
  proxyUrl: null,  // Default no proxy URL
  useLightweightApi: true,  // Default use lightweight API
  subscriptionPlan: 2,  // Default Pro plan
  paymentPeriod: 1,  // Default monthly
  startTrial: true,  // Default start trial
  teamName: '',  // Default empty team name
  seatCount: 1,  // Default 1 seat
  cunzhiEnabled: false,  // Default off geek feature
  autoBackupEnabled: true,  // Default on auto backup
  backupInterval: 10,  // Default 10 minutes
  backupMaxCount: 10,  // Default max 10 copies
});

// Backup related
const backupLoading = ref(false);
const loadingBackups = ref(false);
const showBackupsDialog = ref(false);
const backupList = ref<Array<{ name: string; path: string; size: number }>>([]);

interface BackupInfo {
  name: string;
  path: string;
  size: number;
}

async function handleManualBackup() {
  backupLoading.value = true;
  try {
    const result = await invoke<{ success: boolean; path: string; message: string }>('create_backup');
    if (result.success) {
      ElMessage.success('Backup created successfully');
    }
  } catch (e: any) {
    ElMessage.error(`Backup failed: ${e}`);
  } finally {
    backupLoading.value = false;
  }
}

async function handleShowBackups() {
  loadingBackups.value = true;
  showBackupsDialog.value = true;
  try {
    backupList.value = await invoke<BackupInfo[]>('list_backups');
  } catch (e: any) {
    ElMessage.error(`Failed to get backup list: ${e}`);
    backupList.value = [];
  } finally {
    loadingBackups.value = false;
  }
}

async function handleRestoreBackup(backup: BackupInfo) {
  try {
    await ElMessageBox.confirm(
      `Restore from backup "${backup.name}"? Current data will be overwritten (will auto backup current data first).`,
      'Confirm Restore',
      { type: 'warning' }
    );
    
    await invoke('restore_backup', { backupPath: backup.path });
    ElMessage.success('Restore successful, please refresh the page');
    showBackupsDialog.value = false;
    await settingsStore.loadSettings();
  } catch (e: any) {
    if (e !== 'cancel') {
      ElMessage.error(`Restore failed: ${e}`);
    }
  }
}

async function handleDeleteBackup(backup: BackupInfo) {
  try {
    await ElMessageBox.confirm(
      `Delete backup "${backup.name}"? This action cannot be undone.`,
      'Confirm Delete',
      { type: 'warning' }
    );
    
    await invoke('delete_backup', { backupName: backup.name });
    ElMessage.success('Backup deleted');
    await handleShowBackups();
  } catch (e: any) {
    if (e !== 'cancel') {
      ElMessage.error(`Delete failed: ${e}`);
    }
  }
}

function formatFileSize(bytes: number): string {
  if (bytes < 1024) return bytes + ' B';
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB';
  return (bytes / (1024 * 1024)).toFixed(2) + ' MB';
}

function formatBackupTime(name: string): string {
  // Extract time from filename accounts_20260109_231500.json
  const match = name.match(/accounts_(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2})/);
  if (match) {
    return `${match[1]}-${match[2]}-${match[3]} ${match[4]}:${match[5]}:${match[6]}`;
  }
  return name;
}

// Success BIN pool related
const successBinCount = ref(0);
const testModeProgress = ref<string | null>(null);

async function loadSuccessBinCount() {
  try {
    const bins = await invoke<string[]>('get_success_bins');
    successBinCount.value = bins.length;
  } catch (e) {
    successBinCount.value = 0;
  }
}

async function loadTestModeProgress() {
  try {
    testModeProgress.value = await invoke<string | null>('get_test_mode_progress');
  } catch (e) {
    testModeProgress.value = null;
  }
}

async function resetTestModeProgress() {
  try {
    await ElMessageBox.confirm('Reset test mode progress? Next will start from range beginning.', 'Confirm Reset', {
      type: 'warning'
    });
    await invoke('reset_test_mode_progress');
    testModeProgress.value = null;
    ElMessage.success('Progress reset');
  } catch (e) {
    // User cancelled
  }
}

async function viewSuccessBins() {
  try {
    const bins = await invoke<string[]>('get_success_bins');
    if (bins.length === 0) {
      ElMessage.info('BIN pool is empty');
      return;
    }
    ElMessageBox.alert(
      `<div style="max-height: 300px; overflow-y: auto;">
        <p><b>Total ${bins.length} successful BINs:</b></p>
        <p style="font-family: monospace; word-break: break-all;">${bins.join(', ')}</p>
      </div>`,
      'Success BIN Pool',
      { dangerouslyUseHTMLString: true }
    );
  } catch (e) {
    ElMessage.error('Failed to get BIN pool');
  }
}

async function clearSuccessBins() {
  try {
    await ElMessageBox.confirm('Clear all successful card BINs?', 'Confirm Clear', {
      type: 'warning'
    });
    await invoke('clear_success_bins');
    successBinCount.value = 0;
    ElMessage.success('BIN pool cleared');
  } catch (e) {
    // User cancelled
  }
}

// Seamless switch related
const windsurfPath = ref('');
const detectingPath = ref(false);
const patchLoading = ref(false);
// Patch status (fields match backend check_patch_status return)
// - installed: all three sub-patches applied = true (summarized by backend)
// - oauthHandler / timeoutRemoved / promptBypassApplied: each sub-patch applied status
// - currentVersion: whether file contains "current version injected code" feature string, to distinguish
//   current tool vs historical/third-party tool patches (see CURRENT_VERSION_MARKER)
const patchStatus = reactive({
  installed: false,
  error: '',
  oauthHandler: false,
  timeoutRemoved: false,
  promptBypassApplied: false,
  currentVersion: false,
});

// Structured data for three sub-patches, drives UI checklist rendering
// Order matches backend apply branch (6.1 / 6.2 / 6.3), convenient for user comparison
const patchItems = computed(() => [
  { key: 'oauthHandler', label: 'OAuth Callback Handler', applied: patchStatus.oauthHandler },
  { key: 'timeoutRemoved', label: 'Remove 180s Timeout Limit', applied: patchStatus.timeoutRemoved },
  { key: 'promptBypassApplied', label: 'Skip Switch Confirm Dialog', applied: patchStatus.promptBypassApplied },
]);

// Applied sub-item count (0~3), for UI summary text
const patchAppliedCount = computed(() =>
  patchItems.value.filter(item => item.applied).length
);

// Whether to show "Upgrade Patch" button: file already has "current version injected code" but some sub-items not applied.
// Typical scenario: user had patch from old tool (1+2), new tool version adds #3, needs one-click fix.
const canUpgrade = computed(() =>
  patchStatus.currentVersion &&
  !patchStatus.installed &&
  patchAppliedCount.value > 0
);

// Summary tag: derive four presentations based on three sub-states + current_version
// - error: backend reports read/rule error
// - Not Installed: 0/3
// - Installed: 3/3 (installed=true)
// - Upgradable: current version patch but incomplete -> canUpgrade=true
// - Third-party patch: partially applied but no current version feature -> possibly historical or other tool
const patchSummary = computed<{ type: 'success' | 'info' | 'warning' | 'danger'; label: string }>(() => {
  if (patchStatus.error) {
    return { type: 'danger', label: patchStatus.error };
  }
  if (patchStatus.installed) {
    return { type: 'success', label: 'Installed' };
  }
  if (patchAppliedCount.value === 0) {
    return { type: 'info', label: 'Not Installed' };
  }
  if (canUpgrade.value) {
    return { type: 'warning', label: `Upgradable ${patchAppliedCount.value}/3` };
  }
  return { type: 'warning', label: `Third-party ${patchAppliedCount.value}/3` };
});

// Geek Feature (Cunzhi) related
const cunzhiLoading = ref(false);
const cunzhiStatus = reactive({
  installed: false,
  error: '',
});

watch(() => uiStore.showSettingsDialog, async (show) => {
  if (show && settingsStore.settings) {
    Object.assign(settings, settingsStore.settings);
    windsurfPath.value = settings.windsurfPath || '';
    // Sync seat count options to input
    if (settings.seat_count_options && settings.seat_count_options.length > 0) {
      seatCountOptionsInput.value = settings.seat_count_options.join(', ');
    }
    // Check patch status
    if (windsurfPath.value) {
      await checkPatchStatus();
    }
    // Check cunzhi status
    await checkCunzhiStatus();
    // Load success BIN pool count and test mode progress
    await loadSuccessBinCount();
    await loadTestModeProgress();
  }
});

onMounted(async () => {
  // If path exists, check status
  const storedPath = (settingsStore.settings as any)?.windsurfPath;
  if (storedPath) {
    settings.windsurfPath = storedPath;
    windsurfPath.value = storedPath;
    await checkPatchStatus();
  }
});

async function handleSave() {
  loading.value = true;
  try {
    // Ensure save path settings
    if (windsurfPath.value) {
      settings.windsurfPath = windsurfPath.value;
    }
    await settingsStore.updateSettings(settings);
    uiStore.setTheme(settings.theme as 'light' | 'dark');
    ElMessage.success('Settings saved successfully');
    handleClose();
  } catch (error) {
    ElMessage.error(`Save failed: ${error}`);
  } finally {
    loading.value = false;
  }
}

function handleClose() {
  uiStore.showSettingsDialog = false;
}

// Validate card BIN input
function validateCardBin(value: string) {
  // Only allow numbers
  const cleaned = value.replace(/[^\d]/g, '');
  settings.customCardBin = cleaned;
  
  // Check length
  if (cleaned.length > 0 && cleaned.length < 4) {
    ElMessage.warning('Card BIN must be 4-12 digits');
  }
}

// Reset default card BIN
function resetCardBin() {
  settings.customCardBin = '626202';
  ElMessage.success('Default card BIN restored');
}

// Validate card BIN range format
function validateCardBinRange(value: string) {
  // Only allow numbers and hyphens
  const cleaned = value.replace(/[^\d-]/g, '');
  settings.customCardBinRange = cleaned;
  
  // If content entered, validate format
  if (cleaned && cleaned.includes('-')) {
    const parts = cleaned.split('-');
    if (parts.length === 2) {
      const [start, end] = parts;
      // Validate both ends have same length and are numbers
      if (start && end && start.length === end.length) {
        const startNum = parseInt(start, 10);
        const endNum = parseInt(end, 10);
        if (startNum > endNum) {
          ElMessage.warning('Start BIN must be less than or equal to end BIN');
        }
      } else if (start && end && start.length !== end.length) {
        ElMessage.warning('Start and end BIN must have same length');
      }
    }
  }
}

// Clear card BIN range
function clearCardBinRange() {
  settings.customCardBinRange = '';
  ElMessage.success('BIN range cleared');
}

// Clear path and re-detect when switching client type
async function handleClientTypeChange() {
  windsurfPath.value = '';
  settings.windsurfPath = null;
  settings.seamlessSwitchEnabled = false;
  patchStatus.installed = false;
  patchStatus.error = '';
  patchStatus.oauthHandler = false;
  patchStatus.timeoutRemoved = false;
  patchStatus.promptBypassApplied = false;
  patchStatus.currentVersion = false;
  await settingsStore.updateSettings(settings);
  // Auto-detect new client path
  await detectWindsurfPath();
}

// Detect Windsurf path
async function detectWindsurfPath() {
  detectingPath.value = true;
  const clientLabel = settings.windsurfClientType === 'windsurf-next' ? 'Windsurf - Next' : 'Windsurf';
  try {
    const path = await invoke<string>('get_windsurf_path', {
      clientType: settings.windsurfClientType
    });
    windsurfPath.value = path;
    settings.windsurfPath = path;
    ElMessage.success(`Found ${clientLabel} install path`);
    // Check patch status
    await checkPatchStatus();
    // Save path settings to local
    await settingsStore.updateSettings(settings);
  } catch (error) {
    ElMessage.error(`Detection failed: ${error}`);
    windsurfPath.value = '';
  } finally {
    detectingPath.value = false;
  }
}

// Check patch status
async function checkPatchStatus() {
  if (!windsurfPath.value) return;
  
  try {
    const status = await invoke<any>('check_patch_status', {
      windsurfPath: windsurfPath.value
    });
    patchStatus.installed = status.installed;
    patchStatus.error = status.error || '';
    patchStatus.oauthHandler = !!status.oauth_handler;
    patchStatus.timeoutRemoved = !!status.timeout_removed;
    patchStatus.promptBypassApplied = !!status.prompt_bypass_applied;
    patchStatus.currentVersion = !!status.current_version;
    
    // Sync switch state with actual patch state
    if (status.installed !== settings.seamlessSwitchEnabled) {
      settings.seamlessSwitchEnabled = status.installed;
      // Save synced state
      await settingsStore.updateSettings(settings);
    }
  } catch (error) {
    patchStatus.installed = false;
    patchStatus.oauthHandler = false;
    patchStatus.timeoutRemoved = false;
    patchStatus.promptBypassApplied = false;
    patchStatus.currentVersion = false;
    patchStatus.error = error as string;
  }
}

// Handle path change
function handlePathChange() {
  if (windsurfPath.value) {
    settings.windsurfPath = windsurfPath.value;
    // Check new path patch status
    checkPatchStatus();
  }
}

// Browse select path
async function browseWindsurfPath() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      directory: true,
      multiple: false,
      title: 'Select Windsurf Install Directory'
    });
    
    if (selected && typeof selected === 'string') {
      // Validate selected path contains extension.js file
      const isValid = await invoke<boolean>('validate_windsurf_path', {
        path: selected
      });
      
      if (isValid) {
        windsurfPath.value = selected;
        settings.windsurfPath = selected;
        ElMessage.success('Selected Windsurf path');
        await checkPatchStatus();
        // Save path settings to local
        await settingsStore.updateSettings(settings);
      } else {
        ElMessage.error('Selected directory is not a valid Windsurf install directory');
      }
    }
  } catch (error) {
    ElMessage.error(`Select path failed: ${error}`);
  }
}

// Handle seamless switch toggle
async function handleSeamlessSwitch(value: boolean) {
  if (!windsurfPath.value) {
    ElMessage.error('Please detect or set client path first');
    settings.seamlessSwitchEnabled = !value;
    return;
  }
  
  // Confirm dialog
  const action = value ? 'Enable' : 'Disable';
  const clientLabel = settings.windsurfClientType === 'windsurf-next' ? 'Windsurf - Next' : 'Windsurf';
  const message = value 
    ? `Enabling seamless switch will modify ${clientLabel}'s extension.js file, auto restart if client running, continue?`
    : `Disabling seamless switch will restore original file, auto restart if client running, continue?`;
  
  try {
    await ElMessageBox.confirm(
      message,
      `${action} Seamless Switch`,
      {
        confirmButtonText: 'Confirm',
        cancelButtonText: 'Cancel',
        type: 'warning',
      }
    );
  } catch {
    // User cancelled, restore switch state
    settings.seamlessSwitchEnabled = !value;
    return;
  }
  
  patchLoading.value = true;
  try {
    let result;
    if (value) {
      // Apply patch
      result = await invoke<any>('apply_seamless_patch', {
        windsurfPath: windsurfPath.value
      });
    } else {
      // Restore patch
      result = await invoke<any>('restore_seamless_patch');
    }
    
    if (result.success) {
      ElMessage.success(result.message || `Seamless switch ${action}`);
      if (result.already_patched) {
        ElMessage.info('Patch already applied');
      }
      // Update status
      await checkPatchStatus();
      // Save settings to local
      settings.windsurfPath = windsurfPath.value;
      settings.patchBackupPath = result.backup_file || settings.patchBackupPath;
      // Save immediately to local file
      await settingsStore.updateSettings(settings);
    } else {
      ElMessage.error(result.message || `${action} failed`);
      settings.seamlessSwitchEnabled = !value;
    }
  } catch (error) {
    ElMessage.error(`${action} failed: ${error}`);
    settings.seamlessSwitchEnabled = !value;
  } finally {
    patchLoading.value = false;
  }
}

// Upgrade patch (only show when canUpgrade=true)
// Essentially run apply again: backend dry-run finds three patterns still not replaced,
// will go to apply branch for incremental replace of remaining items; already rewritten patterns skip automatically,
// so won't repeat completed parts or produce invalid backups (no-op for branches where original structure doesn't match)).
async function handleUpgradePatch() {
  if (!windsurfPath.value) return;
  patchLoading.value = true;
  try {
    const result = await invoke<any>('apply_seamless_patch', {
      windsurfPath: windsurfPath.value
    });
    if (result.success) {
      const mods: string[] = result.modifications || [];
      if (mods.length > 0) {
        ElMessage.success(`Patch upgraded: ${mods.join(', ')}`);
      } else {
        ElMessage.info(result.message || 'Patch is already latest');
      }
      await checkPatchStatus();
      settings.windsurfPath = windsurfPath.value;
      settings.patchBackupPath = result.backup_file || settings.patchBackupPath;
      await settingsStore.updateSettings(settings);
    } else {
      ElMessage.error(result.message || 'Upgrade failed');
    }
  } catch (error) {
    ElMessage.error(`Upgrade failed: ${error}`);
  } finally {
    patchLoading.value = false;
  }
}

// Reset HTTP client
async function handleResetHttpClient() {
  resettingHttp.value = true;
  try {
    const result = await systemApi.resetHttpClient();
    if (result.success) {
      ElMessage.success(result.message || 'HTTP client reset');
    } else {
      ElMessage.error('Reset failed');
    }
  } catch (error) {
    ElMessage.error(`Reset failed: ${error}`);
  } finally {
    resettingHttp.value = false;
  }
}

// Check Geek Feature status
async function checkCunzhiStatus() {
  try {
    const status = await invoke<any>('check_cunzhi_status');
    cunzhiStatus.installed = status.installed;
    cunzhiStatus.error = status.error || '';
    
    // Sync switch state with actual state
    if (status.installed !== settings.cunzhiEnabled) {
      settings.cunzhiEnabled = status.installed;
      await settingsStore.updateSettings(settings);
    }
  } catch (error) {
    cunzhiStatus.installed = false;
    cunzhiStatus.error = error as string;
  }
}

// Handle Geek Feature toggle
async function handleCunzhiSwitch(value: boolean) {
  const action = value ? 'Enable' : 'Disable';
  const message = value 
    ? 'Enable Geek Feature will install MCP server and global rules, continue?'
    : 'Disable Geek Feature will delete MCP config and global rules, continue?';
  
  try {
    await ElMessageBox.confirm(
      message,
      `${action} Geek Feature`,
      {
        confirmButtonText: 'Confirm',
        cancelButtonText: 'Cancel',
        type: 'warning',
      }
    );
  } catch {
    // User cancelled, restore switch state
    settings.cunzhiEnabled = !value;
    return;
  }
  
  cunzhiLoading.value = true;
  try {
    let result;
    if (value) {
      // Install Geek Feature
      result = await invoke<any>('install_cunzhi', { windsurfPath: settings.windsurfPath || null });
    } else {
      // Uninstall Geek Feature
      result = await invoke<any>('uninstall_cunzhi', { windsurfPath: settings.windsurfPath || null });
    }
    
    if (result.success) {
      ElMessage.success(result.message || `Geek Feature ${action}`);
      // Update status
      await checkCunzhiStatus();
      // Save settings
      await settingsStore.updateSettings(settings);
      // Prompt restart
      ElMessage.warning('Please restart Windsurf for changes to take effect');
    } else {
      ElMessage.error(result.message || `${action} failed`);
      settings.cunzhiEnabled = !value;
    }
  } catch (error) {
    ElMessage.error(`${action} failed: ${error}`);
    settings.cunzhiEnabled = !value;
  } finally {
    cunzhiLoading.value = false;
  }
}

// Function disabled in simple version
void parseSeatCountOptions;
</script>

<style scoped>
/* Dark mode styles */
:deep(.el-dialog) {
  /* Controlled by global styles in dark mode */
}

/* Description text in dark mode */
:root.dark .el-form-item > div[style*="color: #909399"] {
  color: #94a3b8 !important;
}

/* Tab content in dark mode */
:root.dark .el-tabs__content {
  background-color: transparent;
}

/* Form item labels in dark mode */
:root.dark .el-form-item__label {
  color: #cfd3dc;
}

/* Alert in dark mode */
:root.dark .el-alert--warning {
  background-color: rgba(230, 162, 60, 0.1);
  border-color: rgba(230, 162, 60, 0.3);
}

:root.dark .el-alert--warning .el-alert__description {
  color: #cfd3dc;
}

/* ==================== Patch status block ==================== */
.patch-status-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
}

.patch-status-header {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.patch-checklist {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 12px;
  background-color: rgba(0, 0, 0, 0.02);
  border: 1px solid rgba(0, 0, 0, 0.06);
  border-radius: 4px;
}

.patch-checklist-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  line-height: 1.6;
  color: #909399;
}

.patch-checklist-item.is-applied {
  color: #303133;
}

.patch-checklist-icon {
  font-size: 14px;
  color: #C0C4CC;
}

.patch-checklist-icon.is-applied {
  color: #67C23A;
}

/* Dark mode adaptation */
:root.dark .patch-checklist {
  background-color: rgba(255, 255, 255, 0.03);
  border-color: rgba(255, 255, 255, 0.08);
}

:root.dark .patch-checklist-item {
  color: #7a8394;
}

:root.dark .patch-checklist-item.is-applied {
  color: #cfd3dc;
}
</style>
