<template>
  <el-dialog
    v-model="visible"
    title="Change Subscription Plan"
    width="1100px"
    class="plan-dialog"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <div class="plan-selection">
      <!-- Current Plan Info -->
      <div v-if="account?.plan_name" class="current-plan-info">
        <div class="info-left">
          <div class="info-label">Current Subscription Plan</div>
          <div class="info-value">
            <el-tag :class="['plan-tag', `plan-${account.plan_name?.toLowerCase()}`]" effect="dark">
              <el-icon><Trophy /></el-icon>
              {{ account.plan_name }}
            </el-tag>
          </div>
        </div>
        <div class="info-right" v-if="account.total_quota">
          <div class="quota-label">Quota Usage</div>
          <div class="quota-value">
            <span class="used">{{ formatQuota(account.used_quota) }}</span>
            <span class="separator">/</span>
            <span class="total">{{ formatQuota(account.total_quota) }}</span>
          </div>
          <el-progress 
            :percentage="Math.min(Math.round((account.used_quota || 0) / (account.total_quota || 1) * 100), 100)" 
            :status="getQuotaStatus((account.used_quota || 0) / (account.total_quota || 1))"
            :stroke-width="6"
            :show-text="false"
            class="quota-progress"
          />
        </div>
      </div>
      
      <div class="plans-container">
        <div 
          v-for="plan in planConfigs"
          :key="plan.key"
          class="plan-card"
          :class="{ 
            active: selectedPlan === plan.key,
            'is-current': isCurrentPlan(plan.key)
          }"
          :style="{ '--theme-color': plan.color, '--theme-bg': plan.color + '15', '--theme-shadow': plan.color + '25' }"
          @click="selectedPlan = plan.key as PlanType"
        >
          <div class="tier-badge">Tier {{ plan.tier }}</div>
          <div class="card-header">
            <div class="icon-wrapper" :style="{ background: plan.color + '20', color: plan.color }">
              <el-icon>
                <component :is="getIconComponent(plan.icon)" />
              </el-icon>
            </div>
            <h3>{{ plan.name }}</h3>
            <p class="subtitle">{{ plan.desc }}</p>
          </div>
          
          <div class="card-body">
            <div class="features-list">
              <div v-for="feature in plan.features" :key="feature" class="feature-item">
                <el-icon :style="{ color: plan.color }"><Check /></el-icon>
                <span>{{ feature }}</span>
              </div>
            </div>
          </div>

          <div class="card-footer">
            <el-button 
              class="select-btn"
              :type="selectedPlan === plan.key ? 'primary' : 'default'"
              :style="selectedPlan === plan.key ? { background: plan.color, borderColor: plan.color } : {}"
              :disabled="isCurrentPlan(plan.key)"
              round
              size="small"
            >
              {{ isCurrentPlan(plan.key) ? 'Current Plan' : (selectedPlan === plan.key ? 'Selected' : 'Select') }}
            </el-button>
          </div>
          
          <div v-if="isCurrentPlan(plan.key)" class="current-badge">In Use</div>
        </div>
      </div>
      
      <!-- Payment PeriodSelect -->
      <div class="payment-period-section">
        <div class="section-title">
          <el-icon><Calendar /></el-icon>
          <span>Payment Period</span>
        </div>
        <el-radio-group v-model="paymentPeriod" :disabled="isLooping">
          <el-radio-button :value="1">
            <el-icon><Clock /></el-icon>
            Monthly
          </el-radio-button>
          <el-radio-button :value="2">
            <el-icon><Calendar /></el-icon>
            Yearly
          </el-radio-button>
        </el-radio-group>
        <el-button
          type="info"
          plain
          size="small"
          @click="executePreview"
          :loading="loading"
          :disabled="!selectedPlan || isLooping"
          style="margin-left: 16px;"
        >
          <el-icon><View /></el-icon>
          Preview Billing
        </el-button>
      </div>

      <!-- Billing Preview Result -->
      <div v-if="billingPreview" class="billing-preview">
        <div class="preview-header">
          <el-icon><Ticket /></el-icon>
          <span>Billing Preview</span>
        </div>
        <div class="preview-content">
          <div class="preview-item" v-if="billingPreview.amount_due_immediately !== undefined">
            <span class="label">Amount Due Immediately</span>
            <span class="value">${{ billingPreview.amount_due_immediately?.toFixed(2) }}</span>
          </div>
          <div class="preview-item" v-if="billingPreview.price_per_seat !== undefined">
            <span class="label">Price Per Seat</span>
            <span class="value">${{ billingPreview.price_per_seat?.toFixed(2) }}</span>
          </div>
          <div class="preview-item" v-if="billingPreview.num_seats !== undefined">
            <span class="label">Number of Seats</span>
            <span class="value">{{ billingPreview.num_seats }}</span>
          </div>
          <div class="preview-item" v-if="billingPreview.amount_per_interval !== undefined">
            <span class="label">Cost Per Period</span>
            <span class="value">${{ billingPreview.amount_per_interval?.toFixed(2) }}/{{ billingPreview.sub_interval_name === 'yearly' ? 'year' : 'month' }}</span>
          </div>
          <div class="preview-item" v-if="billingPreview.billing_start">
            <span class="label">Billing Start</span>
            <span class="value">{{ billingPreview.billing_start }}</span>
          </div>
          <div class="preview-item" v-if="billingPreview.billing_end">
            <span class="label">Billing End</span>
            <span class="value">{{ billingPreview.billing_end }}</span>
          </div>
        </div>
      </div>

      <div v-if="error" class="error-container">
        <el-alert
          :title="error"
          type="error"
          show-icon
          :closable="false"
        />
      </div>

      <!-- Loop Update Settings -->
      <div class="loop-settings">
        <div class="loop-header">
          <div class="loop-title">
            <el-icon><Refresh /></el-icon>
            <span>Loop Update Mode</span>
          </div>
          <el-switch v-model="loopMode" :disabled="isLooping" />
        </div>
        <p class="loop-desc">After enabling, subscription updates will continue until 3 consecutive failures or manually stopped</p>
        
        <!-- Loop Execution Status -->
        <div v-if="isLooping || loopStats.totalAttempts > 0" class="loop-status">
          <div class="status-row">
            <div class="stat-item success">
              <el-icon><SuccessFilled /></el-icon>
              <span>Success: {{ loopStats.successCount }}</span>
            </div>
            <div class="stat-item failed">
              <el-icon><CircleCloseFilled /></el-icon>
              <span>Failed: {{ loopStats.failedCount }}</span>
            </div>
            <div class="stat-item total">
              <el-icon><DataLine /></el-icon>
              <span>Total: {{ loopStats.totalAttempts }}</span>
            </div>
          </div>
          <div v-if="loopStats.consecutiveFailures > 0" class="consecutive-warn">
            <el-icon><Warning /></el-icon>
            ConsecutiveFailed: {{ loopStats.consecutiveFailures }} / 3
          </div>
          <div v-if="loopStats.lastError" class="last-error">
            <el-icon><InfoFilled /></el-icon>
            {{ loopStats.lastError }}
          </div>
        </div>
      </div>

      <!-- Subscription Management Area -->
      <div class="subscription-management">
        <div class="management-header">
          <span class="title">Subscription Management</span>
          <span class="subtitle">Manage Your Subscription Status</span>
        </div>

        <div class="subscription-actions">
          <el-button
            type="danger"
            plain
            @click="handleCancelSubscription"
            :loading="cancelLoading"
            class="action-btn"
          >
            <el-icon><CircleClose /></el-icon>
            Cancel Subscription
          </el-button>

          <el-button
            type="success"
            plain
            @click="handleResumeSubscription"
            :loading="resumeLoading"
            class="action-btn"
          >
            <el-icon><CircleCheck /></el-icon>
            Resume Subscription
          </el-button>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="dialog-footer">
        <el-button @click="handleClose" :disabled="isLooping">Cancel</el-button>
        <el-button
          v-if="isLooping"
          type="danger"
          @click="stopLoop"
        >
          <el-icon><VideoPause /></el-icon>
          Stop Loop
        </el-button>
        <el-button
          v-else
          type="primary"
          @click="handleConfirm"
          :loading="loading"
          :disabled="!selectedPlan"
        >
          {{ loopMode ? 'Start Loop Update' : 'Confirm Update' }}
        </el-button>
      </div>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { 
  UserFilled, OfficeBuilding, Check, Star, Trophy, CircleClose, CircleCheck, 
  Refresh, SuccessFilled, CircleCloseFilled, DataLine, Warning, InfoFilled, 
  VideoPause, Connection, Monitor, Clock, Medal, Promotion, Briefcase, Calendar, Grid,
  View, Ticket, Present, StarFilled, Cpu
} from '@element-plus/icons-vue';
import type { Component } from 'vue';
import { apiService } from '@/api';
import type { Account } from '@/types';

