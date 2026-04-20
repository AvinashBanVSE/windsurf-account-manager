<template>
  <el-dialog
    v-model="dialogVisible"
    title="Team Settings"
    width="700px"
    :close-on-click-modal="false"
    destroy-on-close
    class="team-settings-dialog"
  >
    <div v-loading="loading" class="settings-container">
      <!-- Windsurf Settings -->
      <div class="settings-section">
        <h3 class="section-title">Windsurf Settings</h3>
        
        <!-- Models -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Model Configuration</span>
            <span class="setting-desc">Configure models to use in Windsurf</span>
          </div>
          <el-button size="small" @click="openModelsConfig">Configure</el-button>
        </div>
        
        <!-- Enable Web Search -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Web Search</span>
            <span class="setting-desc">Allow Cascade to search the web for relevant information</span>
          </div>
          <el-switch v-model="settings.enableWebSearch" @change="handleSettingChange" />
        </div>
        
        <!-- Auto Run Terminal Commands -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Auto Run Commands</span>
            <span class="setting-desc">Allow Cascade to automatically run commands on user machine</span>
          </div>
          <el-switch v-model="settings.allowAutoRunCommands" @change="handleSettingChange" />
        </div>
        
        <!-- MCP Servers -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">MCP Servers</span>
            <span class="setting-desc">Allow users to use and configure MCP servers</span>
          </div>
          <el-switch v-model="settings.allowMcpServers" @change="handleSettingChange" />
        </div>
        
        <!-- Whitelisted MCP Servers -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">MCP Whitelist</span>
            <span class="setting-desc">When no MCP servers are added, all servers are in the whitelist by default</span>
          </div>
          <el-button size="small" @click="openMcpWhitelist">Add Server</el-button>
        </div>
        
        <!-- App Deploys -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">App Deploys <el-tag size="small" type="warning">BETA</el-tag></span>
            <span class="setting-desc">Manage team deployment permissions in Cascade</span>
          </div>
          <el-select v-model="settings.appDeploysMode" size="small" style="width: 160px" @change="handleSettingChange">
            <el-option label="Full Deployment" value="full" />
            <el-option label="Internal Teams Only" value="teams" />
            <el-option label="Deployment Disabled" value="disabled" />
          </el-select>
        </div>
        
        <!-- Conversation Sharing -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Conversation Sharing</span>
            <span class="setting-desc">Allow team members to share Cascade conversations</span>
          </div>
          <el-switch v-model="settings.allowConversationSharing" @change="handleSettingChange" />
        </div>
        
        <!-- DeepWiki -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">DeepWiki</span>
            <span class="setting-desc">Enable DeepWiki floating cards and IDE articles for code symbols</span>
          </div>
          <el-switch v-model="settings.enableDeepwiki" @change="handleSettingChange" />
        </div>
        
        <!-- Fast Context -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Fast Context</span>
            <span class="setting-desc">Enable fast context functionality</span>
          </div>
          <el-switch v-model="settings.enableFastContext" @change="handleSettingChange" />
        </div>
        
        <!-- Codemaps -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Codemaps</span>
            <span class="setting-desc">Generate, view and share interactive code maps</span>
          </div>
          <el-button size="small" @click="codemapsDialogVisible = true">Configure</el-button>
        </div>
        
        <!-- Vibe and Replace -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Vibe and Replace</span>
            <span class="setting-desc">Enable Vibe and Replace feature for advanced code editing</span>
          </div>
          <el-switch v-model="settings.allowVibeAndReplace" @change="handleSettingChange" />
        </div>
        
        <!-- Github Integration -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Github Integration</span>
            <span class="setting-desc">Install Windsurf in team's GitHub organization for PR review</span>
          </div>
          <el-switch v-model="settings.allowGithubReviews" @change="handleSettingChange" />
        </div>
      </div>
      
      <!-- Other Settings -->
      <div class="settings-section">
        <h3 class="section-title">Other Settings</h3>
        
        <!-- Domain Management -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Domain Management</span>
            <span class="setting-desc">Manage and verify team domains</span>
          </div>
          <el-button size="small" disabled>Configure</el-button>
        </div>
        
        <!-- SSO and SAML Configuration -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">SSO and SAML Configuration</span>
            <span class="setting-desc">Set up Azure, Google, Okta or custom SAML single sign-on</span>
          </div>
          <el-button size="small" disabled>Configure</el-button>
        </div>
        
        <!-- Service Key Configuration -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Service Key Configuration</span>
            <span class="setting-desc">Generate service keys for SCIM user provisioning and analytics API</span>
          </div>
          <el-button size="small" disabled>Configure</el-button>
        </div>
        
        <!-- Role Management -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Role Management</span>
            <span class="setting-desc">Manage the list of roles users can have</span>
          </div>
          <el-button size="small" disabled>Configure</el-button>
        </div>
        
        <!-- Individual Level Analytics -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Individual Analytics</span>
            <span class="setting-desc">Usage analytics at individual level</span>
          </div>
          <el-switch v-model="settings.allowIndividualAnalytics" @change="handleSettingChange" />
        </div>
        
        <!-- Attribution Toggle -->
        <div class="setting-item">
          <div class="setting-info">
            <span class="setting-name">Attribution <el-tag size="small" type="warning">BETA</el-tag></span>
            <span class="setting-desc">When enabled Windsurf will block writing code with attribution</span>
          </div>
          <el-switch v-model="settings.allowAttribution" @change="handleSettingChange" />
        </div>
      </div>
    </div>
    
    <template #footer>
      <div class="dialog-footer">
        <el-button @click="dialogVisible = false">Close</el-button>
        <el-button type="primary" :loading="saving" @click="saveSettings">
          <el-icon><Check /></el-icon> Save Settings
        </el-button>
      </div>
    </template>
    
    <!-- Models Configuration Dialog -->
    <el-dialog
      v-model="modelsDialogVisible"
      title="Model Configuration"
      width="700px"
      append-to-body
      @open="loadModelsConfig"
    >
      <div class="models-config" v-loading="loadingModels">
        <p class="models-desc">Configure models to use in Windsurf, multiple models can be selected for each category</p>
        
        <!-- Cascade Models -->
        <div class="models-section">
          <h4>Cascade Models</h4>
          <el-select
            v-model="selectedCascadeModels"
            multiple
            filterable
            collapse-tags
            collapse-tags-tooltip
            :max-collapse-tags="5"
            placeholder="Select Cascade model"
            style="width: 100%"
            popper-class="model-select-dropdown"
          >
            <el-option
              v-for="model in availableCascadeModels"
              :key="model"
              :label="model"
              :value="model"
            >
              <div class="model-option">
                <span class="model-check" :class="{ checked: selectedCascadeModels.includes(model) }">✓</span>
                <span class="model-name">{{ model }}</span>
                <span class="model-multiplier" :class="getMultiplierClass(getModelMultiplier(model, 'cascade'))">
                  {{ formatMultiplier(getModelMultiplier(model, 'cascade')) }}
                </span>
              </div>
            </el-option>
          </el-select>
        </div>
        
        <!-- Command Models -->
        <div class="models-section">
          <h4>Command Models</h4>
          <el-select
            v-model="selectedCommandModels"
            multiple
            filterable
            collapse-tags
            collapse-tags-tooltip
            :max-collapse-tags="5"
            placeholder="Select Command model"
            style="width: 100%"
            popper-class="model-select-dropdown"
          >
            <el-option
              v-for="model in availableCommandModels"
              :key="model"
              :label="model"
              :value="model"
            >
              <div class="model-option">
                <span class="model-check" :class="{ checked: selectedCommandModels.includes(model) }">✓</span>
                <span class="model-name">{{ model }}</span>
                <span class="model-multiplier" :class="getMultiplierClass(getModelMultiplier(model, 'command'))">
                  {{ formatMultiplier(getModelMultiplier(model, 'command')) }}
                </span>
              </div>
            </el-option>
          </el-select>
        </div>
        
        <!-- Extension Models -->
        <div class="models-section">
          <h4>Extension Models</h4>
          <el-select
            v-model="selectedExtensionModels"
            multiple
            filterable
            collapse-tags
            collapse-tags-tooltip
            :max-collapse-tags="5"
            placeholder="Select Extension model"
            style="width: 100%"
            popper-class="model-select-dropdown"
          >
            <el-option
              v-for="model in availableExtensionModels"
              :key="model"
              :label="model"
              :value="model"
            >
              <div class="model-option">
                <span class="model-check" :class="{ checked: selectedExtensionModels.includes(model) }">✓</span>
                <span class="model-name">{{ model }}</span>
              </div>
            </el-option>
          </el-select>
        </div>
      </div>
      <template #footer>
        <el-button @click="modelsDialogVisible = false">Cancel</el-button>
        <el-button type="primary" :loading="savingModels" @click="saveModelsConfig">Save</el-button>
      </template>
    </el-dialog>
    
    <!-- MCP Whitelist Dialog -->
    <el-dialog
      v-model="mcpDialogVisible"
      title="Add MCP Server"
      width="500px"
      append-to-body
    >
      <div class="mcp-add-dialog">
        <div class="mcp-header">
          <span class="mcp-label">Server ID</span>
          <el-link type="primary" @click="mcpManualMode = !mcpManualMode">
            {{ mcpManualMode ? 'Select from list' : 'Manual input' }}
          </el-link>
        </div>
        
        <!-- Dropdown selection mode -->
        <el-select
          v-if="!mcpManualMode"
          v-model="selectedMcpPlugin"
          placeholder="Select MCP server"
          filterable
          style="width: 100%; margin-bottom: 16px"
          :loading="loadingPlugins"
          @visible-change="onMcpSelectOpen"
        >
          <el-option
            v-for="plugin in availableMcpPlugins"
            :key="plugin.id"
            :label="plugin.title"
            :value="plugin.id"
          >
            <div style="display: flex; flex-direction: column;">
              <span style="font-weight: 500;">{{ plugin.title }}</span>
              <span style="font-size: 12px; color: #999;">{{ plugin.id }}</span>
            </div>
          </el-option>
        </el-select>
        
        <!-- Manual input mode -->
        <el-input
          v-else
          v-model="newMcpServer"
          placeholder="Enter MCP server ID"
          style="margin-bottom: 16px"
        />
        
        <div class="mcp-config-section">
          <span class="mcp-label">Server Configuration (JSON)</span>
          <el-input
            v-model="mcpServerConfig"
            type="textarea"
            :rows="4"
            placeholder="Refer to MCP server docs for config. Leave empty for default."
          />
        </div>
        
        <div class="mcp-help-text">
          Need help? Check the <el-link type="primary" href="https://docs.windsurf.com/mcp" target="_blank">documentation</el-link> for configuration details
        </div>
        
        <!-- Added servers list -->
        <div class="mcp-added-list" v-if="mcpServers.length">
          <div class="mcp-added-header">Added Servers</div>
          <div class="mcp-list">
            <el-tag
              v-for="(server, index) in mcpServers"
              :key="index"
              closable
              @close="removeMcpServer(index)"
              class="mcp-tag"
            >
              {{ server }}
            </el-tag>
          </div>
        </div>
      </div>
      <template #footer>
        <el-button @click="mcpDialogVisible = false">Cancel</el-button>
        <el-button type="primary" @click="addSelectedMcpServer">Add</el-button>
      </template>
    </el-dialog>
    
    <!-- Codemaps Configuration Dialog -->
    <el-dialog
      v-model="codemapsDialogVisible"
      title="Codemaps"
      width="500px"
      append-to-body
    >
      <div class="codemaps-config">
        <p class="codemaps-desc">Generate, view and share interactive code maps</p>
        
        <div class="codemaps-setting">
          <div class="setting-info">
            <span class="setting-name">Enable Codemaps</span>
            <span class="setting-desc">Allow team to generate and view code maps</span>
          </div>
          <el-switch v-model="settings.enableCodemaps" />
        </div>
        
        <div class="codemaps-setting">
          <div class="setting-info">
            <span class="setting-name">Codemap Sharing</span>
            <span class="setting-desc">Allow team to share code maps</span>
          </div>
          <el-select v-model="settings.codemapSharing" size="small" style="width: 130px">
            <el-option label="Enabled" value="enabled" />
            <el-option label="Team Only" value="team" />
            <el-option label="Disabled" value="disabled" />
          </el-select>
        </div>
      </div>
      <template #footer>
        <el-button @click="codemapsDialogVisible = false">Cancel</el-button>
        <el-button type="primary" @click="saveCodemapsConfig">Save</el-button>
      </template>
    </el-dialog>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch, reactive } from 'vue';
