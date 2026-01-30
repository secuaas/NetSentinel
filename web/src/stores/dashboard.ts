import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { DashboardStats, TimeRange } from '@/types'
import api from '@/api'

export const useDashboardStore = defineStore('dashboard', () => {
  const stats = ref<DashboardStats | null>(null)
  const loading = ref(false)
  const timeRange = ref<TimeRange>('24h')
  const lastUpdate = ref<Date | null>(null)

  async function fetchStats() {
    loading.value = true
    try {
      const response = await api.get<DashboardStats>(
        `/api/v1/stats/dashboard?time_range=${timeRange.value}`
      )
      stats.value = response.data
      lastUpdate.value = new Date()
    } catch (error) {
      console.error('Failed to fetch dashboard stats:', error)
    } finally {
      loading.value = false
    }
  }

  function setTimeRange(range: TimeRange) {
    timeRange.value = range
    fetchStats()
  }

  return {
    stats,
    loading,
    timeRange,
    lastUpdate,
    fetchStats,
    setTimeRange
  }
})
