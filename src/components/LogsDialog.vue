<template>
  <el-dialog
    v-model="uiStore.showLogsDialog"
    title="Operation Logs"
    width="800px"
  >
    <div class="logs-container">
      <div class="logs-header">
        <el-button size="small" @click="loadLogs" :icon="Refresh">
          Refresh
        </el-button>
        <el-button size="small" @click="clearLogs" :icon="Delete">
          Clear Logs
        </el-button>
      </div>
      
      <el-table :data="logs" style="width: 100%" max-height="400">
        <el-table-column prop="timestamp" label="Time" width="180">
          <template #default="{ row }">
            {{ formatDate(row.timestamp) }}
          </template>
        </el-table-column>
        
        <el-table-column prop="operation_type" label="Operation Type" width="120">
          <template #default="{ row }">
            <el-tag :type="getOperationTypeTag(row.operation_type)">
              {{ formatOperationType(row.operation_type) }}
            </el-tag>
          </template>
        </el-table-column>
        
        <el-table-column prop="account_email" label="Account" width="180" />
        
        <el-table-column prop="message" label="Message" />
        
        <el-table-column prop="status" label="Status" width="80">
          <template #default="{ row }">
            <el-tag :type="row.status === 'success' ? 'success' : 'danger'">
              {{ row.status === 'success' ? 'Success' : 'Failed' }}
            </el-tag>
          </template>
        </el-table-column>
      </el-table>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Refresh, Delete } from '@element-plus/icons-vue';
import { useSettingsStore, useUIStore } from '@/store';
import dayjs from 'dayjs';

const settingsStore = useSettingsStore();
const uiStore = useUIStore();

const logs = computed(() => {
  // Sort by time, newest first
  return [...settingsStore.logs].sort((a, b) => {
    return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime();
  });
});

onMounted(() => {
  loadLogs();
});

async function loadLogs() {
  try {
    await settingsStore.loadLogs(100);
  } catch (error) {
    ElMessage.error(`Failed to load logs: ${error}`);
  }
}

async function clearLogs() {
  try {
    await ElMessageBox.confirm(
      'Are you sure you want to clear all logs?',
      'Clear Logs',
      {
        confirmButtonText: 'Confirm',
        cancelButtonText: 'Cancel',
        type: 'warning',
      }
    );
    
    await settingsStore.clearLogs();
    ElMessage.success('Logs cleared');
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error(`Failed to clear logs: ${error}`);
    }
  }
}

function formatDate(date: string) {
  return dayjs(date).format('YYYY-MM-DD HH:mm:ss');
}

function formatOperationType(type: string) {
  const typeMap: Record<string, string> = {
    login: 'Login',
    refresh_token: 'Refresh Token',
    reset_credits: 'Reset Credits',
    update_seats: 'Update Seats',
    get_billing: 'Get Billing',
    update_plan: 'Update Plan',
    add_account: 'Add Account',
    delete_account: 'Delete Account',
    edit_account: 'Edit Account',
    batch_operation: 'Batch Operation',
  };
  return typeMap[type] || type;
}

function getOperationTypeTag(type: string) {
  const tagMap: Record<string, string> = {
    login: 'primary',
    refresh_token: 'info',
    reset_credits: 'success',
    update_seats: 'warning',
    get_billing: 'info',
    update_plan: 'warning',
    add_account: 'success',
    delete_account: 'danger',
    edit_account: 'warning',
    batch_operation: 'primary',
  };
  return tagMap[type] || 'info';
}
</script>

<style scoped>
.logs-container {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.logs-header {
  display: flex;
  gap: 8px;
  padding-bottom: 12px;
  border-bottom: 1px solid #ebeef5;
}
</style>