import { ElMessage } from 'element-plus';
import { Check } from '@element-plus/icons-vue';
import { invoke } from '@tauri-apps/api/core';

interface Props {
  modelValue: boolean;
  accountId: string;
}

const props = defineProps<Props>();
const emit = defineEmits(['update:modelValue']);

const dialogVisible = ref(false);
const loading = ref(false);
const saving = ref(false);
const modelsDialogVisible = ref(false);
const mcpDialogVisible = ref(false);
const codemapsDialogVisible = ref(false);
const newMcpServer = ref('');
const mcpServers = ref<string[]>([]);

// MCP Plugin related
const mcpManualMode = ref(false);
const selectedMcpPlugin = ref('');
const loadingPlugins = ref(false);
const mcpServerConfig = ref('');
const availableMcpPlugins = ref<Array<{id: string; title: string; description?: string; trustLevel?: string}>>([]);

// Settings state
const settings = reactive({
  enableWebSearch: false,
  canEditWebSearch: false,
  allowAutoRunCommands: true,
  allowMcpServers: true,
  appDeploysMode: 'disabled',
  allowConversationSharing: false,
  enableDeepwiki: false,
  enableCodemaps: false,
  codemapSharing: 'enabled',
  enableFastContext: false,
  allowVibeAndReplace: false,
  allowGithubReviews: false,
  allowGithubDescriptionEdits: false,
  allowIndividualAnalytics: false,
  allowAttribution: false,
  allowBrowserFeatures: false,
});