const props = defineProps<{
  modelValue: boolean;
  accountId: string;
  account?: Account;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'success': [];
}>();

const visible = ref(props.modelValue);
// All available subscription plan types
type PlanType = 'free' | 'teams' | 'pro' | 'enterprise_saas' | 'hybrid' | 'enterprise_self_hosted' | 'waitlist_pro' | 'teams_ultimate' | 'pro_ultimate' | 'trial' | 'enterprise_self_serve' | 'enterprise_saas_pooled' | 'devin_enterprise' | 'devin_teams' | 'devin_teams_v2' | 'devin_pro' | 'devin_max' | 'max' | 'devin_free' | 'devin_trial' | '';
const selectedPlan = ref<PlanType>('');
// Payment Period: 1=Monthly, 2=Yearly
const paymentPeriod = ref<number>(1);

// Subscription plan configuration
const planConfigs = [
  { key: 'free', name: 'Free', tier: 0, icon: 'Present', color: '#6b7280', desc: 'Free', features: ['Basic features', 'Free usage', 'Community support'] },
  { key: 'teams', name: 'Teams', tier: 1, icon: 'UserFilled', color: '#10b981', desc: 'Teams', features: ['Team collaboration', 'Multi-user management', 'Centralized billing'] },
  { key: 'pro', name: 'Pro', tier: 2, icon: 'Star', color: '#3b82f6', desc: 'Pro', features: ['Personal Pro', 'Advanced features', 'Priority support'] },
  { key: 'enterprise_saas', name: 'Enterprise SaaS', tier: 3, icon: 'OfficeBuilding', color: '#8b5cf6', desc: 'Enterprise SaaS', features: ['Enterprise-grade security', 'SaaS deployment', 'API access'] },
  { key: 'hybrid', name: 'Hybrid', tier: 4, icon: 'Connection', color: '#f59e0b', desc: 'Hybrid', features: ['Hybrid cloud deployment', 'Flexible configuration', 'Data isolation'] },
  { key: 'enterprise_self_hosted', name: 'Enterprise Self-Hosted', tier: 5, icon: 'Monitor', color: '#ec4899', desc: 'Enterprise Self-Hosted', features: ['Local deployment', 'Full control', 'Data autonomy'] },
  { key: 'waitlist_pro', name: 'Waitlist Pro', tier: 6, icon: 'Clock', color: '#6366f1', desc: 'Waitlist Pro', features: ['Priority access', 'Priority experience', 'Special offer'] },
  { key: 'teams_ultimate', name: 'Teams Ultimate', tier: 7, icon: 'Trophy', color: '#14b8a6', desc: 'Teams Ultimate', features: ['All team features', 'Unlimited quota', 'VIP Support'] },
  { key: 'pro_ultimate', name: 'Pro Ultimate', tier: 8, icon: 'Medal', color: '#f97316', desc: 'Pro Ultimate', features: ['All Pro features', 'Unlimited quota', 'VIP Support'] },
  { key: 'trial', name: 'Trial', tier: 9, icon: 'Promotion', color: '#84cc16', desc: 'Trial', features: ['Limited-time experience', 'All features', 'No payment required'] },
  { key: 'enterprise_self_serve', name: 'Enterprise Self-Serve', tier: 10, icon: 'Briefcase', color: '#a855f7', desc: 'Enterprise Self-Serve', features: ['Enterprise-grade features', 'Self-service management', 'SLA guarantee'] },
  { key: 'enterprise_saas_pooled', name: 'Enterprise SaaS Pooled', tier: 11, icon: 'Grid', color: '#0891b2', desc: 'Enterprise SaaS Pooled', features: ['Shared resource pool', 'Elastic scaling', 'Cost optimization'] },
  { key: 'devin_enterprise', name: 'Devin Enterprise', tier: 12, icon: 'Cpu', color: '#dc2626', desc: 'Devin Enterprise', features: ['AI agent', 'Enterprise-grade', 'Team management'] },
  { key: 'devin_teams', name: 'Devin Teams', tier: 14, icon: 'Cpu', color: '#e11d48', desc: 'Devin Teams', features: ['AI agent', 'Team collaboration', 'Multiple users'] },
  { key: 'devin_teams_v2', name: 'Devin Teams V2', tier: 15, icon: 'Cpu', color: '#be123c', desc: 'Devin Teams V2', features: ['AI agent', 'Team V2', 'Enhanced features'] },
  { key: 'devin_pro', name: 'Devin Pro', tier: 16, icon: 'Cpu', color: '#ea580c', desc: 'Devin Pro', features: ['AI agent', 'Professional features', 'Personal use'] },
  { key: 'devin_max', name: 'Devin Max', tier: 17, icon: 'Cpu', color: '#c2410c', desc: 'Devin Max', features: ['AI agent', 'Unlimited features', 'Highest configuration'] },
  { key: 'max', name: 'Max', tier: 18, icon: 'StarFilled', color: '#7c3aed', desc: 'Max', features: ['Highest configuration', 'Unlimited quota', 'All features unlocked'] },
  { key: 'devin_free', name: 'Devin Free', tier: 19, icon: 'Cpu', color: '#9ca3af', desc: 'Devin Free', features: ['AI agent', 'Basic features', 'Free usage'] },
  { key: 'devin_trial', name: 'Devin Trial', tier: 20, icon: 'Cpu', color: '#f472b6', desc: 'Devin Trial', features: ['AI agent', 'Limited-time experience', 'All features'] },
];
const loading = ref(false);
const cancelLoading = ref(false);
const resumeLoading = ref(false);
const error = ref('');

