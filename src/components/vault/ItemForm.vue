<script setup lang="ts">
import { ref, computed } from 'vue'
import KvButton from '@/components/ui/KvButton.vue'
import KvInput from '@/components/ui/KvInput.vue'
import KvModal from '@/components/ui/KvModal.vue'
import { TEMPLATES, TEMPLATE_LIST } from '@/constants/templates'
import { useVaultStore } from '@/stores/vault'
import { useToast } from '@/composables/useToast'
import { vault as vaultBridge } from '@/bridge/tauri'
import type { EntryType, FieldInput, CreateEntryInput, EntryMeta } from '@/types/vault'

interface Props {
  open: boolean
  editEntry?: EntryMeta | null
}

const props = withDefaults(defineProps<Props>(), {
  editEntry: null,
})

const emit = defineEmits<{
  close: []
  saved: []
}>()

const vault = useVaultStore()
const toast = useToast()
const step = ref<'type' | 'form'>(props.editEntry ? 'form' : 'type')
const selectedType = ref<EntryType>(props.editEntry?.entryType || 'login')
const title = ref(props.editEntry?.title || '')
const subtitle = ref(props.editEntry?.subtitle || '')
const tags = ref(props.editEntry?.tags.join(', ') || '')
const fields = ref<FieldInput[]>([])

const template = computed(() => TEMPLATES[selectedType.value])

function selectType(type: EntryType) {
  selectedType.value = type
  const tpl = TEMPLATES[type]
  fields.value = tpl.fields.map(f => ({
    fieldKey: f.key,
    fieldType: f.type,
    value: '',
    isSensitive: f.type === 'password' || f.type === 'otp',
  }))
  step.value = 'form'
}

function addCustomField() {
  fields.value.push({
    fieldKey: `field_${fields.value.length + 1}`,
    fieldType: 'text',
    value: '',
    isSensitive: false,
  })
}

function removeField(index: number) {
  fields.value.splice(index, 1)
}

async function handleSave() {
  if (!title.value.trim()) {
    toast.error('请输入标题')
    return
  }

  const tagList = tags.value
    .split(',')
    .map(t => t.trim())
    .filter(Boolean)

  const input: CreateEntryInput = {
    entryType: selectedType.value,
    title: title.value.trim(),
    subtitle: subtitle.value.trim() || undefined,
    tags: tagList,
    fields: fields.value.filter(f => f.value.trim()),
  }

  try {
    if (props.editEntry) {
      await vaultBridge.updateEntry(props.editEntry.id, input)
      toast.success('条目已更新')
    } else {
      await vaultBridge.createEntry(input)
      toast.success('条目已创建')
    }
    emit('saved')
    emit('close')
    resetForm()
  } catch (e: any) {
    toast.error(e.message || '保存失败')
  }
}

function resetForm() {
  step.value = props.editEntry ? 'form' : 'type'
  title.value = ''
  subtitle.value = ''
  tags.value = ''
  fields.value = []
}
</script>

<template>
  <KvModal :open="open" @close="emit('close')">
    <template #header>
      <h3 class="form-title">{{ editEntry ? '编辑条目' : '新建条目' }}</h3>
    </template>

    <!-- Step 1: 选择类型 -->
    <div v-if="step === 'type'" class="type-grid">
      <button
        v-for="tpl in TEMPLATE_LIST"
        :key="tpl.id"
        class="type-card"
        @click="selectType(tpl.id)"
      >
        <span class="type-card__name">{{ tpl.name }}</span>
      </button>
    </div>

    <!-- Step 2: 填写表单 -->
    <div v-else class="form-body">
      <KvInput
        v-model="title"
        label="标题"
        placeholder="例如：GitHub, OpenAI API"
      />
      <KvInput
        v-model="subtitle"
        label="副标题（可选）"
        placeholder="例如：github.com"
      />

      <div class="fields-section">
        <div class="fields-header">
          <span class="fields-label">字段</span>
          <button class="add-field-btn" @click="addCustomField">+ 添加字段</button>
        </div>

        <div
          v-for="(field, i) in fields"
          :key="i"
          class="field-edit-row"
        >
          <input
            v-model="field.fieldKey"
            class="field-key-input"
            placeholder="字段名"
          />
          <select v-model="field.fieldType" class="field-type-select">
            <option value="text">文本</option>
            <option value="password">密码</option>
            <option value="email">邮箱</option>
            <option value="url">URL</option>
            <option value="textarea">多行</option>
          </select>
          <input
            v-model="field.value"
            :type="field.fieldType === 'password' ? 'password' : 'text'"
            class="field-value-input"
            placeholder="值"
          />
          <button class="remove-field-btn" @click="removeField(i)">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
              <path d="M1 1l12 12M13 1L1 13" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
            </svg>
          </button>
        </div>
      </div>

      <KvInput
        v-model="tags"
        label="标签（逗号分隔）"
        placeholder="例如：work, dev, important"
      />
    </div>

    <template #footer>
      <KvButton v-if="step === 'type'" variant="ghost" @click="emit('close')">
        取消
      </KvButton>
      <template v-else>
        <KvButton v-if="!editEntry" variant="ghost" @click="step = 'type'">
          返回
        </KvButton>
        <KvButton variant="ghost" @click="emit('close')">取消</KvButton>
        <KvButton @click="handleSave">
          {{ editEntry ? '保存' : '创建' }}
        </KvButton>
      </template>
    </template>
  </KvModal>
</template>

<style scoped>
.form-title {
  font-size: var(--text-md);
  font-weight: 600;
}

.type-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: var(--space-2);
}

.type-card {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  transition: all var(--duration-fast);
}

.type-card:hover {
  border-color: var(--accent-blue);
  background: var(--accent-blue-glow);
}

.type-card__name {
  font-size: var(--text-sm);
  color: var(--text-primary);
}

.form-body {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.fields-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.fields-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.fields-label {
  font-size: var(--text-xs);
  color: var(--text-secondary);
  font-weight: 500;
}

.add-field-btn {
  font-size: var(--text-xs);
  color: var(--text-accent);
}

.add-field-btn:hover {
  text-decoration: underline;
}

.field-edit-row {
  display: flex;
  gap: var(--space-2);
  align-items: center;
}

.field-key-input {
  width: 100px;
  padding: var(--space-1) var(--space-2);
  font-size: var(--text-xs);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
}

.field-type-select {
  width: 80px;
  padding: var(--space-1) var(--space-2);
  font-size: var(--text-xs);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
}

.field-value-input {
  flex: 1;
  padding: var(--space-1) var(--space-2);
  font-size: var(--text-xs);
  background: var(--bg-input);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-sm);
  color: var(--text-primary);
}

.remove-field-btn {
  padding: var(--space-1);
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
}

.remove-field-btn:hover {
  color: var(--color-danger);
  background: var(--bg-elevated);
}
</style>