// Models configuration
interface ModelInfo {
  name: string;
  multiplier: number; // Multiplier: 0=Free, 0.5=0.5x, 1=1x, 2=2x, 3=3x
}

const loadingModels = ref(false);
const savingModels = ref(false);
const teamId = ref('');
const cascadeModelInfos = ref<ModelInfo[]>([]);
const commandModelInfos = ref<ModelInfo[]>([]);
const extensionModelInfos = ref<ModelInfo[]>([]);
const availableCascadeModels = ref<string[]>([]);
const availableCommandModels = ref<string[]>([]);
const availableExtensionModels = ref<string[]>([]);
const selectedCascadeModels = ref<string[]>([]);
const selectedCommandModels = ref<string[]>([]);
const selectedExtensionModels = ref<string[]>([]);

// Get model multiplier
function getModelMultiplier(modelName: string, type: 'cascade' | 'command' | 'extension'): number {
  let infos: ModelInfo[] = [];
  if (type === 'cascade') infos = cascadeModelInfos.value;
  else if (type === 'command') infos = commandModelInfos.value;
  else infos = extensionModelInfos.value;
  
  const model = infos.find(m => m.name === modelName);
  return model?.multiplier ?? 1;
}

// Format multiplier display
function formatMultiplier(multiplier: number): string {
  if (multiplier === 0) return '(Free)';
  if (multiplier === 1) return '(1x credits)';
  if (multiplier < 1) return `(${multiplier}x credits)`;
  return `(${multiplier}x credits)`;
}

