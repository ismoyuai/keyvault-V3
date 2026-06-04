<script setup lang="ts">
import { ref, watch } from 'vue'
import KvModal from '@/components/ui/KvModal.vue'
import KvInput from '@/components/ui/KvInput.vue'
import KvButton from '@/components/ui/KvButton.vue'
import PasswordStrength from '@/components/security/PasswordStrength.vue'
import { useAuthStore } from '@/stores/auth'
import { useToast } from '@/composables/useToast'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()

const auth = useAuthStore()
const toast = useToast()

const oldPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const loading = ref(false)

watch(() => props.open, (isOpen) => {
  if (!isOpen) {
    oldPassword.value = ''
    newPassword.value = ''
    confirmPassword.value = ''
  }
})

async function handleSubmit() {
  if (newPassword.value !== confirmPassword.value) {
    toast.error('两次密码不一致')
    return
  }
  if (newPassword.value.length < 8) {
    toast.error('密码长度至少 8 位')
    return
  }
  loading.value = true
  try {
    await auth.changePassword(oldPassword.value, newPassword.value)
    toast.success('主密码已修改')
    emit('close')
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : '修改失败'
    toast.error(message)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <KvModal :open="open" title="修改主密码" width="480px" blur @close="emit('close')">
    <div class="change-pw-form">
      <KvInput v-model="oldPassword" label="当前主密码" type="password" placeholder="输入当前密码" />
      <KvInput v-model="newPassword" label="新主密码" type="password" placeholder="至少 8 个字符" />
      <PasswordStrength :password="newPassword" variant="segments" />
      <KvInput v-model="confirmPassword" label="确认新密码" type="password" placeholder="再次输入新密码" />
    </div>
    <template #footer>
      <KvButton variant="ghost" @click="emit('close')">取消</KvButton>
      <KvButton :loading="loading" @click="handleSubmit">确认修改</KvButton>
    </template>
  </KvModal>
</template>

<style scoped>
.change-pw-form {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}
</style>
