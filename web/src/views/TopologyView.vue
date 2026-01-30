<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { Network } from 'vis-network'
import { DataSet } from 'vis-data'
import type { TopologyNode, TopologyEdge, DeviceType } from '@/types'
import api from '@/api'

const networkContainer = ref<HTMLElement | null>(null)
const loading = ref(true)
const selectedNode = ref<TopologyNode | null>(null)

let network: Network | null = null
const nodes = new DataSet<TopologyNode>([])
const edges = new DataSet<TopologyEdge>([])

const deviceTypeColors: Record<DeviceType, string> = {
  unknown: '#6b7280',
  workstation: '#3b82f6',
  server: '#8b5cf6',
  router: '#10b981',
  switch: '#14b8a6',
  firewall: '#ef4444',
  printer: '#f97316',
  camera: '#ec4899',
  iot: '#06b6d4',
  plc: '#eab308',
  hmi: '#f59e0b',
  scada: '#84cc16',
  mobile: '#6366f1',
  virtual: '#7c3aed'
}

const deviceTypeShapes: Record<DeviceType, string> = {
  unknown: 'dot',
  workstation: 'dot',
  server: 'square',
  router: 'triangle',
  switch: 'diamond',
  firewall: 'star',
  printer: 'dot',
  camera: 'dot',
  iot: 'dot',
  plc: 'hexagon',
  hmi: 'hexagon',
  scada: 'hexagon',
  mobile: 'dot',
  virtual: 'square'
}

async function fetchTopology() {
  loading.value = true
  try {
    const response = await api.get<{ nodes: TopologyNode[]; edges: TopologyEdge[] }>('/api/v1/topology')

    // Clear existing data
    nodes.clear()
    edges.clear()

    // Add new data with styling
    const styledNodes = response.data.nodes.map(node => ({
      ...node,
      color: deviceTypeColors[node.group] || deviceTypeColors.unknown,
      shape: deviceTypeShapes[node.group] || 'dot',
      size: node.group === 'router' || node.group === 'switch' ? 25 : 15
    }))

    nodes.add(styledNodes)
    edges.add(response.data.edges)
  } catch (error) {
    console.error('Failed to fetch topology:', error)
    // Add demo data for visualization
    addDemoData()
  } finally {
    loading.value = false
  }
}

function addDemoData() {
  const demoNodes: TopologyNode[] = [
    { id: '1', label: 'Router', group: 'router', title: 'Main Router\n192.168.1.1', shape: 'triangle' },
    { id: '2', label: 'Switch', group: 'switch', title: 'Core Switch\n192.168.1.2', shape: 'diamond' },
    { id: '3', label: 'Server 1', group: 'server', title: 'Web Server\n192.168.1.10', shape: 'square' },
    { id: '4', label: 'Server 2', group: 'server', title: 'DB Server\n192.168.1.11', shape: 'square' },
    { id: '5', label: 'WS-001', group: 'workstation', title: 'Workstation\n192.168.1.100', shape: 'dot' },
    { id: '6', label: 'WS-002', group: 'workstation', title: 'Workstation\n192.168.1.101', shape: 'dot' },
    { id: '7', label: 'PLC-01', group: 'plc', title: 'PLC Controller\n192.168.2.10', shape: 'hexagon' },
    { id: '8', label: 'HMI-01', group: 'hmi', title: 'HMI Panel\n192.168.2.20', shape: 'hexagon' }
  ]

  const styledNodes = demoNodes.map(node => ({
    ...node,
    color: deviceTypeColors[node.group] || deviceTypeColors.unknown,
    size: node.group === 'router' || node.group === 'switch' ? 30 : 20
  }))

  const demoEdges: TopologyEdge[] = [
    { id: 'e1', from: '1', to: '2', value: 1000, title: '1 Gbps' },
    { id: 'e2', from: '2', to: '3', value: 500, title: '500 Mbps' },
    { id: 'e3', from: '2', to: '4', value: 500, title: '500 Mbps' },
    { id: 'e4', from: '2', to: '5', value: 100, title: '100 Mbps' },
    { id: 'e5', from: '2', to: '6', value: 100, title: '100 Mbps' },
    { id: 'e6', from: '2', to: '7', value: 50, title: '50 Mbps' },
    { id: 'e7', from: '7', to: '8', value: 10, title: '10 Mbps' }
  ]

  nodes.add(styledNodes)
  edges.add(demoEdges)
}