// Get multiplier style class
function getMultiplierClass(multiplier: number): string {
  if (multiplier === 0) return 'free';
  if (multiplier <= 0.5) return 'low';
  if (multiplier <= 1) return 'normal';
  if (multiplier <= 2) return 'high';
  return 'very-high';
}

watch(() => props.modelValue, (val) => {
  dialogVisible.value = val;
  if (val) {
    loadSettings();
  }
});

watch(dialogVisible, (val) => {
  emit('update:modelValue', val);
});

async function loadSettings() {
  loading.value = true;
  try {
    // Call GetTeamConfigRecord API to get current settings
    console.log('Loading team config for account:', props.accountId);
    const result = await invoke('get_team_config', { id: props.accountId }) as any;
    console.log('Team config result:', JSON.stringify(result, null, 2));
    
    if (result.success && result.data) {
      const config = result.data;
      // Parse nested team config data
      // Response format: { "subMesssage_1": { "int_5": 1, ... } } or { "1": { ... } }
      const teamConfig = config["subMesssage_1"] || config["1"] || config;
      console.log('Parsed teamConfig:', JSON.stringify(teamConfig, null, 2));
      
      // Helper function: Get field value (supports both int_X and X formats)
      const getField = (fieldNum: number): any => {
        return teamConfig[`int_${fieldNum}`] ?? teamConfig[`string_${fieldNum}`] ?? teamConfig[`${fieldNum}`] ?? teamConfig[fieldNum];
      };
      
      // Helper function: Parse protobuf boolean (1=true, 0/undefined=false)
      const parseBool = (fieldNum: number, defaultVal = false): boolean => {
        const val = getField(fieldNum);
        if (val === 1 || val === true || val === "1") return true;
        if (val === 0 || val === false || val === "0") return false;
        return defaultVal;
      };
      
      // TeamConfig proto field mapping:
      // field 5: allow_mcp_servers, field 7: allow_auto_run_commands
      // field 10: allow_app_deployments, field 12: allow_github_reviews
      // field 13: allow_github_description_edits, field 17: allow_individual_level_analytics
      // field 18: allow_conversation_sharing, field 19: allow_sandbox_app_deployments
      // field 20: allow_teams_app_deployments, field 22: allow_attribution
      // field 25: allow_browser_experimental_features, field 27: allow_vibe_and_replace
      // field 28: disable_deepwiki, field 31: disable_codemaps, field 33: disable_fast_context
      
      // Allow fields (allow_xxx): true if exists and is 1
      settings.allowAutoRunCommands = parseBool(7, true);
      settings.allowMcpServers = parseBool(5, true);
      settings.allowConversationSharing = parseBool(18, false);
      settings.allowVibeAndReplace = parseBool(27, false);
      settings.allowGithubReviews = parseBool(12, false);
      settings.allowGithubDescriptionEdits = parseBool(13, false);
      settings.allowIndividualAnalytics = parseBool(17, false);
      settings.allowAttribution = parseBool(22, false);
      settings.allowBrowserFeatures = parseBool(25, false);
      
      // Disable fields (disable_xxx): if 1, feature is disabled, UI shows as off
      // If field does not exist, it means not disabled, feature is enabled
      settings.enableDeepwiki = !parseBool(28, false);
      settings.enableCodemaps = !parseBool(31, false);
      settings.enableFastContext = !parseBool(33, false);
      
      // Codemap sharing (field 32, string: "enabled" or "disabled")
      const codemapSharingValue = getField(32);
      settings.codemapSharing = codemapSharingValue || 'enabled';
      
      // App deploys mode
      // field 10: allow_app_deployments, field 19: allow_sandbox_app_deployments, field 20: allow_teams_app_deployments
      const allowAppDeploys = parseBool(10, false);
      const allowSandboxDeploys = parseBool(19, false);
      const allowTeamsDeploys = parseBool(20, false);
      
      if (allowAppDeploys && allowSandboxDeploys && allowTeamsDeploys) {
        settings.appDeploysMode = 'full';  // Full deploy access
      } else if (allowTeamsDeploys) {
        settings.appDeploysMode = 'teams'; // Internal teams only
      } else {
        settings.appDeploysMode = 'disabled'; // Disable deploys
      }
      
      // MCP servers whitelist (field 23)
      const mcpList = teamConfig["subMesssage_23"] || teamConfig["23"];
      if (mcpList && Array.isArray(mcpList)) {
        mcpServers.value = mcpList;
      }
      
      console.log('Parsed settings:', {
        allowAutoRunCommands: settings.allowAutoRunCommands,
        allowMcpServers: settings.allowMcpServers,
        enableDeepwiki: settings.enableDeepwiki,
        enableCodemaps: settings.enableCodemaps,
        enableFastContext: settings.enableFastContext,
        allowVibeAndReplace: settings.allowVibeAndReplace,
        allowConversationSharing: settings.allowConversationSharing,
        allowIndividualAnalytics: settings.allowIndividualAnalytics,
      });
    } else if (result.error) {
      console.warn('Failed to load team config:', result.error);
      // If it's a permission issue, handle silently
      ElMessage.warning('This account may not have team management permissions');
    }
  } catch (error: any) {
    console.error('Failed to load team settings:', error);
    const errorMsg = error?.message || error?.toString() || 'Unknown error';
    ElMessage.warning(`Loading failed: ${errorMsg}`);
  } finally {
    loading.value = false;
  }
}

