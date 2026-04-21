<template>
  <el-dialog
    v-model="visible"
    title="Tag Management"
    width="600px"
    :close-on-click-modal="false"
    @close="handleClose"
  >
    <div class="tag-manage-container">
      <!-- Add new tag -->
      <div class="add-tag-section">
        <el-input
          v-model="newTagName"
          placeholder="Enter new tag name"
          size="default"
          class="tag-input"
          @keyup.enter="handleAddTag"
        >
          <template #prepend>
            <el-color-picker
              v-model="newTagColor"
              show-alpha
              :predefine="predefineColors"
              size="default"
            />
          </template>
          <template #append>
            <el-button :icon="Plus" @click="handleAddTag" :loading="adding">
Add
            </el-button>
          </template>
        </el-input>
      </div>

      <!-- Tag list -->
      <div class="tags-section">
        <div class="section-header">
          <span class="section-title">Global Tags ({{ settingsStore.tags.length }})</span>
        </div>
        
        <div v-if="settingsStore.tags.length === 0" class="empty-hint">
          No tags available, please add a new tag
        </div>

        <el-scrollbar max-height="300px">
          <div class="tag-list">
            <div 
              v-for="tag in settingsStore.tags" 
              :key="tag.name" 
              class="tag-item"
            >
              <div class="tag-info">
                <span 
                  class="tag-preview"
                  :style="getTagStyle(tag.color)"
                >
                  {{ tag.name }}
                </span>
                <span class="tag-usage">
                  Usage: {{ getTagUsageCount(tag.name) }} accounts
                </span>
              </div>
              <div class="tag-actions">
                <el-color-picker
                  :model-value="tag.color"
                  show-alpha
                  :predefine="predefineColors"
                  size="small"
                  @change="(color: string | null) => handleUpdateColor(tag.name, color)"
                />
                <el-button
                  :icon="Edit"
                  circle
                  size="small"
                  @click="openEditDialog(tag)"
                />
                <el-button
                  :icon="Delete"
                  circle
                  size="small"
                  type="danger"
                  @click="handleDeleteTag(tag.name)"
                />
              </div>
            </div>
          </div>
        </el-scrollbar>
      </div>

      <!-- Batch operations area -->
      <div class="batch-section" v-if="selectedAccountIds.length > 0">
        <div class="section-header">
          <span class="section-title">Batch Operations ({{ selectedAccountIds.length }} accounts selected)</span>
        </div>
        
        <div class="batch-content">
          <div class="batch-row">
            <span class="batch-label">Add tags:</span>
            <el-select
              v-model="batchAddTags"
              multiple
              placeholder="Select tags to add"
              style="flex: 1"
              :disabled="availableTagsToAdd.length === 0"
            >
              <el-option
                v-for="tag in availableTagsToAdd"
                :key="tag.name"
                :label="tag.name"
                :value="tag.name"
              >
                <span :style="{ color: tag.color }">{{ tag.name }}</span>
              </el-option>
            </el-select>
          </div>
          <div class="batch-row">
            <span class="batch-label">Remove tags:</span>
            <el-select
              v-model="batchRemoveTags"
              multiple
              placeholder="Select tags to remove"
              style="flex: 1"
              :disabled="availableTagsToRemove.length === 0"
            >
              <el-option
                v-for="tag in availableTagsToRemove"
                :key="tag.name"
                :label="tag.name"
                :value="tag.name"
              >
                <span :style="{ color: tag.color }">{{ tag.name }}</span>
              </el-option>
            </el-select>
          </div>
          <el-button 
            type="primary" 
            @click="handleBatchUpdate"
            :loading="batchUpdating"
            :disabled="batchAddTags.length === 0 && batchRemoveTags.length === 0"
          >
            Apply Batch Changes
          </el-button>
        </div>
      </div>
    </div>

    <!-- Edit tag dialog -->
    <el-dialog
      v-model="editDialogVisible"
      title="Edit Tag"
      width="400px"
      append-to-body
    >
      <el-form :model="editForm" label-width="80px">
        <el-form-item label="Tag Name">
          <el-input v-model="editForm.name" />
        </el-form-item>
        <el-form-item label="Tag Color">
          <el-color-picker
            v-model="editForm.color"
            show-alpha
            :predefine="predefineColors"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="editDialogVisible = false">Cancel</el-button>
        <el-button type="primary" @click="handleUpdateTag" :loading="updating">
          Save
        </el-button>
      </template>
    </el-dialog>

    <template #footer>
      <el-button @click="handleClose">Close</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus, Edit, Delete } from '@element-plus/icons-vue';
