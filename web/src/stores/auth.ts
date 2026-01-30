import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { User } from '@/types'
import api from '@/api'

export const useAuthStore = defineStore('auth', () => {
  const user = ref<User | null>(null)
  const token = ref<string | null>(localStorage.getItem('token'))

  const isAuthenticated = computed(() => !!token.value)

  async function login(username: string, password: string) {
    try {
      const response = await api.post('/api/v1/auth/token', {
        username,
        password
      })
      token.value = response.data.access_token
      localStorage.setItem('token', token.value)
      await fetchUser()
      return true
    } catch {
      return false
    }
  }

  async function fetchUser() {
    if (!token.value) return
    try {
      const response = await api.get('/api/v1/auth/me')
      user.value = response.data
    } catch {
      logout()
    }
  }

  function logout() {
    user.value = null
    token.value = null
    localStorage.removeItem('token')
  }

  // Initialize user on store creation
  if (token.value) {
    fetchUser()
  }

  return {
    user,
    token,
    isAuthenticated,
    login,
    logout,
    fetchUser
  }
})
