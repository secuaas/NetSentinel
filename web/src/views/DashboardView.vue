<script setup lang="ts">
import { onMounted, computed } from 'vue'
import { useDashboardStore } from '@/stores/dashboard'
import type { TimeRange } from '@/types'

const dashboardStore = useDashboardStore()

const timeRanges: { label: string; value: TimeRange }[] = [
  { label: '1h', value: '1h' },
  { label: '6h', value: '6h' },
  { label: '24h', value: '24h' },
  { label: '7d', value: '7d' },
  { label: '30d', value: '30d' }
]

const stats = computed(() => dashboardStore.stats)

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

onMounted(() => {
  dashboardStore.fetchStats()
})
</script>

<template>
  <div class="space-y-6">
    <!-- Time range selector -->
    <div class="flex justify-end">
      <div class="inline-flex rounded-lg bg-dark-100 p-1">
        <button
          v-for="range in timeRanges"
          :key="range.value"
          @click="dashboardStore.setTimeRange(range.value)"
          :class="[
            'px-3 py-1.5 text-sm font-medium rounded-md transition-colors',
            dashboardStore.timeRange === range.value
              ? 'bg-primary-600 text-white'
              : 'text-gray-400 hover:text-gray-200'
          ]"
        >
          {{ range.label }}
        </button>
      </div>
    </div>

    <!-- Stats cards -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-400">Total Devices</p>
            <p class="text-2xl font-bold text-gray-100">
              {{ stats?.total_devices ?? '-' }}
            </p>
          </div>
          <div class="p-3 rounded-full bg-primary-900/50">
            <svg class="w-6 h-6 text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
            </svg>
          </div>
        </div>
        <p class="mt-2 text-sm text-green-400">
          {{ stats?.active_devices ?? 0 }} active
        </p>
      </div>

      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-400">New Devices (24h)</p>
            <p class="text-2xl font-bold text-gray-100">
              {{ stats?.new_devices_24h ?? '-' }}
            </p>
          </div>
          <div class="p-3 rounded-full bg-green-900/50">
            <svg class="w-6 h-6 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
            </svg>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-400">Total Flows</p>
            <p class="text-2xl font-bold text-gray-100">
              {{ stats?.total_flows?.toLocaleString() ?? '-' }}
            </p>
          </div>
          <div class="p-3 rounded-full bg-blue-900/50">
            <svg class="w-6 h-6 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16l-4-4m0 0l4-4m-4 4h18m-4 4l4-4m0 0l-4-4" />
            </svg>
          </div>
        </div>
      </div>

      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-400">Total Traffic</p>
            <p class="text-2xl font-bold text-gray-100">
              {{ formatBytes(stats?.total_bytes ?? 0) }}
            </p>
          </div>
          <div class="p-3 rounded-full bg-purple-900/50">
            <svg class="w-6 h-6 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
            </svg>
          </div>
        </div>
      </div>
    </div>

    <!-- Charts row -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- Protocol distribution -->
      <div class="card">
        <h3 class="text-lg font-semibold text-gray-100 mb-4">Protocol Distribution</h3>
        <div v-if="stats?.protocols?.length" class="space-y-3">
          <div
            v-for="protocol in stats.protocols.slice(0, 8)"
            :key="protocol.protocol"
            class="flex items-center"
          >
            <span class="w-16 text-sm text-gray-400">{{ protocol.protocol }}</span>
            <div class="flex-1 mx-3 h-4 bg-dark-200 rounded-full overflow-hidden">
              <div
                class="h-full bg-primary-500 rounded-full"
                :style="{ width: `${protocol.percentage}%` }"
              />
            </div>
            <span class="w-12 text-sm text-gray-400 text-right">
              {{ protocol.percentage.toFixed(1) }}%
            </span>
          </div>
        </div>
        <p v-else class="text-gray-500 text-center py-8">No data available</p>
      </div>

      <!-- Device types -->
      <div class="card">
        <h3 class="text-lg font-semibold text-gray-100 mb-4">Device Types</h3>
        <div v-if="stats?.device_types?.length" class="grid grid-cols-2 gap-3">
          <div
            v-for="dt in stats.device_types"
            :key="dt.type"
            class="flex items-center justify-between p-3 bg-dark-200 rounded-lg"
          >
            <span class="text-sm text-gray-300 capitalize">{{ dt.type }}</span>
            <span class="text-lg font-semibold text-gray-100">{{ dt.count }}</span>
          </div>
        </div>
        <p v-else class="text-gray-500 text-center py-8">No data available</p>
      </div>
    </div>

    <!-- Top talkers -->
    <div class="card">
      <h3 class="text-lg font-semibold text-gray-100 mb-4">Top Talkers</h3>
      <div v-if="stats?.top_talkers?.length" class="overflow-x-auto">
        <table class="w-full">
          <thead>
            <tr class="text-left text-sm text-gray-400 border-b border-gray-700">
              <th class="pb-3 font-medium">IP Address</th>
              <th class="pb-3 font-medium">Hostname</th>
              <th class="pb-3 font-medium text-right">Sent</th>
              <th class="pb-3 font-medium text-right">Received</th>
              <th class="pb-3 font-medium text-right">Total</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="talker in stats.top_talkers"
              :key="talker.device_id"
              class="table-row"
            >
              <td class="py-3 font-mono text-sm text-gray-300">
                {{ talker.ip_address }}
              </td>
              <td class="py-3 text-sm text-gray-400">
                {{ talker.hostname || '-' }}
              </td>
              <td class="py-3 text-sm text-gray-300 text-right">
                {{ formatBytes(talker.bytes_sent) }}
              </td>
              <td class="py-3 text-sm text-gray-300 text-right">
                {{ formatBytes(talker.bytes_received) }}
              </td>
              <td class="py-3 text-sm font-semibold text-gray-100 text-right">
                {{ formatBytes(talker.total_bytes) }}
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <p v-else class="text-gray-500 text-center py-8">No data available</p>
    </div>

    <!-- Recent alerts -->
    <div class="card">
      <div class="flex items-center justify-between mb-4">
        <h3 class="text-lg font-semibold text-gray-100">Recent Alerts</h3>
        <RouterLink to="/alerts" class="text-sm text-primary-400 hover:text-primary-300">
          View all
        </RouterLink>
      </div>
      <div v-if="stats?.alerts?.length" class="space-y-3">
        <div
          v-for="alert in stats.alerts.slice(0, 5)"
          :key="alert.id"
          class="flex items-start p-3 bg-dark-200 rounded-lg"
        >
          <div
            :class="[
              'p-2 rounded-full mr-3',
              alert.severity === 'critical' ? 'bg-red-900/50' :
              alert.severity === 'high' ? 'bg-orange-900/50' :
              alert.severity === 'medium' ? 'bg-yellow-900/50' : 'bg-blue-900/50'
            ]"
          >
            <svg class="w-4 h-4 text-current" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-gray-200">{{ alert.title }}</p>
            <p class="text-xs text-gray-400 truncate">{{ alert.description }}</p>
          </div>
          <span
            :class="[
              'badge ml-2',
              alert.severity === 'critical' ? 'badge-danger' :
              alert.severity === 'high' ? 'bg-orange-900/50 text-orange-400' :
              alert.severity === 'medium' ? 'badge-warning' : 'badge-info'
            ]"
          >
            {{ alert.severity }}
          </span>
        </div>
      </div>
      <p v-else class="text-gray-500 text-center py-8">No alerts</p>
    </div>
  </div>
</template>