// Icon component mapping
const iconMap: Record<string, Component> = {
  UserFilled, OfficeBuilding, Star, StarFilled, Trophy, Connection, Monitor, Clock, Medal, Promotion, Briefcase, Check, Grid, Present, Cpu
};

// Get icon component
function getIconComponent(iconName: string): Component {
  return iconMap[iconName] || Star;
}

// Check ifCurrent Plan
function isCurrentPlan(planKey: string): boolean {
  const currentPlan = props.account?.plan_name?.toLowerCase();
  if (!currentPlan) return false;
  // Handle some special mappings
  if (planKey === 'enterprise' && currentPlan.includes('enterprise')) return true;
  return currentPlan === planKey || currentPlan.replace(/[_-]/g, '') === planKey.replace(/[_-]/g, '');
}

// Loop update related state
const loopMode = ref(false);
const isLooping = ref(false);
const shouldStopLoop = ref(false);
const loopStats = reactive({
  successCount: 0,
  failedCount: 0,
  totalAttempts: 0,
  consecutiveFailures: 0,
  lastError: ''
});

// Preview mode
const previewMode = ref(false);
const billingPreview = ref<{
  amount_due_immediately?: number;
  price_per_seat?: number;
  num_seats?: number;
  sub_interval_name?: string;
  amount_per_interval?: number;
  billing_start?: string;
  billing_end?: string;
} | null>(null);

