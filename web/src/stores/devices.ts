import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Device, DeviceFilters, PaginatedResponse } from '@/types'
import api from '@/api'

export const useDevicesStore = defineStore('devices', () => {
  const devices = ref<Device[]>([])
  const currentDevice = ref<Device | null>(null)
  const total = ref(0)
  const page = ref(1)
  const pageSize = ref(20)
  const loading = ref(false)
  const filters = ref<DeviceFilters>({})

  const totalPages = computed(() => Math.ceil(total.value / pageSize.value))

  async function fetchDevices() {
    loading.value = true
    try {
      const params = new URLSearchParams()
      params.append('page', page.value.toString())
      params.append('page_size', pageSize.value.toString())

      if (filters.value.search) {
        params.append('search', filters.value.search)
      }
      if (filters.value.device_type) {
        params.append('device_type', filters.value.device_type)
      }
      if (filters.value.is_active !== undefined) {
        params.append('is_active', filters.value.is_active.toString())
      }
      if (filters.value.network_zone) {
        params.append('network_zone', filters.value.network_zone)
      }

      const response = await api.get<PaginatedResponse<Device>>(
        `/api/v1/devices?${params.toString()}`
      )
      devices.value = response.data.items
      total.value = response.data.total
    } catch (error) {
      console.error('Failed to fetch devices:', error)
    } finally {
      loading.value = false
    }
  }

  async function fetchDevice(id: string) {
    loading.value = true
    try {
      const response = await api.get<Device>(`/api/v1/devices/${id}`)
      currentDevice.value = response.data
    } catch (error) {
      console.error('Failed to fetch device:', error)
    } finally {
      loading.value = false
    }
  }

  async function updateDevice(id: string, data: Partial<Device>) {
    try {
      const response = await api.patch<Device>(`/api/v1/devices/${id}`, data)
      currentDevice.value = response.data
      // Update in list if present
      const index = devices.value.findIndex((d) => d.id === id)
      if (index !== -1) {
        devices.value[index] = response.data
      }
      return true
    } catch (error) {
      console.error('Failed to update device:', error)
      return false
    }
  }

  function setPage(newPage: number) {
    page.value = newPage
    fetchDevices()
  }

  function setFilters(newFilters: DeviceFilters) {
    filters.value = newFilters
    page.value = 1
    fetchDevices()
  }

  function clearFilters() {
    filters.value = {}
    page.value = 1
    fetchDevices()
  }

  return {
    devices,
    currentDevice,
    total,
    page,
    pageSize,
    totalPages,
    loading,
    filters,
    fetchDevices,
    fetchDevice,
    updateDevice,
    setPage,
    setFilters,
    clearFilters
  }
})
