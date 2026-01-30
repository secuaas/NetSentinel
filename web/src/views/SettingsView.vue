<script setup lang="ts">
import { ref } from 'vue'

const activeTab = ref('general')

const tabs = [
  { id: 'general', label: 'General' },
  { id: 'capture', label: 'Capture' },
  { id: 'alerts', label: 'Alerts' },
  { id: 'users', label: 'Users' }
]

// General settings
const settings = ref({
  retention_days: 30,
  auto_classify: true,
  resolve_hostnames: true,
  track_vendors: true
})

// Capture settings
const captureSettings = ref({
  interfaces: ['eth0'],
  promiscuous: true,
  snap_length: 65535,
  buffer_size: 32
})

// Alert settings
const alertSettings = ref({
  new_device_alert: true,
  port_scan_threshold: 10,
  bandwidth_threshold: 100,
  email_notifications: false,
  email_address: ''
})

function saveSettings() {
  // API call would go here
  console.log('Saving settings...')
}
</script>

<template>
  <div class="space-y-6">
    <!-- Tabs -->
    <div class="border-b border-gray-700">
      <nav class="flex space-x-8">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="activeTab = tab.id"
          :class="[
            'pb-4 text-sm font-medium transition-colors',
            activeTab === tab.id
              ? 'text-primary-400 border-b-2 border-primary-400'
              : 'text-gray-400 hover:text-gray-200'
          ]"
        >
          {{ tab.label }}
        </button>
      </nav>
    </div>

    <!-- General settings -->
    <div v-if="activeTab === 'general'" class="card space-y-6">
      <h3 class="text-lg font-semibold text-gray-100">General Settings</h3>

      <div>
        <label class="block text-sm text-gray-400 mb-2">Data Retention (days)</label>
        <input
          v-model.number="settings.retention_days"
          type="number"
          min="1"
          max="365"
          class="input w-40"
        />
        <p class="mt-1 text-xs text-gray-500">How long to keep flow and device data</p>
      </div>

      <div class="space-y-3">
        <label class="flex items-center space-x-3">
          <input
            v-model="settings.auto_classify"
            type="checkbox"
            class="rounded bg-dark-200 border-gray-600 text-primary-500 focus:ring-primary-500"
          />
          <span class="text-gray-300">Auto-classify devices</span>
        </label>

        <label class="flex items-center space-x-3">
          <input
            v-model="settings.resolve_hostnames"
            type="checkbox"
            class="rounded bg-dark-200 border-gray-600 text-primary-500 focus:ring-primary-500"
          />
          <span class="text-gray-300">Resolve hostnames via DNS</span>
        </label>

        <label class="flex items-center space-x-3">
          <input
            v-model="settings.track_vendors"
            type="checkbox"
            class="rounded bg-dark-200 border-gray-600 text-primary-500 focus:ring-primary-500"
          />
          <span class="text-gray-300">Track vendor from MAC OUI</span>
        </label>
      </div>

      <button @click="saveSettings" class="btn btn-primary">
        Save Changes
      </button>
    </div>

    <!-- Capture settings -->
    <div v-if="activeTab === 'capture'" class="card space-y-6">
      <h3 class="text-lg font-semibold text-gray-100">Capture Settings</h3>

      <div>
        <label class="block text-sm text-gray-400 mb-2">Network Interfaces</label>
        <p class="text-gray-300">{{ captureSettings.interfaces.join(', ') }}</p>
        <p class="mt-1 text-xs text-gray-500">Configure in capture.toml</p>
      </div>

      <div class="grid grid-cols-2 gap-4">
        <div>
          <label class="block text-sm text-gray-400 mb-2">Snap Length (bytes)</label>
          <input
            v-model.number="captureSettings.snap_length"
            type="number"
            class="input w-full"
          />
        </div>
        <div>
          <label class="block text-sm text-gray-400 mb-2">Buffer Size (MB)</label>
          <input
            v-model.number="captureSettings.buffer_size"
            type="number"
            class="input w-full"
          />
        </div>
      </div>

      <label class="flex items-center space-x-3">
        <input
          v-model="captureSettings.promiscuous"
          type="checkbox"
          class="rounded bg-dark-200 border-gray-600 text-primary-500 focus:ring-primary-500"
        />
        <span class="text-gray-300">Promiscuous mode</span>
      </label>

      <button @click="saveSettings" class="btn btn-primary">
        Save Changes
      </button>
    </div>

    <!-- Alert settings -->
    <div v-if="activeTab === 'alerts'" class="card space-y-6">
      <h3 class="text-lg font-semibold text-gray-100">Alert Settings</h3>

      <label class="flex items-center space-x-3">
        <input
          v-model="alertSettings.new_device_alert"
          type="checkbox"
          class="rounded bg-dark-200 border-gray-600 text-primary-500 focus:ring-primary-500"
        />
        <span class="text-gray-300">Alert on new devices</span>
      </label>

      <div>
        <label class="block text-sm text-gray-400 mb-2">Port Scan Detection Threshold</label>
        <input
          v-model.number="alertSettings.port_scan_threshold"
          type="number"
          min="1"
          class="input w-40"
        />
        <p class="mt-1 text-xs text-gray-500">Number of ports per second to trigger alert</p>
      </div>

      <div>
        <label class="block text-sm text-gray-400 mb-2">High Bandwidth Threshold (Mbps)</label>
        <input
          v-model.number="alertSettings.bandwidth_threshold"
          type="number"
          min="1"
          class="input w-40"
        />
      </div>

      <div class="pt-4 border-t border-gray-700">
        <label class="flex items-center space-x-3 mb-4">
          <input
            v-model="alertSettings.email_notifications"
            type="checkbox"
            class="rounded bg-dark-200 border-gray-600 text-primary-500 focus:ring-primary-500"
          />
          <span class="text-gray-300">Email notifications</span>
        </label>

        <div v-if="alertSettings.email_notifications">
          <label class="block text-sm text-gray-400 mb-2">Email Address</label>
          <input
            v-model="alertSettings.email_address"
            type="email"
            class="input w-full max-w-md"
            placeholder="alerts@example.com"
          />
        </div>
      </div>

      <button @click="saveSettings" class="btn btn-primary">
        Save Changes
      </button>
    </div>

    <!-- Users settings -->
    <div v-if="activeTab === 'users'" class="card space-y-6">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold text-gray-100">Users</h3>
        <button class="btn btn-primary">Add User</button>
      </div>

      <table class="w-full">
        <thead>
          <tr class="text-left text-sm text-gray-400 border-b border-gray-700">
            <th class="pb-3 font-medium">Username</th>
            <th class="pb-3 font-medium">Email</th>
            <th class="pb-3 font-medium">Role</th>
            <th class="pb-3 font-medium">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr class="table-row">
            <td class="py-3 text-gray-200">admin</td>
            <td class="py-3 text-gray-400">admin@example.com</td>
            <td class="py-3">
              <span class="badge badge-info">Admin</span>
            </td>
            <td class="py-3">
              <button class="text-sm text-primary-400 hover:text-primary-300">Edit</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>