import { useSettingsStore, useAccountsStore } from '@/store';
import type { GlobalTag } from '@/types';

const props = defineProps<{
  modelValue: boolean;
  selectedAccountIds?: string[];
}>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  'refresh': [];
}>();

const settingsStore = useSettingsStore();
const accountsStore = useAccountsStore();

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit('update:modelValue', val)
});

const selectedAccountIds = computed(() => props.selectedAccountIds || []);

// Predefined colors
const predefineColors = [
  '#ff4500',
  '#ff8c00',
  '#ffd700',
  '#90ee90',
  '#00ced1',
  '#1e90ff',
  '#c71585',
  '#ff69b4',
  '#ba55d3',
  '#cd5c5c',
  '#40e0d0',
  '#7b68ee',
  '#00fa9a',
  '#6495ed',
  '#dda0dd',
  '#a0522d',
];

// Add new tag
const newTagName = ref('');
const newTagColor = ref('#1e90ff');
const adding = ref(false);

// Edit tag
const editDialogVisible = ref(false);
const editForm = reactive({
  originalName: '',
  name: '',
  color: ''
});
const updating = ref(false);

// Batch operations
const batchAddTags = ref<string[]>([]);
const batchRemoveTags = ref<string[]>([]);
const batchUpdating = ref(false);

// Get selected accounts
const selectedAccounts = computed(() => {
  return accountsStore.accounts.filter(a => selectedAccountIds.value.includes(a.id));
});

// Tags that can be added: exclude tags that all selected accounts already have
const availableTagsToAdd = computed(() => {
  if (selectedAccounts.value.length === 0) return settingsStore.tags;
  
// Get the intersection of tags that all selected accounts have
   const commonTags = settingsStore.tags
     .map(t => t.name)
     .filter(tagName => selectedAccounts.value.every(account => account.tags.includes(tagName)));
   
   // Return tags that are not in the intersection (i.e., tags that at least one account does not have)
  return settingsStore.tags.filter(tag => !commonTags.includes(tag.name));
});

// Tags that can be removed: only show tags that at least one selected account has
const availableTagsToRemove = computed(() => {
  if (selectedAccounts.value.length === 0) return [];
  
// Get the union of tags from all selected accounts
   const allOwnedTags = new Set<string>();
   selectedAccounts.value.forEach(account => {
     account.tags.forEach(tag => allOwnedTags.add(tag));
   });
   
   // Only return tags that are owned by the selected accounts
  return settingsStore.tags.filter(tag => allOwnedTags.has(tag.name));
});