function handleSettingChange() {
  // Can add instant save logic here
}

async function saveSettings() {
  saving.value = true;
  try {
    const updateData = {
      allow_auto_run_commands: settings.allowAutoRunCommands,
      allow_mcp_servers: settings.allowMcpServers,
      allow_conversation_sharing: settings.allowConversationSharing,
      disable_deepwiki: !settings.enableDeepwiki,
      disable_codemaps: !settings.enableCodemaps,
      disable_fast_context: !settings.enableFastContext,
      allow_vibe_and_replace: settings.allowVibeAndReplace,
      allow_github_reviews: settings.allowGithubReviews,
      allow_github_description_edits: settings.allowGithubDescriptionEdits,
      allow_individual_level_analytics: settings.allowIndividualAnalytics,
      allow_attribution: settings.allowAttribution,
      allow_browser_experimental_features: settings.allowBrowserFeatures,
      // App deploys: full = all three, teams = only teams, disabled = none
      allow_app_deployments: settings.appDeploysMode === 'full',
      allow_sandbox_app_deployments: settings.appDeploysMode === 'full',
      allow_teams_app_deployments: settings.appDeploysMode === 'full' || settings.appDeploysMode === 'teams',
      allowed_mcp_servers: mcpServers.value.join(','),
      // Codemaps
      allow_codemap_sharing: settings.codemapSharing,
    };
    
    const result = await invoke('update_team_config', { id: props.accountId, config: updateData }) as any;
    if (result.success) {
      ElMessage.success('Team settings saved');
    } else {
      ElMessage.error(result.error || 'Save failed');
    }
  } catch (error) {
    console.error('Failed to save team settings:', error);
    ElMessage.error('Failed to save team settings');
  } finally {
    saving.value = false;
  }
}

function openModelsConfig() {
  modelsDialogVisible.value = true;
}