// Cancellation reason options
const cancelReasons = [
  { value: 'too_expensive', label: 'Too expensive' },
  { value: 'not_using', label: 'No longer using' },
  { value: 'missing_features', label: 'Missing features' },
  { value: 'switching_service', label: 'Switching to another service' },
  { value: 'other', label: 'Other reasons' }
];

watch(() => props.modelValue, (val) => {
  visible.value = val;
  if (val) {
    selectedPlan.value = '';
    error.value = '';
    loopMode.value = false;
    isLooping.value = false;
    shouldStopLoop.value = false;
    previewMode.value = false;
    billingPreview.value = null;
    resetLoopStats();
  }
});

watch(visible, (val) => {
  emit('update:modelValue', val);
});

function getQuotaStatus(percentage: number) {
  if (percentage >= 0.9) return 'exception';
  if (percentage >= 0.75) return 'warning';
  return 'success';
}

// Reset loop statistics
function resetLoopStats() {
  loopStats.successCount = 0;
  loopStats.failedCount = 0;
  loopStats.totalAttempts = 0;
  loopStats.consecutiveFailures = 0;
  loopStats.lastError = '';
}

// Stop Loop
function stopLoop() {
  shouldStopLoop.value = true;
  ElMessage.info('Stopping loop...');
}

// Execute single update
// Returns: success=whether successful, hasReason=whether there is a clear reason on failure
async function executeSingleUpdate(): Promise<{ success: boolean; hasReason: boolean }> {
  try {
    const result = await apiService.updatePlan(props.accountId, selectedPlan.value, paymentPeriod.value, false);
    if (result.success) {
      return { success: true, hasReason: false };
    } else {
      const reason = result.payment_failure_reason;
      loopStats.lastError = reason || 'Failed to update plan';
      // Having a clear reason indicates payment issue（e.g., wrong card number），not countedConsecutive Failures
      return { success: false, hasReason: !!reason };
    }
  } catch (err: any) {
    loopStats.lastError = err.toString();
    return { success: false, hasReason: true }; // Exceptions also count as having a reason
  }
}

