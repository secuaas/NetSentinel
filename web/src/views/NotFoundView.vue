<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-dark-300">
    <div class="text-center">
      <h1 class="text-6xl font-bold text-primary-400">404</h1>
      <p class="mt-4 text-xl text-gray-300">Page not found</p>
      <p class="mt-2 text-gray-500">The page you're looking for doesn't exist.</p>
      <button
        @click="router.push('/')"
        class="btn btn-primary mt-6"
      >
        Go to Dashboard
      </button>
    </div>
  </div>
</template>
