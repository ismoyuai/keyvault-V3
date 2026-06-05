<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import KvModal from '@/components/ui/KvModal.vue'
import KvInput from '@/components/ui/KvInput.vue'
import KvButton from '@/components/ui/KvButton.vue'
import KvIcon from '@/components/icons/KvIcon.vue'
import { useAuthStore } from '@/stores/auth'
import { useToast } from '@/composables/useToast'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

const router = useRouter()
const auth = useAuthStore()
const toast = useToast()

const password = ref('')
const confirmation = ref('')
const loading = ref(false)
const error = ref<string | null>(null)

watch(() => props.open, (isOpen) => {
  if (isOpen) {
    password.value = ''
    confirmation.value = ''
    error.value = null
  }
})

async function handleWipe() {
  error.value = null
  if (confirmation.value !== 'DELETE') {
    error.value = '请输入 DELETE 以确认'
    return
  }
  loading.value = true
  try {
    await auth.emergencyWipe(password.value, confirmation.value)
    toast.success('所有数据已永久擦除')
    emit('close')
    router.replace('/setup')
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : '擦除失败'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <KvModal :open="open" width="440px" blur @close="emit('close')">
    <div class="wipe-modal">
      <div class="wipe-header">
        <div class="wipe-icon-wrap">
          <KvIcon name="warning" :size="22" class="wipe-icon" />
        </div>
        <div>
          <h2 class="wipe-title">紧急擦除所有数据？</h2>
          <p class="wipe-desc">此操作将永久删除本地所有加密库数据，且不可通过任何手段恢复。</p>
        </div>
      </div>

      <div class="wipe-form">
        <KvInput
          v-model="password"
          type="password"
          label="验证身份以继续"
          placeholder="主密码"
          autocomplete="off"
        />
        <KvInput
          v-model="confirmation"
          label="确认意图"
          placeholder="请输入 DELETE 以确认"
          autocomplete="off"
        />
        <p v-if="error" class="wipe-error">{{ error }}</p>
      </div>

      <div class="wipe-actions">
        <KvButton variant="ghost" @click="emit('close')">取消</KvButton>
        <KvButton variant="danger" :loading="loading" @click="handleWipe">
          <KvIcon name="delete_forever" :size="16" />
          永久擦除
        </KvButton>
      </div>
    </div>
  </KvModal>
</template>

<style scoped>
.wipe-modal {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.wipe-header {
  display: flex;
  gap: var(--space-3);
  align-items: flex-start;
}

.wipe-icon-wrap {
  flex-shrink: 0;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, var(--color-danger) 20%, transparent);
  border: 1px solid color-mix(in srgb, var(--color-danger) 30%, transparent);
  border-radius: var(--radius-full);
}

.wipe-icon {
  color: var(--color-danger);
}

.wipe-title {
  font-size: var(--text-lg);
  font-weight: 600;
  color: var(--color-danger);
  margin-bottom: var(--space-1);
}

.wipe-desc {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: 1.5;
}

.wipe-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.wipe-error {
  font-size: var(--text-sm);
  color: var(--color-danger);
}

.wipe-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-3);
  padding-top: var(--space-4);
  border-top: 1px solid var(--border-subtle);
}
</style>
