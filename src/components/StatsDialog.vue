<template>
  <el-dialog
    v-model="uiStore.showStatsDialog"
    title="Statistics"
    width="700px"
  >
    <div v-if="loading" class="loading-container">
      <el-icon class="is-loading" size="32"><Loading /></el-icon>
    </div>
    
    <div v-else-if="stats">
      <el-row :gutter="20">
        <el-col :span="8">
          <el-statistic title="Total Accounts" :value="stats.total_accounts" />
        </el-col>
        <el-col :span="8">
          <el-statistic title="Active Accounts" :value="stats.active_accounts" />
        </el-col>
        <el-col :span="8">
          <el-statistic title="Group Count" :value="stats.groups" />
        </el-col>
      </el-row>
      
      <el-divider />
      
      <el-row :gutter="20">
        <el-col :span="12">
          <el-statistic 
            title="Operation Success Rate" 
            :value="stats.success_rate" 
            suffix="%" 
            :precision="1"
          />
          <div class="stat-detail">
            Success: {{ stats.successful_operations }} / Failed: {{ stats.failed_operations }}
          </div>
        </el-col>
        <el-col :span="12">
          <el-statistic 
            title="Credit Reset Success Rate" 
            :value="stats.reset_success_rate" 
            suffix="%" 
            :precision="1"
          />
          <div class="stat-detail">
            Success: {{ stats.successful_resets }} / Failed: {{ stats.failed_resets }}
          </div>
        </el-col>
      </el-row>
      
      <el-divider />
      
      <el-descriptions :column="1" border>
        <el-descriptions-item label="Total Operations">
          {{ stats.total_operations }}
        </el-descriptions-item>
        
        <el-descriptions-item label="Total Resets">
          {{ stats.total_resets }}
        </el-descriptions-item>
        
        <el-descriptions-item label="Last Operation Time">
          {{ stats.last_operation ? formatDate(stats.last_operation) : 'None' }}
        </el-descriptions-item>
        
        <el-descriptions-item label="Auto Refresh Token">
          <el-tag :type="stats.settings?.auto_refresh_token ? 'success' : 'info'">
            {{ stats.settings?.auto_refresh_token ? 'On' : 'Off' }}
          </el-tag>
        </el-descriptions-item>
        
        <el-descriptions-item label="Retry Times">
          {{ stats.settings?.retry_times || 2 }}
        </el-descriptions-item>
        
        <el-descriptions-item label="Concurrent Limit">
          {{ stats.settings?.concurrent_limit || 5 }}
        </el-descriptions-item>
      </el-descriptions>
    </div>
    
    <template #footer>
      <el-button @click="refresh" :icon="Refresh">Refresh</el-button>
      <el-button @click="uiStore.closeStatsDialog">Close</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { Loading, Refresh } from '@element-plus/icons-vue';
import { useUIStore } from '@/store';
import { settingsApi } from '@/api';
import dayjs from 'dayjs';

const uiStore = useUIStore();

const loading = ref(false);
const stats = ref<any>(null);

watch(() => uiStore.showStatsDialog, (show) => {
  if (show) {
    loadStats();
  }
});

onMounted(() => {
  if (uiStore.showStatsDialog) {
    loadStats();
  }
});

async function loadStats() {
  loading.value = true;
  try {
    stats.value = await settingsApi.getStats();
  } catch (error) {
    ElMessage.error(`Failed to load statistics: ${error}`);
  } finally {
    loading.value = false;
  }
}

function refresh() {
  loadStats();
}

function formatDate(date: string) {
  return dayjs(date).format('YYYY-MM-DD HH:mm:ss');
}
</script>

<style scoped>
.loading-container {
  display: flex;
  justify-content: center;
  align-items: center;
  padding: 40px;
}

.stat-detail {
  margin-top: 8px;
  font-size: 12px;
  color: #909399;
  text-align: center;
}

.el-statistic {
  text-align: center;
}

.el-row {
  margin-bottom: 20px;
}
</style>
