<script setup lang="ts">
import { ref, watch, computed, withDefaults } from 'vue'
import KvIcon from '@/components/icons/KvIcon.vue'
import KvModal from '@/components/ui/KvModal.vue'
import KvButton from '@/components/ui/KvButton.vue'
import type { EntryMeta } from '@/types/vault'

const props = withDefaults(defineProps<{
  open: boolean
  entry: EntryMeta | null
  mode?: 'soft-delete' | 'permanent'
}>(), { mode: 'soft-delete' })

const emit = defineEmits<{
  close: []
  confirm: []
}>()

const acknowledged = ref(false)

const isPermanent = computed(() => props.mode === 'permanent')

watch(() => props.open, (isOpen) => {
  if (isOpen) acknowledged.value = false
})

function handleConfirm() {
  if (!acknowledged.value) return
  emit('confirm')
  emit('close')
}
</script>

<template>
  <KvModal
    :open="open && !!entry"
    width="420px"
    blur
    @close="emit('close')"
  >
    <div class="delete-modal-body">
      <div class="delete-icon-wrap">
        <KvIcon name="delete" :size="24" class="delete-icon" />
      </div>
      <div class="delete-content">
        <h2 class="delete-title">{{ isPermanent ? '永久删除？' : '移至回收站？' }}</h2>
        <p class="delete-desc">
          <template v-if="isPermanent">
            将永久删除「{{ entry?.title }}」及其所有字段。此操作<strong>不可撤销</strong>。
          </template>
          <template v-else>
            将「{{ entry?.title }}」移至回收站，保留 30 天后自动永久销毁。在此之前可恢复。
          </template>
        </p>
        <label class="delete-check">
          <input v-model="acknowledged" type="checkbox" class="delete-checkbox" />
          <span>{{ isPermanent ? '我已了解此操作不可撤销' : '我已了解上述说明' }}</span>
        </label>
      </div>
    </div>
    <template #footer>
      <KvButton variant="ghost" @click="emit('close')">取消</KvButton>
      <KvButton
        variant="danger"
        :disabled="!acknowledged"
        @click="handleConfirm"
      >
        {{ isPermanent ? '永久删除' : '移至回收站' }}
      </KvButton>
    </template>
  </KvModal>
</template>

<style scoped>
.delete-modal-body {
  display: flex;
  gap: var(--space-4);
}

.delete-icon-wrap {
  flex-shrink: 0;
  padding-top: var(--space-1);
}

.delete-icon {
  color: var(--color-danger);
}

.delete-content {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  min-width: 0;
}

.delete-title {
  font-size: var(--text-md);
  font-weight: 700;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.delete-desc {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: 1.5;
}

.delete-check {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  margin-top: var(--space-2);
  font-size: var(--text-sm);
  color: var(--text-primary);
  cursor: pointer;
}

.delete-check:hover span {
  color: var(--text-accent);
}

.delete-checkbox {
  width: 16px;
  height: 16px;
  accent-color: var(--accent-blue);
  cursor: pointer;
}
</style>