async function loadModelsConfig() {
  loadingModels.value = true;
  try {
    // Get available models list
    const modelConfigResult = await invoke('get_cascade_model_configs', { id: props.accountId }) as any;
    console.log('Model configs:', modelConfigResult);
    
    if (modelConfigResult.success && modelConfigResult.data) {
      const data = modelConfigResult.data;
      console.log('Model data keys:', Object.keys(data));
      console.log('Model data full:', JSON.stringify(data, null, 2).substring(0, 2000));
      console.log('subMesssage_2 (sorts):', JSON.stringify(data.subMesssage_2, null, 2));
      console.log('subMesssage_3 (default):', JSON.stringify(data.subMesssage_3, null, 2));
      
      // Response format: CascadeModelConfigData
      // - subMesssage_1 = client_model_configs (repeated ClientModelConfig)
      // - Each ClientModelConfig's string_1 = label (model name)
      let modelConfigs: any[] = [];
      
      // Try multiple possible field names and nesting levels
      const field1 = data.subMesssage_1 || data.subMessage_1 || data.repeated_1 || data["1"];
      console.log('Field1 type:', typeof field1, 'isArray:', Array.isArray(field1));
      
      if (Array.isArray(field1)) {
        modelConfigs = field1;
      } else if (field1 && typeof field1 === 'object') {
        // Check if there are nested repeated fields
        const nested = field1.repeated_1 || field1.subMesssage_1 || field1["1"];
        if (Array.isArray(nested)) {
          modelConfigs = nested;
        } else if (field1.string_1) {
          // If it's a single object, check if it has string_1 field
          modelConfigs = [field1];
        }
      }
      
      console.log('Model configs count:', modelConfigs.length);
      
      // Extract model name and multiplier - from each element's string_1 and float_3 fields
      // ClientModelConfig: label=1(string), multiplier=3(float), disabled=4(bool)
      const allModels: string[] = [];
      const enabledModels: string[] = [];
      const modelInfos: ModelInfo[] = [];
      
      if (Array.isArray(modelConfigs)) {
        modelConfigs.forEach((config: any, idx: number) => {
          const modelName = config.string_1 || config.label || config["1"];
          // multiplier field (field 3): multiplier, absence of this field indicates free model
          const rawMultiplier = config.float_3 ?? config.multiplier ?? config["3"];
          const multiplier = rawMultiplier !== undefined ? rawMultiplier : 0; // No float_3 = Free
          // disabled field (field 4): true=not selected, false=selected
          const isDisabled = config.bool_4 === true || config.disabled === true || config["4"] === true;
          
          console.log(`Config ${idx}:`, modelName, 'multiplier:', multiplier, 'disabled:', isDisabled);
          if (modelName && typeof modelName === 'string') {
            allModels.push(modelName);
            modelInfos.push({ name: modelName, multiplier: typeof multiplier === 'number' ? multiplier : 1 });
            if (!isDisabled) {
              enabledModels.push(modelName);
            }
          }
        });
      }
      
      console.log('All cascade models:', allModels.length, 'Enabled:', enabledModels.length);
      
      // Available models = all models, selected models = models with disabled=false
      cascadeModelInfos.value = modelInfos;
      availableCascadeModels.value = allModels.sort();
      selectedCascadeModels.value = enabledModels;
      console.log('Selected cascade models:', enabledModels);
      
      availableExtensionModels.value = [
        'Base Model ⚡️',
        'GPT-4o',
        'o1-preview',
        'o1-mini',
        'Codeium Premier 🔥',
        'Claude 3.7 Sonnet',
        'Claude 3.5 Sonnet',
      ];
    }
    
    // Get Command model configuration
    const commandConfigResult = await invoke('get_command_model_configs', { id: props.accountId }) as any;
    console.log('Command configs:', commandConfigResult);
    
    if (commandConfigResult.success && commandConfigResult.data) {
      const cmdData = commandConfigResult.data;
      const cmdField1 = cmdData.subMesssage_1 || cmdData.subMessage_1 || cmdData.repeated_1 || cmdData["1"];
      
      if (Array.isArray(cmdField1)) {
        // Get model list from Command API and set as selected
        const commandModels: string[] = [];
        const cmdModelInfos: ModelInfo[] = [];
        cmdField1.forEach((config: any) => {
          const modelName = config.string_1 || config.label || config["1"];
          // multiplier field (field 3): multiplier, absence of this field indicates free model
          const rawMultiplier = config.float_3 ?? config.multiplier ?? config["3"];
          const multiplier = rawMultiplier !== undefined ? rawMultiplier : 0; // No float_3 = Free
          if (modelName) {
            commandModels.push(modelName);
            cmdModelInfos.push({ name: modelName, multiplier: typeof multiplier === 'number' ? multiplier : 1 });
          }
        });
        
        console.log('Command models from API:', commandModels);
        commandModelInfos.value = cmdModelInfos;
        availableCommandModels.value = commandModels;
        selectedCommandModels.value = commandModels; // Command API returns selected models
      }
    }
    
    // Get current team's model configuration
    const controlsResult = await invoke('get_team_organizational_controls', { id: props.accountId }) as any;
    console.log('Team controls:', controlsResult);
    console.log('Team controls data keys:', controlsResult.data ? Object.keys(controlsResult.data) : 'no data');
    
    if (controlsResult.success && controlsResult.data) {
      const data = controlsResult.data;
      console.log('Full data:', JSON.stringify(data, null, 2));
      
      // Response format: { subMessage_1: { string_1: team_id, repeated_2: [...], repeated_3: [...], repeated_6: [...] } }
      const controls = data.subMessage_1 || data.subMesssage_1 || data["1"] || data;
      console.log('Controls keys:', Object.keys(controls));
      console.log('Controls:', controls);
      
      // Parse team_id, may contain protobuf prefix that needs cleaning
      let rawTeamId = controls.string_1 || controls["1"] || '';
      // Remove possible protobuf nested message prefix (e.g., "\n$")
      if (typeof rawTeamId === 'string' && rawTeamId.includes('$')) {
        rawTeamId = rawTeamId.substring(rawTeamId.indexOf('$') + 1);
      }
      teamId.value = rawTeamId;
      
      // Parse selected models - Rust parser uses string_X format
      const parseRepeatedString = (field: any): string[] => {
        if (Array.isArray(field)) return field;
        if (typeof field === 'string') return [field];
        return [];
      };
      
      // string_2 = Cascade, string_3 = Command, string_6 = Extension
      // Only override if API returns selected models, otherwise keep defaults from Recommended list
      const cascadeFromApi = parseRepeatedString(controls.string_2 || controls.repeated_2 || controls["2"]);
      const commandFromApi = parseRepeatedString(controls.string_3 || controls.repeated_3 || controls["3"]);
      const extensionFromApi = parseRepeatedString(controls.string_6 || controls.repeated_6 || controls["6"]);
      
      if (cascadeFromApi.length > 0) {
        selectedCascadeModels.value = cascadeFromApi;
      }
      if (commandFromApi.length > 0) {
        selectedCommandModels.value = commandFromApi;
      }
      if (extensionFromApi.length > 0) {
        selectedExtensionModels.value = extensionFromApi;
      }
      
      // Also update available list from selected models (if there are new models)
      selectedCascadeModels.value.forEach(m => {
        if (!availableCascadeModels.value.includes(m)) {
          availableCascadeModels.value.push(m);
        }
      });
      selectedCommandModels.value.forEach(m => {
        if (!availableCommandModels.value.includes(m)) {
          availableCommandModels.value.push(m);
        }
      });
      selectedExtensionModels.value.forEach(m => {
        if (!availableExtensionModels.value.includes(m)) {
          availableExtensionModels.value.push(m);
        }
      });
      
      console.log('Parsed models:', {
        teamId: teamId.value,
        cascade: selectedCascadeModels.value,
        command: selectedCommandModels.value,
        extension: selectedExtensionModels.value,
      });
    }
  } catch (error) {
    console.error('Failed to load models config:', error);
    ElMessage.error('Failed to load model configuration');
  } finally {
    loadingModels.value = false;
  }
}

