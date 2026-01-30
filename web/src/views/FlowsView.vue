<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useFlowsStore } from '@/stores/flows'

const route = useRoute()
const flowsStore = useFlowsStore()

const srcIp = ref('')
const dstIp = ref('')
const protocol = ref('')

const protocols = ['', 'TCP', 'UDP', 'ICMP', 'HTTP', 'HTTPS', 'DNS', 'SSH', 'MODBUS', 'DNP3', 'S7COMM']

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

function applyFilters() {
  flowsStore.setFilters({
    src_ip: srcIp.value || undefined,
    dst_ip: dstIp.value || undefined,
    protocol: protocol.value || undefined
  })
}

function clearFilters() {
  srcIp.value = ''
  dstIp.value = ''
  protocol.value = ''
  flowsStore.setFilters({})
}

onMounted(() => {
  // Check for query params
  if (route.query.src_ip) {
    srcIp.value = route.query.src_ip as string
  }
  if (route.query.dst_ip) {
    dstIp.value = route.query.dst_ip as string
  }

  if (srcIp.value || dstIp.value) {
    applyFilters()
  } else {
    flowsStore.fetchFlows()
  }
})
</script>

<template>
  <div class="space-y-6">
    <!-- Filters -->
    <div class="card">
      <div class="flex flex-wrap gap-4">
        <div class="flex-1 min-w-[150px]">
          <label class="block text-sm text-gray-400 mb-1">Source IP</label>
          <input
            v-model="srcIp"
            type="text"
            placeholder="e.g., 192.168.1.100"
            class="input w-full"
            @keyup.enter="applyFilters"
          />
        </div>
        <div class="flex-1 min-w-[150px]">
          <label class="block text-sm text-gray-400 mb-1">Destination IP</label>
          <input
            v-model="dstIp"
            type="text"
            placeholder="e.g., 192.168.1.1"
            class="input w-full"
            @keyup.enter="applyFilters"
          />
        </div>
        <div class="w-40">
          <label class="block text-sm text-gray-400 mb-1">Protocol</label>
          <select v-model="protocol" class="input w-full">
            <option value="">All</option>
            <option v-for="p in protocols.slice(1)" :key="p" :value="p">{{ p }}</option>
          </select>
        </div>
        <div class="flex items-end space-x-2">
          <button @click="applyFilters" class="btn btn-primary">
            Filter
          </button>
          <button @click="clearFilters" class="btn btn-secondary">
            Clear
          </button>
        </div>
      </div>
    </div>

    <!-- Flows table -->
    <div class="card overflow-hidden">
      <div class="overflow-x-auto">
        <table class="w-full">
          <thead>
            <tr class="text-left text-sm text-gray-400 border-b border-gray-700">
              <th class="pb-3 px-4 font-medium">Source</th>
              <th class="pb-3 px-4 font-medium">Destination</th>
              <th class="pb-3 px-4 font-medium">Protocol</th>
              <th class="pb-3 px-4 font-medium text-right">Packets</th>
              <th class="pb-3 px-4 font-medium text-right">Bytes</th>
              <th class="pb-3 px-4 font-medium">Application</th>
              <th class="pb-3 px-4 font-medium">Last Seen</th>
            </tr>
          </thead>
          <tbody v-if="!flowsStore.loading && flowsStore.flows.length">
            <tr
              v-for="flow in flowsStore.flows"
              :key="flow.id"
              class="table-row"
            >
              <td class="py-3 px-4">
                <div class="font-mono text-sm text-gray-300">{{ flow.src_ip }}</div>
                <div class="text-xs text-gray-500">:{{ flow.src_port }}</div>
              </td>
              <td class="py-3 px-4">
                <div class="font-mono text-sm text-gray-300">{{ flow.dst_ip }}</div>
                <div class="text-xs text-gray-500">:{{ flow.dst_port }}</div>
              </td>
              <td class="py-3 px-4">
                <span class="badge badge-info">{{ flow.protocol }}</span>
              </td>
              <td class="py-3 px-4 text-sm text-gray-300 text-right font-mono">
                {{ flow.packets.toLocaleString() }}
              </td>
              <td class="py-3 px-4 text-sm text-gray-300 text-right">
                {{ formatBytes(flow.bytes) }}
              </td>
              <td class="py-3 px-4 text-sm text-gray-400">
                {{ flow.application || '-' }}
              </td>
              <td class="py-3 px-4 text-sm text-gray-400">
                {{ new Date(flow.last_seen).toLocaleString() }}
              </td>
            </tr>
          </tbody>
        </table>

        <!-- Loading state -->
        <div v-if="flowsStore.loading" class="flex justify-center py-12">
          <svg class="animate-spin h-8 w-8 text-primary-400" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
        </div>

        <!-- Empty state -->
        <div v-if="!flowsStore.loading && !flowsStore.flows.length" class="text-center py-12">
          <svg class="mx-auto h-12 w-12 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16l-4-4m0 0l4-4m-4 4h18m-4 4l4-4m0 0l-4-4" />
          </svg>
          <p class="mt-4 text-gray-400">No flows found</p>
        </div>
      </div>

      <!-- Pagination -->
      <div v-if="flowsStore.totalPages > 1" class="flex items-center justify-between px-4 py-3 border-t border-gray-700">
        <div class="text-sm text-gray-400">
          Showing {{ (flowsStore.page - 1) * flowsStore.pageSize + 1 }} to
          {{ Math.min(flowsStore.page * flowsStore.pageSize, flowsStore.total) }}
          of {{ flowsStore.total }} flows
        </div>
        <div class="flex space-x-2">
          <button
            @click="flowsStore.setPage(flowsStore.page - 1)"
            :disabled="flowsStore.page === 1"
            class="btn btn-secondary text-sm disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Previous
          </button>
          <button
            @click="flowsStore.setPage(flowsStore.page + 1)"
            :disabled="flowsStore.page === flowsStore.totalPages"
            class="btn btn-secondary text-sm disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Next
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
