<script setup lang="ts">
import { onMounted } from 'vue'
import { useAlertsStore } from '@/stores/alerts'
import type { AlertSeverity } from '@/types'

const alertsStore = useAlertsStore()

function getSeverityClass(severity: AlertSeverity): string {
  const classes: Record<AlertSeverity, string> = {
    critical: 'badge-danger',
    high: 'bg-orange-900/50 text-orange-400',
    medium: 'badge-warning',
    low: 'badge-info'
  }
  return classes[severity]
}

function getSeverityIcon(severity: AlertSeverity): string {
  if (severity === 'critical' || severity === 'high') {
    return 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z'
  }
  return 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z'
}

onMounted(() => {
  alertsStore.fetchAlerts()
})
</script>

<template>
  <div class="space-y-6">
    <!-- Header -->
    <div class="flex items-center justify-between">
      <div class="flex items-center space-x-4">
        <label class="flex items-center space-x-2 text-gray-300">
          <input
            type="checkbox"
            :checked="alertsStore.showAcknowledged"
            @change="alertsStore.toggleShowAcknowledged"
            class="rounded bg-dark-200 border-gray-600 text-primary-500 focus:ring-primary-500"
          />
          <span class="text-sm">Show acknowledged</span>
        </label>
      </div>
      <button
        v-if="alertsStore.unacknowledgedCount > 0"
        @click="alertsStore.acknowledgeAll"
        class="btn btn-secondary"
      >
        Acknowledge All
      </button>
    </div>

    <!-- Alerts list -->
    <div class="space-y-4">
      <div v-if="alertsStore.loading" class="flex justify-center py-12">
        <svg class="animate-spin h-8 w-8 text-primary-400" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
        </svg>
      </div>

      <div
        v-else-if="alertsStore.alerts.length"
        v-for="alert in alertsStore.alerts"
        :key="alert.id"
        :class="[
          'card',
          alert.acknowledged ? 'opacity-60' : ''
        ]"
      >
        <div class="flex items-start">
          <div
            :class="[
              'p-3 rounded-full mr-4',
              alert.severity === 'critical' ? 'bg-red-900/50 text-red-400' :
              alert.severity === 'high' ? 'bg-orange-900/50 text-orange-400' :
              alert.severity === 'medium' ? 'bg-yellow-900/50 text-yellow-400' : 'bg-blue-900/50 text-blue-400'
            ]"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" :d="getSeverityIcon(alert.severity)" />
            </svg>
          </div>

          <div class="flex-1 min-w-0">
            <div class="flex items-center space-x-2 mb-1">
              <h3 class="text-lg font-semibold text-gray-100">{{ alert.title }}</h3>
              <span :class="['badge', getSeverityClass(alert.severity)]">
                {{ alert.severity }}
              </span>
              <span v-if="alert.acknowledged" class="badge badge-success">
                Acknowledged
              </span>
            </div>

            <p class="text-gray-400 mb-2">{{ alert.description }}</p>

            <div class="flex items-center space-x-4 text-sm text-gray-500">
              <span>{{ new Date(alert.created_at).toLocaleString() }}</span>
              <span v-if="alert.device_id">
                Device:
                <RouterLink
                  :to="`/devices/${alert.device_id}`"
                  class="text-primary-400 hover:text-primary-300"
                >
                  View
                </RouterLink>
              </span>
              <span v-if="alert.acknowledged_by">
                Acknowledged by {{ alert.acknowledged_by }}
              </span>
            </div>
          </div>

          <button
            v-if="!alert.acknowledged"
            @click="alertsStore.acknowledgeAlert(alert.id)"
            class="btn btn-secondary text-sm ml-4"
          >
            Acknowledge
          </button>
        </div>
      </div>

      <div v-else class="text-center py-12">
        <svg class="mx-auto h-12 w-12 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p class="mt-4 text-gray-400">No alerts</p>
      </div>
    </div>

    <!-- Pagination -->
    <div v-if="alertsStore.totalPages > 1" class="flex items-center justify-between">
      <div class="text-sm text-gray-400">
        Page {{ alertsStore.page }} of {{ alertsStore.totalPages }}
      </div>
      <div class="flex space-x-2">
        <button
          @click="alertsStore.setPage(alertsStore.page - 1)"
          :disabled="alertsStore.page === 1"
          class="btn btn-secondary text-sm disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Previous
        </button>
        <button
          @click="alertsStore.setPage(alertsStore.page + 1)"
          :disabled="alertsStore.page === alertsStore.totalPages"
          class="btn btn-secondary text-sm disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Next
        </button>
      </div>
    </div>
  </div>
</template>
