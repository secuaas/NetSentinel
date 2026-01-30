<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useDevicesStore } from '@/stores/devices'
import type { DeviceType } from '@/types'

const devicesStore = useDevicesStore()

const searchQuery = ref('')
const selectedType = ref<DeviceType | ''>('')
const showActiveOnly = ref(false)

const deviceTypes: { label: string; value: DeviceType | '' }[] = [
  { label: 'All Types', value: '' },
  { label: 'Workstation', value: 'workstation' },
  { label: 'Server', value: 'server' },
  { label: 'Router', value: 'router' },
  { label: 'Switch', value: 'switch' },
  { label: 'Firewall', value: 'firewall' },
  { label: 'Printer', value: 'printer' },
  { label: 'Camera', value: 'camera' },
  { label: 'IoT', value: 'iot' },
  { label: 'PLC', value: 'plc' },
  { label: 'HMI', value: 'hmi' },
  { label: 'SCADA', value: 'scada' },
  { label: 'Mobile', value: 'mobile' },
  { label: 'Virtual', value: 'virtual' },
  { label: 'Unknown', value: 'unknown' }
]

function applyFilters() {
  devicesStore.setFilters({
    search: searchQuery.value || undefined,
    device_type: selectedType.value || undefined,
    is_active: showActiveOnly.value ? true : undefined
  })
}

function clearFilters() {
  searchQuery.value = ''
  selectedType.value = ''
  showActiveOnly.value = false
  devicesStore.clearFilters()
}

function getDeviceTypeColor(type: DeviceType): string {
  const colors: Record<DeviceType, string> = {
    unknown: 'bg-gray-900/50 text-gray-400',
    workstation: 'bg-blue-900/50 text-blue-400',
    server: 'bg-purple-900/50 text-purple-400',
    router: 'bg-green-900/50 text-green-400',
    switch: 'bg-teal-900/50 text-teal-400',
    firewall: 'bg-red-900/50 text-red-400',
    printer: 'bg-orange-900/50 text-orange-400',
    camera: 'bg-pink-900/50 text-pink-400',
    iot: 'bg-cyan-900/50 text-cyan-400',
    plc: 'bg-yellow-900/50 text-yellow-400',
    hmi: 'bg-amber-900/50 text-amber-400',
    scada: 'bg-lime-900/50 text-lime-400',
    mobile: 'bg-indigo-900/50 text-indigo-400',
    virtual: 'bg-violet-900/50 text-violet-400'
  }
  return colors[type] || colors.unknown
}

onMounted(() => {
  devicesStore.fetchDevices()
})
</script>

<template>
  <div class="space-y-6">
    <!-- Filters -->
    <div class="card">
      <div class="flex flex-wrap gap-4">
        <div class="flex-1 min-w-[200px]">
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search by IP, MAC, hostname..."
            class="input w-full"
            @keyup.enter="applyFilters"
          />
        </div>
        <select
          v-model="selectedType"
          class="input w-40"
          @change="applyFilters"
        >
          <option v-for="dt in deviceTypes" :key="dt.value" :value="dt.value">
            {{ dt.label }}
          </option>
        </select>
        <label class="flex items-center space-x-2 text-gray-300">
          <input
            v-model="showActiveOnly"
            type="checkbox"
            class="rounded bg-dark-200 border-gray-600 text-primary-500 focus:ring-primary-500"
            @change="applyFilters"
          />
          <span class="text-sm">Active only</span>
        </label>
        <button @click="applyFilters" class="btn btn-primary">
          Search
        </button>
        <button @click="clearFilters" class="btn btn-secondary">
          Clear
        </button>
      </div>
    </div>

    <!-- Devices table -->
    <div class="card overflow-hidden">
      <div class="overflow-x-auto">
        <table class="w-full">
          <thead>
            <tr class="text-left text-sm text-gray-400 border-b border-gray-700">
              <th class="pb-3 px-4 font-medium">Status</th>
              <th class="pb-3 px-4 font-medium">IP Address</th>
              <th class="pb-3 px-4 font-medium">MAC Address</th>
              <th class="pb-3 px-4 font-medium">Hostname</th>
              <th class="pb-3 px-4 font-medium">Type</th>
              <th class="pb-3 px-4 font-medium">Vendor</th>
              <th class="pb-3 px-4 font-medium">Last Seen</th>
              <th class="pb-3 px-4 font-medium">Risk</th>
            </tr>
          </thead>
          <tbody v-if="!devicesStore.loading && devicesStore.devices.length">
            <tr
              v-for="device in devicesStore.devices"
              :key="device.id"
              class="table-row"
            >
              <td class="py-3 px-4">
                <span
                  :class="[
                    'inline-block w-2 h-2 rounded-full',
                    device.is_active ? 'bg-green-400' : 'bg-gray-500'
                  ]"
                />
              </td>
              <td class="py-3 px-4">
                <RouterLink
                  :to="`/devices/${device.id}`"
                  class="font-mono text-sm text-primary-400 hover:text-primary-300"
                >
                  {{ device.ip_addresses[0] || '-' }}
                </RouterLink>
              </td>
              <td class="py-3 px-4 font-mono text-sm text-gray-300">
                {{ device.mac_address }}
              </td>
              <td class="py-3 px-4 text-sm text-gray-300">
                {{ device.hostname || '-' }}
              </td>
              <td class="py-3 px-4">
                <span :class="['badge', getDeviceTypeColor(device.device_type)]">
                  {{ device.device_type }}
                </span>
              </td>
              <td class="py-3 px-4 text-sm text-gray-400">
                {{ device.vendor || '-' }}
              </td>
              <td class="py-3 px-4 text-sm text-gray-400">
                {{ new Date(device.last_seen).toLocaleString() }}
              </td>
              <td class="py-3 px-4">
                <span
                  :class="[
                    'badge',
                    device.risk_score >= 80 ? 'badge-danger' :
                    device.risk_score >= 50 ? 'badge-warning' :
                    device.risk_score >= 20 ? 'badge-info' : 'badge-success'
                  ]"
                >
                  {{ device.risk_score }}
                </span>
              </td>
            </tr>
          </tbody>
        </table>

        <!-- Loading state -->
        <div v-if="devicesStore.loading" class="flex justify-center py-12">
          <svg class="animate-spin h-8 w-8 text-primary-400" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
        </div>

        <!-- Empty state -->
        <div v-if="!devicesStore.loading && !devicesStore.devices.length" class="text-center py-12">
          <svg class="mx-auto h-12 w-12 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2" />
          </svg>
          <p class="mt-4 text-gray-400">No devices found</p>
        </div>
      </div>

      <!-- Pagination -->
      <div v-if="devicesStore.totalPages > 1" class="flex items-center justify-between px-4 py-3 border-t border-gray-700">
        <div class="text-sm text-gray-400">
          Showing {{ (devicesStore.page - 1) * devicesStore.pageSize + 1 }} to
          {{ Math.min(devicesStore.page * devicesStore.pageSize, devicesStore.total) }}
          of {{ devicesStore.total }} devices
        </div>
        <div class="flex space-x-2">
          <button
            @click="devicesStore.setPage(devicesStore.page - 1)"
            :disabled="devicesStore.page === 1"
            class="btn btn-secondary text-sm disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Previous
          </button>
          <button
            @click="devicesStore.setPage(devicesStore.page + 1)"
            :disabled="devicesStore.page === devicesStore.totalPages"
            class="btn btn-secondary text-sm disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Next
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
