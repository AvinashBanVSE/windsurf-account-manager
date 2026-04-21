<template>
  <el-dialog
    v-model="visible"
    title="Seat Update Result"
    width="600px"
    @close="handleClose"
  >
    <div v-if="resultData">
      <!-- Main info card -->
      <el-card v-if="resultData.success" class="result-card" shadow="never">
        <template #header>
          <div class="card-header">
            <span>Update Successful</span>
            <el-tag type="success">
              <el-icon><Check /></el-icon>
              Seats: {{ lastAttempt?.total_seats || 'N/A' }}
            </el-tag>
          </div>
        </template>
        
        <!-- Seat info -->
        <div class="seats-section" v-if="lastAttempt">
          <el-descriptions :column="2" border>
            <el-descriptions-item label="Total Seats">
              <el-text type="success" style="font-size: 16px; font-weight: bold;">{{ lastAttempt.total_seats || 0 }}</el-text>
            </el-descriptions-item>
            <el-descriptions-item label="Billing Cycle">
              <el-text type="primary">{{ lastAttempt.billing_interval === 'yearly' ? 'Yearly' : 'Monthly' }}</el-text>
            </el-descriptions-item>
            <el-descriptions-item label="Price Per Seat">
              <el-text type="info">${{ lastAttempt.price_per_seat || 0 }}</el-text>
            </el-descriptions-item>
            <el-descriptions-item label="Total Cost">
              <el-text type="warning" style="font-size: 16px; font-weight: bold;">${{ lastAttempt.total_monthly_price || 0 }}</el-text>
            </el-descriptions-item>
          </el-descriptions>
          
          <!-- Amount due immediately (if any) -->
          <div v-if="lastAttempt.amount_due_immediately > 0" style="margin-top: 15px;">
            <el-alert
              :title="`Due now: $${lastAttempt.amount_due_immediately}`"
              type="warning"
              :closable="false"
              show-icon
            />
          </div>
        </div>
        
        <!-- Time info -->
        <div class="time-section" v-if="lastAttempt && (lastAttempt.billing_start_time || lastAttempt.next_billing_time)">
          <h4>Billing Cycle</h4>
          <el-descriptions :column="1" border>
            <el-descriptions-item label="Current Period Start" v-if="lastAttempt.billing_start_time">
              <el-text>{{ lastAttempt.billing_start_time }}</el-text>
            </el-descriptions-item>
            <el-descriptions-item label="Next Billing Time" v-if="lastAttempt.next_billing_time">
              <el-text type="warning">{{ lastAttempt.next_billing_time }}</el-text>
            </el-descriptions-item>
          </el-descriptions>
        </div>
      </el-card>
      
      <!-- Error info -->
      <el-alert
        v-if="!resultData.success"
        title="Seat Update Failed"
        :description="getErrorMessage()"
        type="error"
        :closable="false"
        show-icon
      />
      
      <!-- Attempt records -->
      <div class="attempts-section" v-if="resultData.attempts?.length > 0">
        <h4>Attempt Records ({{ resultData.attempts.length }} times)</h4>
        <el-collapse>
          <el-collapse-item 
            v-for="(attempt, index) in resultData.attempts" 
            :key="index"
            :title="`Attempt #${attempt.attempt} - Status: ${attempt.status_code || 'N/A'}`"
          >
            <el-descriptions :column="1" size="small">
              <el-descriptions-item label="Time">{{ attempt.timestamp }}</el-descriptions-item>
              <el-descriptions-item label="Status Code">{{ attempt.status_code || 'N/A' }}</el-descriptions-item>
              <el-descriptions-item label="Error" v-if="attempt.error">
                <el-text type="danger">{{ attempt.error }}</el-text>
              </el-descriptions-item>
            </el-descriptions>
            <div v-if="attempt.raw_response" style="margin-top: 10px;">
              <el-text type="info" size="small">Response Data:</el-text>
              <pre class="raw-response">{{ formatResponse(attempt.raw_response) }}</pre>
            </div>
          </el-collapse-item>
        </el-collapse>
      </div>
    </div>
    
    <template #footer>
      <el-button @click="handleClose">Close</el-button>
      <el-button type="primary" @click="copyToClipboard" v-if="resultData">
        Copy Data
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { ElMessage } from 'element-plus';
import { Check } from '@element-plus/icons-vue';

const props = defineProps<{
  modelValue: boolean;
  resultData?: any;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const visible = ref(props.modelValue);

watch(() => props.modelValue, (val) => {
  visible.value = val;
});

watch(visible, (val) => {
  emit('update:modelValue', val);
});

// Get parsed data from last attempt
const lastAttempt = computed(() => {
  if (!props.resultData?.attempts?.length) return null;
  const last = props.resultData.attempts[props.resultData.attempts.length - 1];
  if (last?.raw_response) {
    try {
      return JSON.parse(last.raw_response);
    } catch {
      return null;
    }
  }
  return null;
});


// Get error message
function getErrorMessage() {
  if (!props.resultData?.attempts?.length) return 'Unknown error';
  const lastAttempt = props.resultData.attempts[props.resultData.attempts.length - 1];
  return lastAttempt?.error || `HTTP ${lastAttempt?.status_code || 'Unknown status code'}`;
}

// Format response data
function formatResponse(response: string) {
  try {
    const parsed = JSON.parse(response);
    return JSON.stringify(parsed, null, 2);
  } catch {
    return response;
  }
}

function handleClose() {
  visible.value = false;
}

async function copyToClipboard() {
  if (props.resultData) {
    try {
      await navigator.clipboard.writeText(JSON.stringify(props.resultData, null, 2));
      ElMessage.success('Copied to clipboard');
    } catch (error) {
      ElMessage.error('Copy failed');
    }
  }
}
</script>

<style scoped>
.result-card {
  border: none;
  margin-bottom: 20px;
}

.result-card .card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 16px;
  font-weight: 600;
}

.seats-section {
  margin-bottom: 20px;
}

.time-section {
  margin-top: 20px;
}

.time-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: #606266;
}

.attempts-section {
  margin-top: 20px;
}

.attempts-section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: #606266;
}

.raw-response {
  margin-top: 5px;
  padding: 10px;
  background: #f5f7fa;
  border-radius: 4px;
  font-size: 12px;
  font-family: monospace;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 200px;
  overflow-y: auto;
}

/* Dark theme support */
:root.dark .result-card {
  background: #1e1e1e;
}

:root.dark .raw-response {
  background: #2a2a2a;
  color: #e4e4e7;
}
</style>
