<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { auth, setSessionToken } from '@/bridge/tauri'

const router = useRouter()

onMounted(async () => {
  try {
    const initialized = await auth.isInitialized()
    if (!initialized) {
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
