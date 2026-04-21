<template>
  <el-dialog
    v-model="dialogVisible"
    title="Human Verification"
    width="400px"
    :close-on-click-modal="false"
    :close-on-press-escape="false"
    @close="handleClose"
  >
    <div class="turnstile-container">
      <p class="turnstile-tip">Please complete verification to get trial link</p>
      
      <div class="turnstile-wrapper">
        <div 
          ref="turnstileRef" 
          class="cf-turnstile"
        ></div>
      </div>
      
      <p v-if="status === 'loading'" class="status-text loading">
        <el-icon class="is-loading"><Loading /></el-icon>
        Loading verification...
      </p>
      <p v-else-if="status === 'success'" class="status-text success">
        <el-icon><CircleCheck /></el-icon>
        Verification successful!
      </p>
      <p v-else-if="status === 'error'" class="status-text error">
        <el-icon><CircleClose /></el-icon>
        Verification failed, please try again
      </p>
    </div>
    
    <template #footer>
      <el-button @click="handleClose">Cancel</el-button>
      <el-button 
        type="primary" 
        :disabled="status !== 'success'"
        :loading="isSubmitting"
        @click="handleConfirm"
      >
        Get Link
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch, onUnmounted, nextTick } from 'vue';
import { Loading, CircleCheck, CircleClose } from '@element-plus/icons-vue';

const TURNSTILE_SITE_KEY = '0x4AAAAAAA447Bur1xJStKg5';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'success', token: string): void;
  (e: 'cancel'): void;
}>();

const dialogVisible = ref(false);
const turnstileRef = ref<HTMLElement | null>(null);
const turnstileToken = ref('');
const status = ref<'idle' | 'loading' | 'success' | 'error'>('loading');
const isSubmitting = ref(false);
const widgetId = ref<string | null>(null);

// Sync visible
watch(() => props.visible, (val) => {
  dialogVisible.value = val;
  if (val) {
    status.value = 'loading';
    turnstileToken.value = '';
    nextTick(() => {
      loadTurnstile();
    });
  }
});

watch(dialogVisible, (val) => {
  emit('update:visible', val);
});

// Load Turnstile script
function loadTurnstileScript(): Promise<void> {
  return new Promise((resolve, reject) => {
    if ((window as any).turnstile) {
      resolve();
      return;
    }
    
    const script = document.createElement('script');
    script.src = 'https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit';
    script.async = true;
    script.defer = true;
    
    script.onload = () => {
      console.log('[Turnstile] Script loaded');
      resolve();
    };
    
    script.onerror = () => {
      console.error('[Turnstile] Failed to load script');
      reject(new Error('Failed to load Turnstile script'));
    };
    
    document.head.appendChild(script);
  });
}

// Render Turnstile component
async function loadTurnstile() {
  try {
    await loadTurnstileScript();
    
    // Wait for turnstile object to be available
    await new Promise<void>((resolve) => {
      const checkTurnstile = () => {
        if ((window as any).turnstile) {
          resolve();
        } else {
          setTimeout(checkTurnstile, 100);
        }
      };
      checkTurnstile();
    });
    
    const turnstile = (window as any).turnstile;
    
    // If existing widget, remove first
    if (widgetId.value) {
      try {
        turnstile.remove(widgetId.value);
      } catch (e) {
        console.log('[Turnstile] Remove old widget failed:', e);
      }
    }
    
    // Clear container
    if (turnstileRef.value) {
      turnstileRef.value.innerHTML = '';
    }
    
    // Render new widget
    await nextTick();
    
    if (turnstileRef.value) {
      widgetId.value = turnstile.render(turnstileRef.value, {
        sitekey: TURNSTILE_SITE_KEY,
        theme: 'light',
        callback: (token: string) => {
          console.log('[Turnstile] Verification success');
          turnstileToken.value = token;
          status.value = 'success';
        },
        'error-callback': () => {
          console.error('[Turnstile] Verification failed');
          status.value = 'error';
        },
        'expired-callback': () => {
          console.log('[Turnstile] Token expired');
          status.value = 'idle';
          turnstileToken.value = '';
        }
      });
      
      status.value = 'idle';
    }
  } catch (error) {
    console.error('[Turnstile] Load error:', error);
    status.value = 'error';
  }
}

function handleConfirm() {
  if (turnstileToken.value) {
    isSubmitting.value = true;
    emit('success', turnstileToken.value);
  }
}

function handleClose() {
  dialogVisible.value = false;
  isSubmitting.value = false;
  emit('cancel');
}

// Cleanup
onUnmounted(() => {
  if (widgetId.value && (window as any).turnstile) {
    try {
      (window as any).turnstile.remove(widgetId.value);
    } catch (e) {
      console.log('[Turnstile] Cleanup failed:', e);
    }
  }
});
</script>

<style scoped>
.turnstile-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px 0;
}

.turnstile-tip {
  margin-bottom: 20px;
  color: #606266;
  font-size: 14px;
}

.turnstile-wrapper {
  min-height: 65px;
  display: flex;
  justify-content: center;
  align-items: center;
}

.status-text {
  margin-top: 15px;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
}

.status-text.loading {
  color: #909399;
}

.status-text.success {
  color: #67c23a;
}

.status-text.error {
  color: #f56c6c;
}

.is-loading {
  animation: rotating 1s linear infinite;
}

@keyframes rotating {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}
</style>