// Execute preview
async function executePreview(): Promise<void> {
  if (!selectedPlan.value) {
    ElMessage.warning('PleasefirstSelectSubscriptionPlan');
    return;
  }
  
  loading.value = true;
  error.value = '';
  billingPreview.value = null;
  
  try {
    const result = await apiService.updatePlan(props.accountId, selectedPlan.value, paymentPeriod.value, true);
    if (result.success && result.billing_update) {
      billingPreview.value = result.billing_update;
      ElMessage.success('Preview successful, please check billing details');
    } else if (result.payment_failure_reason) {
      error.value = `PaymentFailed: ${result.payment_failure_reason}`;
      ElMessage.error(error.value);
    } else {
      ElMessage.info('Preview complete, no billing changes');
    }
  } catch (err: any) {
    error.value = err.toString();
    ElMessage.error(`PreviewFailed: ${err}`);
  } finally {
    loading.value = false;
  }
}

// Delay function
function delay(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function handleConfirm() {
  if (!selectedPlan.value) {
    ElMessage.warning('Please select a subscription plan');
    return;
  }

  // If loop mode is enabled
  if (loopMode.value) {
    isLooping.value = true;
    shouldStopLoop.value = false;
    resetLoopStats();
    error.value = '';

    while (!shouldStopLoop.value && loopStats.consecutiveFailures < 3) {
      loopStats.totalAttempts++;
      
      const result = await executeSingleUpdate();
      
      if (result.success) {
        loopStats.successCount++;
        loopStats.consecutiveFailures = 0; // Reset consecutive failure count
        loopStats.lastError = '';
      } else {
        loopStats.failedCount++;
        // Only failures without clear reason are counted toward consecutive failures times
        // Failures with clear reason (e.g., wrong card) do not count
        if (!result.hasReason) {
          loopStats.consecutiveFailures++;
        } else {
          // Failures with reason reset the counter
          loopStats.consecutiveFailures = 0;
        }
      }

      // Check if should stop
      if (shouldStopLoop.value) {
        ElMessage.warning('Loop has been manually stopped');
        break;
      }

      if (loopStats.consecutiveFailures >= 3) {
        ElMessage.error('3 consecutive failures without reason, loop stopped');
        error.value = `3 consecutive failures without reason:: ${loopStats.lastError}`;
        break;
      }

      // Brief delay to avoid requests too fast
      await delay(500);
    }

    isLooping.value = false;
    
    // Show final statistics
    if (loopStats.successCount > 0) {
      ElMessage.success(`Loop ended: Success ${loopStats.successCount}  times, Failed ${loopStats.failedCount} times`);
      emit('success');
    }
  } else {
    // Single execution mode
    loading.value = true;
    error.value = '';
    const periodName = paymentPeriod.value === 2 ? 'Yearly' : 'Monthly';

    try {
      const result = await apiService.updatePlan(props.accountId, selectedPlan.value, paymentPeriod.value, false);
      if (result.success) {
        ElMessage.success(`Successfully changed to ${selectedPlan.value.toUpperCase()} Plan（${periodName}）`);
        emit('success');
        handleClose();
      } else {
        error.value = result.payment_failure_reason || 'Failed to update plan';
        ElMessage.error(error.value);
      }
    } catch (err: any) {
      error.value = err.toString();
      ElMessage.error(`UpdatePlanFailed: ${err}`);
    } finally {
      loading.value = false;
    }
  }
}

// Cancel Subscription
async function handleCancelSubscription() {
  try {
    // Step 1: Select cancellation reason
    let selectedReason = 'too_expensive'; // Default value

    const reasonHtml = `
      <div style="text-align: left; padding: 10px 0;">
        <p style="margin-bottom: 12px; color: #606266;">Please select cancellation reason：</p>
        <el-radio-group id="cancel-reason-group" style="display: flex; flex-direction: column; gap: 8px;">
          ${cancelReasons.map(r => `
            <label style="display: flex; align-items: center; padding: 8px; cursor: pointer; border-radius: 4px; transition: background 0.2s;"
                   onmouseover="this.style.background='#f5f7fa'"
                   onmouseout="this.style.background='transparent'">
              <input type="radio" name="cancel-reason" value="${r.value}" ${r.value === 'too_expensive' ? 'checked' : ''}
                     style="margin-right: 8px;"
                     onchange="window.__selectedCancelReason='${r.value}'">
              <span style="color: #303133;">${r.label}</span>
            </label>
          `).join('')}
        </el-radio-group>
      </div>
    `;

    // Initialize global variable
    (window as any).__selectedCancelReason = 'too_expensive';

    await ElMessageBox.confirm(reasonHtml, 'Cancel Subscription Confirmation', {
      confirmButtonText: 'Confirm cancellation',
      cancelButtonText: 'Back',
      type: 'warning',
      dangerouslyUseHTMLString: true,
      beforeClose: async (action, instance, done) => {
        if (action === 'confirm') {
          selectedReason = (window as any).__selectedCancelReason || 'too_expensive';

          instance.confirmButtonLoading = true;
          instance.confirmButtonText = 'Cancelling...';

          try {
            console.log('Cancel Subscription，reason:', selectedReason);
            const result = await apiService.cancelSubscription(props.accountId, selectedReason);

            if (result.success) {
              ElMessage.success('Subscription successfully cancelled');
              emit('success');
              done();
              handleClose();
            } else {
              ElMessage.error(result.raw_response || 'Failed to cancel subscription');
              instance.confirmButtonLoading = false;
              instance.confirmButtonText = 'Confirm cancellation';
            }
          } catch (err: any) {
            ElMessage.error(`Failed to cancel subscription: ${err}`);
            instance.confirmButtonLoading = false;
            instance.confirmButtonText = 'Confirm cancellation';
          } finally {
            // Cleanup global variable
            delete (window as any).__selectedCancelReason;
          }
        } else {
          // Cleanup global variable
          delete (window as any).__selectedCancelReason;
          done();
        }
      }
    });
  } catch (err) {
    // User cancelled operation
    console.log('UserCancelCancel Subscriptionoperation');
    // Cleanup global variable
    delete (window as any).__selectedCancelReason;
  }
}

// Resume Subscription
async function handleResumeSubscription() {
  try {
    await ElMessageBox.confirm(
      'Confirm to resume subscription? After resuming, will continue with original plan billing。',
      'Resume SubscriptionConfirm',
      {
        confirmButtonText: 'Confirm resume',
        cancelButtonText: 'Cancel',
        type: 'warning'
      }
    );

    resumeLoading.value = true;

    try {
      const result = await apiService.resumeSubscription(props.accountId);

      if (result.success) {
        ElMessage.success('Subscription successfully resumed');
        emit('success');
        handleClose();
      } else {
        ElMessage.error(result.raw_response || 'Failed to resume subscription');
      }
    } catch (err: any) {
      ElMessage.error(`Failed to resume subscription: ${err}`);
    } finally {
      resumeLoading.value = false;
    }
  } catch (err) {
    // User cancelled operation
    console.log('User cancelled resume subscription operation');
  }
}

// Format quota (divide by 100 and display two decimals)
function formatQuota(num: number | undefined | null) {
  if (!num) return '0.00';
  return (num / 100).toFixed(2);
}

function handleClose() {
  visible.value = false;
}
</script>

<style scoped lang="scss">
.plan-selection {
  padding: 10px;
}

/* Current PlanInformation */
.current-plan-info {
  background: white;
  border: 1px solid #e4e7ed;
  border-radius: 12px;
  padding: 20px 24px;
  margin-bottom: 30px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.05);

  .info-left {
    .info-label {
      font-size: 13px;
      color: #909399;
      margin-bottom: 8px;
    }
    
    .plan-tag {
      padding: 6px 16px;
      height: 36px;
      font-size: 14px;
      font-weight: 600;
      border: none;
      
      :deep(.el-icon) {
        margin-right: 6px;
        font-size: 16px;
      }

      &.plan-pro { background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%); }
      &.plan-teams { background: linear-gradient(135deg, #10b981 0%, #059669 100%); }
      &.plan-enterprise { background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%); }
    }
  }

  .info-right {
    text-align: right;
    min-width: 200px;

    .quota-label {
      font-size: 13px;
      color: #909399;
      margin-bottom: 4px;
    }

    .quota-value {
      font-family: 'Roboto Mono', monospace;
      margin-bottom: 6px;
      
      .used { color: #303133; font-weight: 600; font-size: 18px; }
      .separator { margin: 0 4px; color: #c0c4cc; }
      .total { color: #909399; }
    }

    .quota-progress {
      width: 100%;
    }
  }
}

/* Plan Card Container - Fit More Cards */
.plans-container {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 12px;
  margin-bottom: 20px;
  max-height: 420px;
  overflow-y: auto;
  padding: 4px;
}

/* Plan Card Basic Style - Compact */
.plan-card {
  position: relative;
  background: #fff;
  border: 2px solid #e4e7ed;
  border-radius: 12px;
  padding: 12px;
  cursor: pointer;
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 200px;

  &:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 20px rgba(0, 0, 0, 0.08);
  }

  &.active {
    border-color: var(--theme-color);
    background: var(--theme-bg);
    box-shadow: 0 4px 16px var(--theme-shadow);
  }

  &.is-current {
    border-color: #ffd700;
    border-width: 2px;
  }

  .tier-badge {
    position: absolute;
    top: 8px;
    right: 8px;
    background: rgba(0, 0, 0, 0.05);
    color: #909399;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 10px;
    font-weight: 600;
  }

  .current-badge {
    position: absolute;
    top: -10px;
    left: 50%;
    transform: translateX(-50%);
    background: linear-gradient(135deg, #ffd700 0%, #ffa500 100%);
    color: #fff;
    padding: 2px 8px;
    border-radius: 10px;
    font-size: 10px;
    font-weight: 600;
    box-shadow: 0 2px 8px rgba(255, 215, 0, 0.3);
    z-index: 1;
    white-space: nowrap;
  }
}

/* Card Header - Compact */
.card-header {
  text-align: center;
  margin-bottom: 10px;

  .icon-wrapper {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 8px;
    font-size: 18px;
    transition: transform 0.2s ease;
  }

  h3 {
    font-size: 13px;
    font-weight: 700;
    margin: 0 0 4px;
    color: #303133;
    line-height: 1.2;
  }

  .subtitle {
    font-size: 11px;
    color: #909399;
    margin: 0;
  }
}

.plan-card:hover .icon-wrapper {
  transform: scale(1.1);
}

/* Card Body - Compact */
.card-body {
  flex: 1;
  margin-bottom: 10px;
}

.features-list {
  display: flex;
  flex-direction: column;
  gap: 4px;

  .feature-item {
    display: flex;
    align-items: center;
    font-size: 11px;
    color: #606266;
    
    .el-icon {
      margin-right: 4px;
      font-size: 12px;
    }
  }
}

/* Card Footer - Compact */
.card-footer {
  text-align: center;
  
  .select-btn {
    width: 100%;
    height: 28px;
    font-size: 12px;
    font-weight: 600;
    transition: all 0.2s ease;

    &:not(.is-disabled):hover {
      transform: translateY(-1px);
      box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
    }
  }
}

/* Dynamic theme already applied via inline styles */

/* Payment PeriodSelect */
.payment-period-section {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: linear-gradient(135deg, #fef3c7 0%, #fde68a 100%);
  border: 1px solid #fcd34d;
  border-radius: 10px;
  margin-bottom: 16px;

  .section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 600;
    color: #92400e;

    .el-icon {
      font-size: 18px;
      color: #d97706;
    }
  }

  :deep(.el-radio-group) {
    .el-radio-button__inner {
      display: flex;
      align-items: center;
      gap: 4px;
      padding: 8px 16px;
    }
  }
}

/* Loop Update Settings */
.loop-settings {
  background: linear-gradient(135deg, #f0f9ff 0%, #e0f2fe 100%);
  border: 1px solid #bae6fd;
  border-radius: 12px;
  padding: 16px 20px;
  margin-bottom: 20px;

  .loop-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;

    .loop-title {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 15px;
      font-weight: 600;
      color: #0369a1;

      .el-icon {
        font-size: 18px;
      }
    }
  }

  .loop-desc {
    font-size: 13px;
    color: #64748b;
    margin: 0 0 12px;
  }

  .loop-status {
    background: rgba(255, 255, 255, 0.8);
    border-radius: 8px;
    padding: 12px 16px;

    .status-row {
      display: flex;
      gap: 20px;
      margin-bottom: 8px;

      .stat-item {
        display: flex;
        align-items: center;
        gap: 6px;
        font-size: 14px;
        font-weight: 500;

        &.success {
          color: #16a34a;
          .el-icon { color: #22c55e; }
        }

        &.failed {
          color: #dc2626;
          .el-icon { color: #ef4444; }
        }

        &.total {
          color: #0369a1;
          .el-icon { color: #0ea5e9; }
        }
      }
    }

    .consecutive-warn {
      display: flex;
      align-items: center;
      gap: 6px;
      font-size: 13px;
      color: #d97706;
      padding: 6px 10px;
      background: #fef3c7;
      border-radius: 6px;
      margin-bottom: 8px;

      .el-icon {
        color: #f59e0b;
      }
    }

    .last-error {
      display: flex;
      align-items: flex-start;
      gap: 6px;
      font-size: 12px;
      color: #64748b;
      padding: 6px 10px;
      background: #f1f5f9;
      border-radius: 6px;
      word-break: break-all;

      .el-icon {
        flex-shrink: 0;
        margin-top: 2px;
      }
    }
  }
}

/* Billing Preview */
.billing-preview {
  background: linear-gradient(135deg, #e8f5e9 0%, #c8e6c9 100%);
  border: 1px solid #81c784;
  border-radius: 12px;
  padding: 16px 20px;
  margin-bottom: 16px;

  .preview-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 15px;
    font-weight: 600;
    color: #2e7d32;
    margin-bottom: 12px;

    .el-icon {
      font-size: 18px;
    }
  }

  .preview-content {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;

    .preview-item {
      background: rgba(255, 255, 255, 0.8);
      border-radius: 8px;
      padding: 10px 12px;
      display: flex;
      flex-direction: column;
      gap: 4px;

      .label {
        font-size: 12px;
        color: #666;
      }

      .value {
        font-size: 14px;
        font-weight: 600;
        color: #2e7d32;
      }
    }
  }
}

@media (max-width: 768px) {
  .billing-preview .preview-content {
    grid-template-columns: repeat(2, 1fr);
  }
}

/* Error Message */
.error-container {
  margin-bottom: 30px;
}

/* Subscription Management */
.subscription-management {
  background: linear-gradient(to right, #fdf6ec, #fff);
  border-left: 4px solid #e6a23c;
  border-radius: 4px;
  padding: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  
  .management-header {
    display: flex;
    flex-direction: column;
    
    .title {
      font-size: 16px;
      font-weight: 600;
      color: #e6a23c;
      margin-bottom: 4px;
    }
    
    .subtitle {
      font-size: 13px;
      color: #909399;
    }
  }
  
  .subscription-actions {
    display: flex;
    gap: 12px;
    
    .action-btn {
      padding: 8px 20px;
    }
  }
}

/* Responsive Adaptation */
@media (max-width: 1200px) {
  .plans-container {
    grid-template-columns: repeat(4, 1fr);
  }
}

@media (max-width: 900px) {
  .plans-container {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (max-width: 768px) {
  .plans-container {
    grid-template-columns: repeat(2, 1fr);
  }
  
  .current-plan-info {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
    
    .info-right {
      width: 100%;
      text-align: left;
    }
  }
  
  .subscription-management {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
    
    .subscription-actions {
      width: 100%;
      justify-content: flex-start;
    }
  }
}

@media (max-width: 500px) {
  .plans-container {
    grid-template-columns: 1fr;
  }
}
</style>