function initNetwork() {
  if (!networkContainer.value) return

  const options = {
    nodes: {
      font: {
        color: '#e5e7eb',
        size: 12
      },
      borderWidth: 2,
      shadow: true
    },
    edges: {
      color: {
        color: '#4b5563',
        highlight: '#60a5fa',
        hover: '#60a5fa'
      },
      width: 2,
      smooth: {
        type: 'continuous'
      }
    },
    physics: {
      enabled: true,
      solver: 'forceAtlas2Based',
      forceAtlas2Based: {
        gravitationalConstant: -50,
        centralGravity: 0.01,
        springLength: 150,
        springConstant: 0.08
      },
      stabilization: {
        iterations: 100
      }
    },
    interaction: {
      hover: true,
      tooltipDelay: 200,
      zoomView: true,
      dragView: true
    }
  }

  network = new Network(
    networkContainer.value,
    { nodes, edges },
    options
  )

  network.on('click', (params) => {
    if (params.nodes.length > 0) {
      const nodeId = params.nodes[0]
      const node = nodes.get(nodeId)
      selectedNode.value = node
    } else {
      selectedNode.value = null
    }
  })
}

function fitNetwork() {
  network?.fit({ animation: true })
}

function togglePhysics() {
  if (network) {
    const options = network.getOptionsFromConfigurator()
    network.setOptions({
      physics: { enabled: !options.physics?.enabled }
    })
  }
}

onMounted(async () => {
  await fetchTopology()
  initNetwork()
})

onUnmounted(() => {
  network?.destroy()
})
</script>

<template>
  <div class="h-full flex flex-col">
    <!-- Toolbar -->
    <div class="flex items-center justify-between mb-4">
      <div class="flex items-center space-x-4">
        <button @click="fitNetwork" class="btn btn-secondary">
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
          </svg>
          Fit View
        </button>
        <button @click="togglePhysics" class="btn btn-secondary">
          Toggle Physics
        </button>
        <button @click="fetchTopology" class="btn btn-secondary">
          <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          Refresh
        </button>
      </div>

      <!-- Legend -->
      <div class="flex items-center space-x-4 text-sm">
        <span class="text-gray-400">Legend:</span>
        <div class="flex items-center space-x-1">
          <span class="w-3 h-3 rounded-full" :style="{ backgroundColor: deviceTypeColors.router }" />
          <span class="text-gray-300">Router</span>
        </div>
        <div class="flex items-center space-x-1">
          <span class="w-3 h-3 rounded" :style="{ backgroundColor: deviceTypeColors.switch }" />
          <span class="text-gray-300">Switch</span>
        </div>
        <div class="flex items-center space-x-1">
          <span class="w-3 h-3 rounded" :style="{ backgroundColor: deviceTypeColors.server }" />
          <span class="text-gray-300">Server</span>
        </div>
        <div class="flex items-center space-x-1">
          <span class="w-3 h-3 rounded-full" :style="{ backgroundColor: deviceTypeColors.workstation }" />
          <span class="text-gray-300">Workstation</span>
        </div>
        <div class="flex items-center space-x-1">
          <span class="w-3 h-3" :style="{ backgroundColor: deviceTypeColors.plc }" />
          <span class="text-gray-300">OT</span>
        </div>
      </div>
    </div>

    <!-- Network visualization -->
    <div class="flex-1 relative">
      <div
        ref="networkContainer"
        class="absolute inset-0 bg-dark-200 rounded-lg border border-gray-700"
      />

      <!-- Loading overlay -->
      <div
        v-if="loading"
        class="absolute inset-0 flex items-center justify-center bg-dark-200/80 rounded-lg"
      >
        <svg class="animate-spin h-8 w-8 text-primary-400" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
        </svg>
      </div>

      <!-- Node details panel -->
      <div
        v-if="selectedNode"
        class="absolute top-4 right-4 w-72 card"
      >
        <div class="flex items-center justify-between mb-3">
          <h3 class="font-semibold text-gray-100">{{ selectedNode.label }}</h3>
          <button
            @click="selectedNode = null"
            class="text-gray-400 hover:text-gray-200"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        <div class="space-y-2 text-sm">
          <div class="flex justify-between">
            <span class="text-gray-400">Type</span>
            <span class="text-gray-200 capitalize">{{ selectedNode.group }}</span>
          </div>
          <div v-if="selectedNode.title">
            <span class="text-gray-400">Details</span>
            <p class="text-gray-200 whitespace-pre-line mt-1">{{ selectedNode.title }}</p>
          </div>
        </div>
        <RouterLink
          :to="`/devices/${selectedNode.id}`"
          class="btn btn-primary w-full mt-4 text-center"
        >
          View Device
        </RouterLink>
      </div>
    </div>
  </div>
</template>