async function saveModelsConfig() {
  if (!teamId.value) {
    ElMessage.warning('This account is not a team account, cannot save model configuration');
    return;
  }
  
  savingModels.value = true;
  try {
    const result = await invoke('upsert_team_organizational_controls', {
      id: props.accountId,
      teamId: teamId.value,
      cascadeModels: selectedCascadeModels.value,
      commandModels: selectedCommandModels.value,
      extensionModels: selectedExtensionModels.value,
    }) as any;
    
    if (result.success) {
      ElMessage.success('Model configuration saved');
      modelsDialogVisible.value = false;
    } else {
      ElMessage.error(result.error || 'Save failed');
    }
  } catch (error) {
    console.error('Failed to save models config:', error);
    ElMessage.error('Failed to save model configuration');
  } finally {
    savingModels.value = false;
  }
}

function saveCodemapsConfig() {
  // Codemaps settings will be saved together during main save
  codemapsDialogVisible.value = false;
  ElMessage.success('Codemaps configuration updated');
}

function openMcpWhitelist() {
  mcpDialogVisible.value = true;
  // Reset state
  selectedMcpPlugin.value = '';
  newMcpServer.value = '';
  mcpServerConfig.value = '';
  mcpManualMode.value = false;
}

async function onMcpSelectOpen(visible: boolean) {
  if (visible && availableMcpPlugins.value.length === 0) {
    await loadMcpPlugins();
  }
}

