<script setup lang="ts">
import { ref } from 'vue'
import { RouterView, RouterLink, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useAlertsStore } from '@/stores/alerts'

const authStore = useAuthStore()
const alertsStore = useAlertsStore()
const route = useRoute()
const sidebarOpen = ref(true)

const navigation = [
  { name: 'Dashboard', path: '/', icon: 'chart-bar' },
  { name: 'Devices', path: '/devices', icon: 'server' },
  { name: 'Topology', path: '/topology', icon: 'share' },
  { name: 'Flows', path: '/flows', icon: 'arrows-right-left' },
  { name: 'Alerts', path: '/alerts', icon: 'bell' },
  { name: 'Settings', path: '/settings', icon: 'cog' }
]

function isActive(path: string) {
  if (path === '/') return route.path === '/'
  return route.path.startsWith(path)
}

function handleLogout() {
  authStore.logout()
}
</script>

<template>
  <div class="flex h-screen bg-dark-300">
    <!-- Sidebar -->
    <aside
      :class="[
        'flex flex-col bg-dark-200 border-r border-gray-700 transition-all duration-300',
        sidebarOpen ? 'w-64' : 'w-16'
      ]"
    >
      <!-- Logo -->
      <div class="flex items-center h-16 px-4 border-b border-gray-700">
        <img src="/favicon.svg" alt="NetSentinel" class="w-8 h-8" />
        <span v-if="sidebarOpen" class="ml-3 text-xl font-bold text-primary-400">
          NetSentinel
        </span>
      </div>

      <!-- Navigation -->
      <nav class="flex-1 py-4 space-y-1 overflow-y-auto">
        <RouterLink
          v-for="item in navigation"
          :key="item.path"
          :to="item.path"
          :class="[
            'flex items-center px-4 py-3 text-sm font-medium transition-colors',
            isActive(item.path)
              ? 'bg-primary-900/50 text-primary-400 border-r-2 border-primary-400'
              : 'text-gray-400 hover:bg-dark-100 hover:text-gray-200'
          ]"
        >
          <svg
            class="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <!-- Dashboard -->
            <path
              v-if="item.icon === 'chart-bar'"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
            />
            <!-- Server -->
            <path
              v-if="item.icon === 'server'"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"
            />
            <!-- Share/Topology -->
            <path
              v-if="item.icon === 'share'"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z"
            />
            <!-- Flows -->
            <path
              v-if="item.icon === 'arrows-right-left'"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M7 16l-4-4m0 0l4-4m-4 4h18m-4 4l4-4m0 0l-4-4"
            />
            <!-- Bell -->
            <path
              v-if="item.icon === 'bell'"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
            />
            <!-- Cog -->
            <path
              v-if="item.icon === 'cog'"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
            />
            <path
              v-if="item.icon === 'cog'"
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
            />
          </svg>
          <span v-if="sidebarOpen" class="ml-3">{{ item.name }}</span>
          <!-- Alert badge -->
          <span
            v-if="item.icon === 'bell' && alertsStore.unacknowledgedCount > 0 && sidebarOpen"
            class="ml-auto badge badge-danger"
          >
            {{ alertsStore.unacknowledgedCount }}
          </span>
        </RouterLink>
      </nav>

      <!-- Toggle button -->
      <button
        @click="sidebarOpen = !sidebarOpen"
        class="flex items-center justify-center h-12 border-t border-gray-700 text-gray-400 hover:text-gray-200"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            :d="sidebarOpen ? 'M11 19l-7-7 7-7m8 14l-7-7 7-7' : 'M13 5l7 7-7 7M5 5l7 7-7 7'"
          />
        </svg>
      </button>
    </aside>

    <!-- Main content -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <!-- Header -->
      <header class="flex items-center justify-between h-16 px-6 bg-dark-200 border-b border-gray-700">
        <h1 class="text-lg font-semibold text-gray-100">
          {{ navigation.find(n => isActive(n.path))?.name || 'NetSentinel' }}
        </h1>

        <div class="flex items-center space-x-4">
          <!-- User menu -->
          <div class="flex items-center space-x-3">
            <span class="text-sm text-gray-400">{{ authStore.user?.username }}</span>
            <button
              @click="handleLogout"
              class="text-gray-400 hover:text-gray-200"
              title="Logout"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                />
              </svg>
            </button>
          </div>
        </div>
      </header>

      <!-- Page content -->
      <main class="flex-1 overflow-auto p-6">
        <RouterView />
      </main>
    </div>
  </div>
</template>
