import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Alert, PaginatedResponse } from '@/types'
import api from '@/api'

export const useAlertsStore = defineStore('alerts', () => {
  const alerts = ref<Alert[]>([])
  const total = ref(0)
  const page = ref(1)
  const pageSize = ref(20)
  const loading = ref(false)
  const showAcknowledged = ref(false)

  const totalPages = computed(() => Math.ceil(total.value / pageSize.value))

  const unacknowledgedCount = computed(
    () => alerts.value.filter((a) => !a.acknowledged).length
  )

  async function fetchAlerts() {
    loading.value = true
    try {
      const params = new URLSearchParams()
      params.append('page', page.value.toString())
      params.append('page_size', pageSize.value.toString())
      if (!showAcknowledged.value) {
        params.append('acknowledged', 'false')
      }

      const response = await api.get<PaginatedResponse<Alert>>(
        `/api/v1/alerts?${params.toString()}`
      )
      alerts.value = response.data.items
      total.value = response.data.total
    } catch (error) {
      console.error('Failed to fetch alerts:', error)
    } finally {
      loading.value = false
    }
  }

  async function acknowledgeAlert(id: string) {
    try {
      await api.post(`/api/v1/alerts/${id}/acknowledge`)
      const alert = alerts.value.find((a) => a.id === id)
      if (alert) {
        alert.acknowledged = true
        alert.acknowledged_at = new Date().toISOString()
      }
      return true
    } catch (error) {
      console.error('Failed to acknowledge alert:', error)
      return false
    }
  }

  async function acknowledgeAll() {
    try {
      await api.post('/api/v1/alerts/acknowledge-all')
      alerts.value.forEach((a) => {
        a.acknowledged = true
        a.acknowledged_at = new Date().toISOString()
      })
      return true
    } catch (error) {
      console.error('Failed to acknowledge all alerts:', error)
      return false
    }
  }

  function setPage(newPage: number) {
    page.value = newPage
    fetchAlerts()
  }

  function toggleShowAcknowledged() {
    showAcknowledged.value = !showAcknowledged.value
    page.value = 1
    fetchAlerts()
  }

  return {
    alerts,
    total,
    page,
    pageSize,
    totalPages,
    loading,
    showAcknowledged,
    unacknowledgedCount,
    fetchAlerts,
    acknowledgeAlert,
    acknowledgeAll,
    setPage,
    toggleShowAcknowledged
  }
})