async function loadMcpPlugins() {
  loadingPlugins.value = true;
  try {
    const result = await invoke('get_available_mcp_plugins', { id: props.accountId }) as any;
    if (result.success && result.data?.plugins) {
      availableMcpPlugins.value = result.data.plugins.map((p: any) => ({
        id: p.id,
        title: p.title,
        description: p.description,
        trustLevel: p.trustLevel
      }));
      // Sort by title
      availableMcpPlugins.value.sort((a, b) => a.title.localeCompare(b.title));
    }
  } catch (error) {
    console.error('Failed to load MCP plugins:', error);
  } finally {
    loadingPlugins.value = false;
  }
}

function addSelectedMcpServer() {
  const serverId = mcpManualMode.value ? newMcpServer.value.trim() : selectedMcpPlugin.value;
  if (serverId && !mcpServers.value.includes(serverId)) {
    mcpServers.value.push(serverId);
    // Reset input
    selectedMcpPlugin.value = '';
    newMcpServer.value = '';
    mcpServerConfig.value = '';
    ElMessage.success(`Added ${serverId}`);
  } else if (mcpServers.value.includes(serverId)) {
    ElMessage.warning('This server is already in the whitelist');
  } else {
    ElMessage.warning('Please select or enter server ID');
  }
}

function removeMcpServer(index: number) {
  mcpServers.value.splice(index, 1);
}
</script>

<style lang="scss" scoped>
.team-settings-dialog {
  :deep(.el-dialog__body) {
    padding: 16px 24px;
    max-height: 70vh;
    overflow-y: auto;
  }
}

.settings-container {
  min-height: 300px;
}

.settings-section {
  margin-bottom: 24px;
  
  .section-title {
    font-size: 16px;
    font-weight: 600;
    color: #303133;
    margin-bottom: 16px;
    padding-bottom: 8px;
    border-bottom: 2px solid #e4e7ed;
  }
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid #f0f2f5;
  
  &:last-child {
    border-bottom: none;
  }
  
  .setting-info {
    flex: 1;
    margin-right: 16px;
    
    .setting-name {
      display: block;
      font-size: 14px;
      font-weight: 500;
      color: #303133;
      margin-bottom: 4px;
      
      .el-tag {
        margin-left: 8px;
        vertical-align: middle;
      }
    }
    
    .setting-desc {
      display: block;
      font-size: 12px;
      color: #909399;
      line-height: 1.4;
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.models-config {
  .models-desc {
    font-size: 14px;
    color: #666;
    margin-bottom: 20px;
  }
  
  .models-section {
    margin-bottom: 20px;
    
    h4 {
      font-size: 14px;
      font-weight: 600;
      margin-bottom: 12px;
      color: #303133;
    }
  }
}

// Dropdown option styles (global styles)
:global(.model-select-dropdown) {
  .el-select-dropdown__item {
    padding: 8px 12px;
    height: auto;
  }
}

.model-option {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  
  .model-check {
    width: 16px;
    color: transparent;
    font-weight: bold;
    
    &.checked {
      color: #10b981;
    }
  }
  
  .model-name {
    flex: 1;
  }
  
  .model-multiplier {
    font-size: 12px;
    margin-left: auto;
    padding: 2px 6px;
    border-radius: 4px;
    
    &.free {
      color: #10b981;
    }
    
    &.low {
      color: #3b82f6;
    }
    
    &.normal {
      color: #6b7280;
    }
    
    &.high {
      color: #f59e0b;
    }
    
    &.very-high {
      color: #ef4444;
    }
  }
}

.codemaps-config {
  .codemaps-desc {
    font-size: 14px;
    color: #666;
    margin-bottom: 20px;
  }
  
  .codemaps-setting {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 0;
    border-bottom: 1px solid #f0f0f0;
    
    &:last-child {
      border-bottom: none;
    }
    
    .setting-info {
      flex: 1;
      
      .setting-name {
        display: block;
        font-weight: 500;
        color: #333;
        margin-bottom: 4px;
      }
      
      .setting-desc {
        font-size: 12px;
        color: #999;
      }
    }
  }
}

.mcp-whitelist {
  .mcp-list {
    margin-top: 16px;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  
  .mcp-tag {
    font-size: 13px;
  }
}

.mcp-add-dialog {
  .mcp-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  
  .mcp-label {
    font-weight: 500;
    color: #333;
    display: block;
    margin-bottom: 8px;
  }
  
  .mcp-config-section {
    margin-bottom: 16px;
  }
  
  .mcp-help-text {
    font-size: 12px;
    color: #666;
    margin-bottom: 16px;
  }
  
  .mcp-added-list {
    margin-top: 20px;
    padding-top: 16px;
    border-top: 1px solid #eee;
    
    .mcp-added-header {
      font-weight: 500;
      color: #333;
      margin-bottom: 12px;
    }
    
    .mcp-list {
      display: flex;
      flex-wrap: wrap;
      gap: 8px;
    }
    
    .mcp-tag {
      font-size: 13px;
    }
  }
}
</style>
