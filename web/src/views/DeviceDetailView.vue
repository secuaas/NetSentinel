<script setup lang="ts">
import { onMounted, ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useDevicesStore } from '@/stores/devices'
import type { DeviceType } from '@/types'

const route = useRoute()
const router = useRouter()
const devicesStore = useDevicesStore()

const isEditing = ref(false)
const editForm = ref({
  hostname: '',
  device_type: '' as DeviceType,
  notes: '',
  tags: [] as string[],
  network_zone: ''
})
const newTag = ref('')

const device = computed(() => devicesStore.currentDevice)

const deviceTypes: DeviceType[] = [
  'unknown', 'workstation', 'server', 'router', 'switch', 'firewall',
  'printer', 'camera', 'iot', 'plc', 'hmi', 'scada', 'mobile', 'virtual'
]

function startEdit() {
  if (device.value) {
    editForm.value = {
      hostname: device.value.hostname || '',
      device_type: device.value.device_type,
      notes: device.value.notes || '',
      tags: [...device.value.tags],
      network_zone: device.value.network_zone || ''
    }
    isEditing.value = true
  }
}

function cancelEdit() {
  isEditing.value = false
}

async function saveEdit() {
  if (device.value) {
    const success = await devicesStore.updateDevice(device.value.id, {
      hostname: editForm.value.hostname || null,
      device_type: editForm.value.device_type,
      notes: editForm.value.notes || null,
      tags: editForm.value.tags,
      network_zone: editForm.value.network_zone || null
    })
    if (success) {
      isEditing.value = false
    }
  }
}

function addTag() {
  if (newTag.value && !editForm.value.tags.includes(newTag.value)) {
    editForm.value.tags.push(newTag.value)
    newTag.value = ''
  }
}

