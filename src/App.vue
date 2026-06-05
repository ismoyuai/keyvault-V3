<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { isTauri } from '@/bridge/tauri'

const router = useRouter()
const authStore = useAuthStore()

onMounted(async () => {
  if (!isTauri()) {
    authStore.error =
      '请在 Tauri 桌面应用中运行：npm run tauri dev（浏览器模式无法创建密码库）'
    router.push('/setup')
    return
  }
  try {
    await authStore.checkInitialized()
    if (!authStore.isInitialized) {
      router.push('/setup')
    } else {
      router.push('/login')
    }
  } catch {
    router.push('/login')
  }
})
</script>

<template>
  <router-view />
</template>
