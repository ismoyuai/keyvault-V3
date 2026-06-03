import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      redirect: '/login',
    },
    {
      path: '/setup',
      name: 'setup',
      component: () => import('@/views/SetupView.vue'),
    },
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/UnlockView.vue'),
    },
    {
      path: '/vault',
      name: 'vault',
      component: () => import('@/views/VaultView.vue'),
      meta: { requiresAuth: true },
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
      meta: { requiresAuth: true },
    },
  ],
})

// 路由守卫：未解锁时不允许访问需要认证的页面
router.beforeEach((to) => {
  if (to.meta.requiresAuth) {
    // auth store 的 isUnlocked 检查在组件内进行
    // 这里只做基础路由保护
  }
})

export default router