// Get tag style
function getTagStyle(color: string): Record<string, string> {
  if (!color) {
    return {
      backgroundColor: '#f0f2f5',
      color: '#606266',
      border: '1px solid #dcdfe6'
    };
  }
  
  let r = 0, g = 0, b = 0, a = 1;
  let parsed = false;
  
  // Parse rgba or rgb format
  if (color.startsWith('rgba') || color.startsWith('rgb')) {
    const match = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
    if (match) {
      r = parseInt(match[1]);
      g = parseInt(match[2]);
      b = parseInt(match[3]);
      a = match[4] ? parseFloat(match[4]) : 1;
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
  
  // If parsing fails, return default style
  if (!parsed) {
    return {
      backgroundColor: '#f0f2f5',
      color: '#606266',
      border: '1px solid #dcdfe6'
    };
  }
  
  const bgAlpha = Math.min(a * 0.25, 0.35);
  const borderAlpha = Math.min(a * 0.5, 0.6);
  
  return {
    backgroundColor: `rgba(${r}, ${g}, ${b}, ${bgAlpha})`,
    border: `1px solid rgba(${r}, ${g}, ${b}, ${borderAlpha})`,
    color: `rgba(${r}, ${g}, ${b}, ${Math.max(a, 0.8)})`
  };
}

// Get tag usage count
function getTagUsageCount(tagName: string): number {
  return accountsStore.accounts.filter(a => a.tags.includes(tagName)).length;
}

// Add tag
async function handleAddTag() {
  const name = newTagName.value.trim();
  if (!name) {
    ElMessage.warning('Please enter a tag name');
    return;
  }
  
  if (settingsStore.tags.some(t => t.name === name)) {
    ElMessage.warning('Tag already exists');
    return;
  }
  
  adding.value = true;
  try {
    await settingsStore.addTag({
      name,
      color: newTagColor.value
    });
    ElMessage.success('Tag added successfully');
    newTagName.value = '';
  } catch (e) {
    ElMessage.error(`Failed to add: ${e}`);
  } finally {
    adding.value = false;
  }
}

// Open edit dialog
function openEditDialog(tag: GlobalTag) {
  editForm.originalName = tag.name;
  editForm.name = tag.name;
  editForm.color = tag.color;
  editDialogVisible.value = true;
}

// Update color
async function handleUpdateColor(tagName: string, color: string | null) {
  if (!color) return;
  
  try {
    await settingsStore.updateTag(tagName, {
      name: tagName,
      color
    });
    ElMessage.success('Color updated successfully');
  } catch (e) {
    ElMessage.error(`Update failed: ${e}`);
  }
}

// Update tag
async function handleUpdateTag() {
  const name = editForm.name.trim();
  if (!name) {
    ElMessage.warning('Please enter a tag name');
    return;
  }
  
  // If the name is changed, check for duplicates
  if (name !== editForm.originalName && settingsStore.tags.some(t => t.name === name)) {
    ElMessage.warning('Tag name already exists');
    return;
  }
  
  updating.value = true;
  try {
    await settingsStore.updateTag(editForm.originalName, {
      name,
      color: editForm.color
    });
    ElMessage.success('Tag updated successfully');
    editDialogVisible.value = false;
    
    // If the name has changed, refresh the account list
    if (name !== editForm.originalName) {
      emit('refresh');
    }
  } catch (e) {
    ElMessage.error(`Update failed: ${e}`);
  } finally {
    updating.value = false;
  }
}

// Delete tag
async function handleDeleteTag(name: string) {
  const usageCount = getTagUsageCount(name);
  
  let message = `Are you sure you want to delete the tag \\"${name}\\"?`;
  if (usageCount > 0) {
    message += `\\n\\nThis tag is being used by ${usageCount} accounts. After deletion, this tag will be removed from these accounts.`;
  }
  
  try {
    await ElMessageBox.confirm(message, 'Confirm Deletion', {
      type: 'warning',
      confirmButtonText: 'Delete',
      cancelButtonText: 'Cancel'
    });
    
    await settingsStore.deleteTag(name);
    ElMessage.success('Tag deleted successfully');
    emit('refresh');
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error(`Deletion failed: ${e}`);
    }
  }
}

// Delete tag


// Batch update account tags
async function handleBatchUpdate() {
  if (batchAddTags.value.length === 0 && batchRemoveTags.value.length === 0) {
    ElMessage.warning('Please select tags to add or remove');
    return;
  }
  
  batchUpdating.value = true;
  try {
    const result = await settingsStore.batchUpdateAccountTags(
      selectedAccountIds.value,
      batchAddTags.value,
      batchRemoveTags.value
    );
    ElMessage.success(`Batch update completed: ${result.success_count} successful`);
    batchAddTags.value = [];
    batchRemoveTags.value = [];
    emit('refresh');
  } catch (e) {
    ElMessage.error(`Batch update failed: ${e}`);
  } finally {
    batchUpdating.value = false;
  }
}

function handleClose() {
  visible.value = false;
}
</script>

<style scoped>
.tag-manage-container {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.add-tag-section {
  padding-bottom: 16px;
  border-bottom: 1px solid #e4e7ed;
}

.tag-input {
  width: 100%;
}

.tag-input :deep(.el-input-group__prepend) {
  padding: 0 8px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
}

.empty-hint {
  text-align: center;
  padding: 30px;
  color: #909399;
  font-size: 14px;
}

.tag-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tag-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  background: #f8f9fa;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.tag-item:hover {
  background: #f0f2f5;
}

.tag-info {
  display: flex;
  align-items: center;
  gap: 12px;
}

.tag-preview {
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 13px;
  font-weight: 500;
}

.tag-usage {
  font-size: 12px;
  color: #909399;
}

.tag-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.batch-section {
  padding-top: 16px;
  border-top: 1px solid #e4e7ed;
}

.batch-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.batch-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.batch-label {
  min-width: 70px;
  font-size: 13px;
  color: #606266;
}

/* Dark mode */
:root.dark .section-title {
  color: #cfd3dc;
}

:root.dark .tag-item {
  background: #2a2a2a;
}

:root.dark .tag-item:hover {
  background: #333;
}

:root.dark .tag-usage {
  color: #64748b;
}

:root.dark .add-tag-section,
:root.dark .batch-section {
  border-color: #4c4d4f;
}

:root.dark .empty-hint {
  color: #64748b;
}

:root.dark .batch-label {
  color: #94a3b8;
}
</style>