function removeTag(tag: string) {
  editForm.value.tags = editForm.value.tags.filter(t => t !== tag)
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

onMounted(() => {
  const id = route.params.id as string
  devicesStore.fetchDevice(id)
})
</script>

<template>
  <div class="space-y-6">
    <!-- Back button -->
    <button
      @click="router.back()"
      class="flex items-center text-gray-400 hover:text-gray-200"
    >
      <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
      </svg>
      Back to Devices
    </button>

    <div v-if="devicesStore.loading" class="flex justify-center py-12">
      <svg class="animate-spin h-8 w-8 text-primary-400" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
      </svg>
    </div>

    <div v-else-if="device" class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <!-- Main info -->
      <div class="lg:col-span-2 space-y-6">
        <div class="card">
          <div class="flex items-start justify-between mb-4">
            <div>
              <div class="flex items-center space-x-3">
                <span
                  :class="[
                    'w-3 h-3 rounded-full',
                    device.is_active ? 'bg-green-400' : 'bg-gray-500'
                  ]"
                />
                <h2 class="text-xl font-bold text-gray-100">
                  {{ device.hostname || device.ip_addresses[0] || device.mac_address }}
                </h2>
              </div>
              <p class="mt-1 text-sm text-gray-400">{{ device.vendor || 'Unknown vendor' }}</p>
            </div>
            <button
              v-if="!isEditing"
              @click="startEdit"
              class="btn btn-secondary"
            >
              Edit
            </button>
            <div v-else class="flex space-x-2">
              <button @click="saveEdit" class="btn btn-primary">Save</button>
              <button @click="cancelEdit" class="btn btn-secondary">Cancel</button>
            </div>
          </div>

          <!-- View mode -->
          <div v-if="!isEditing" class="grid grid-cols-2 gap-4">
            <div>
              <p class="text-sm text-gray-400">MAC Address</p>
              <p class="font-mono text-gray-200">{{ device.mac_address }}</p>
            </div>
            <div>
              <p class="text-sm text-gray-400">IP Addresses</p>
              <p class="font-mono text-gray-200">{{ device.ip_addresses.join(', ') || '-' }}</p>
            </div>
            <div>
              <p class="text-sm text-gray-400">Device Type</p>
              <p class="text-gray-200 capitalize">{{ device.device_type }}</p>
            </div>
            <div>
              <p class="text-sm text-gray-400">Network Zone</p>
              <p class="text-gray-200">{{ device.network_zone || '-' }}</p>
            </div>
            <div>
              <p class="text-sm text-gray-400">First Seen</p>
              <p class="text-gray-200">{{ new Date(device.first_seen).toLocaleString() }}</p>
            </div>
            <div>
              <p class="text-sm text-gray-400">Last Seen</p>
              <p class="text-gray-200">{{ new Date(device.last_seen).toLocaleString() }}</p>
            </div>
            <div>
              <p class="text-sm text-gray-400">OS Fingerprint</p>
              <p class="text-gray-200">{{ device.os_fingerprint || '-' }}</p>
            </div>
            <div>
              <p class="text-sm text-gray-400">Risk Score</p>
              <p class="text-gray-200">{{ device.risk_score }}</p>
            </div>
          </div>

          <!-- Edit mode -->
          <div v-else class="space-y-4">
            <div>
              <label class="block text-sm text-gray-400 mb-1">Hostname</label>
              <input v-model="editForm.hostname" type="text" class="input w-full" />
            </div>
            <div>
              <label class="block text-sm text-gray-400 mb-1">Device Type</label>
              <select v-model="editForm.device_type" class="input w-full">
                <option v-for="dt in deviceTypes" :key="dt" :value="dt" class="capitalize">
                  {{ dt }}
                </option>
              </select>
            </div>
            <div>
              <label class="block text-sm text-gray-400 mb-1">Network Zone</label>
              <input v-model="editForm.network_zone" type="text" class="input w-full" />
            </div>
            <div>
              <label class="block text-sm text-gray-400 mb-1">Notes</label>
              <textarea v-model="editForm.notes" rows="3" class="input w-full" />
            </div>
            <div>
              <label class="block text-sm text-gray-400 mb-1">Tags</label>
              <div class="flex flex-wrap gap-2 mb-2">
                <span
                  v-for="tag in editForm.tags"
                  :key="tag"
                  class="badge badge-info flex items-center"
                >
                  {{ tag }}
                  <button @click="removeTag(tag)" class="ml-1 hover:text-red-400">
                    &times;
                  </button>
                </span>
              </div>
              <div class="flex space-x-2">
                <input
                  v-model="newTag"
                  type="text"
                  class="input flex-1"
                  placeholder="Add tag..."
                  @keyup.enter="addTag"
                />
                <button @click="addTag" class="btn btn-secondary">Add</button>
              </div>
            </div>
          </div>
        </div>

        <!-- Open ports -->
        <div class="card">
          <h3 class="text-lg font-semibold text-gray-100 mb-4">Open Ports</h3>
          <div v-if="device.open_ports.length" class="flex flex-wrap gap-2">
            <span
              v-for="port in device.open_ports"
              :key="port"
              class="px-3 py-1 bg-dark-200 rounded-lg text-sm font-mono text-gray-300"
            >
              {{ port }}
            </span>
          </div>
          <p v-else class="text-gray-500">No open ports detected</p>
        </div>

        <!-- Protocols -->
        <div class="card">
          <h3 class="text-lg font-semibold text-gray-100 mb-4">Protocols</h3>
          <div v-if="device.protocols.length" class="flex flex-wrap gap-2">
            <span
              v-for="proto in device.protocols"
              :key="proto"
              class="badge badge-info"
            >
              {{ proto }}
            </span>
          </div>
          <p v-else class="text-gray-500">No protocols detected</p>
        </div>

        <!-- Notes (view mode) -->
        <div v-if="!isEditing && device.notes" class="card">
          <h3 class="text-lg font-semibold text-gray-100 mb-4">Notes</h3>
          <p class="text-gray-300 whitespace-pre-wrap">{{ device.notes }}</p>
        </div>
      </div>

      <!-- Sidebar -->
      <div class="space-y-6">
        <!-- Tags -->
        <div class="card">
          <h3 class="text-lg font-semibold text-gray-100 mb-4">Tags</h3>
          <div v-if="device.tags.length" class="flex flex-wrap gap-2">
            <span
              v-for="tag in device.tags"
              :key="tag"
              class="badge badge-info"
            >
              {{ tag }}
            </span>
          </div>
          <p v-else class="text-gray-500">No tags</p>
        </div>

        <!-- Quick actions -->
        <div class="card">
          <h3 class="text-lg font-semibold text-gray-100 mb-4">Actions</h3>
          <div class="space-y-2">
            <RouterLink
              :to="`/flows?src_ip=${device.ip_addresses[0]}`"
              class="btn btn-secondary w-full text-center"
            >
              View Flows
            </RouterLink>
            <RouterLink
              :to="`/topology?focus=${device.id}`"
              class="btn btn-secondary w-full text-center"
            >
              Show in Topology
            </RouterLink>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="text-center py-12">
      <p class="text-gray-400">Device not found</p>
    </div>
  </div>
</template>
