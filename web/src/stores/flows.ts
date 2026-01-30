import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Flow, FlowFilters, PaginatedResponse } from '@/types'
import api from '@/api'

export const useFlowsStore = defineStore('flows', () => {
  const flows = ref<Flow[]>([])
  const total = ref(0)
  const page = ref(1)
  const pageSize = ref(50)
  const loading = ref(false)
  const filters = ref<FlowFilters>({})

  const totalPages = computed(() => Math.ceil(total.value / pageSize.value))

  async function fetchFlows() {
    loading.value = true
    try {
      const params = new URLSearchParams()
      params.append('page', page.value.toString())
      params.append('page_size', pageSize.value.toString())

      if (filters.value.src_ip) {
        params.append('src_ip', filters.value.src_ip)
      }
      if (filters.value.dst_ip) {
        params.append('dst_ip', filters.value.dst_ip)
      }
      if (filters.value.protocol) {
        params.append('protocol', filters.value.protocol)
      }

      const response = await api.get<PaginatedResponse<Flow>>(
        `/api/v1/flows?${params.toString()}`
      )
      flows.value = response.data.items
      total.value = response.data.total
    } catch (error) {
      console.error('Failed to fetch flows:', error)
    } finally {
      loading.value = false
    }
  }

  function setPage(newPage: number) {
    page.value = newPage
    fetchFlows()
  }

  function setFilters(newFilters: FlowFilters) {
    filters.value = newFilters
    page.value = 1
    fetchFlows()
  }

  return {
    flows,
    total,
    page,
    pageSize,
    totalPages,
    loading,
    filters,
    fetchFlows,
    setPage,
    setFilters
  }
})
