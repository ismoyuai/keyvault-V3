<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()

onMounted(async () => {
  try {
    await authStore.checkInitialized()
    if (!authStore.isInitialized) {
      router.push('/setup')
    } else {
      router.push('/login')
    }
  } catch {
    // Tauri 环境未就绪（开发模式下 vite 预览）
    router.push('/login')
  }
})
</script>

<template>
  <router-view />
</template>
