<script setup lang="ts">
import { ref, watch } from 'vue'
import KvButton from '@/components/ui/KvButton.vue'
import KvInput from '@/components/ui/KvInput.vue'
import KvModal from '@/components/ui/KvModal.vue'
import EntryTypePicker from '@/components/vault/EntryTypePicker.vue'
import KvIcon from '@/components/icons/KvIcon.vue'
import { TEMPLATES } from '@/constants/templates'
import { useToast } from '@/composables/useToast'
import { usePasswordGenerator } from '@/composables/usePasswordGen'
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

const toast = useToast()
const passwordGen = usePasswordGenerator()
const step = ref<'type' | 'form'>(props.editEntry ? 'form' : 'type')
const selectedType = ref<EntryType>(props.editEntry?.entryType || 'login')
const title = ref(props.editEntry?.title || '')
const subtitle = ref(props.editEntry?.subtitle || '')
const tags = ref(props.editEntry?.tags.join(', ') || '')
const fields = ref<FieldInput[]>([])
const isLoadingFields = ref(false)

async function loadEditFields(entryId: string) {
  isLoadingFields.value = true
  fields.value = []
  try {
    const secrets = await vaultBridge.getEntrySecrets(entryId)
    fields.value = secrets.fields.map(f => ({
      fieldKey: f.fieldKey,
      fieldType: f.fieldType,
      value: f.value,
      isSensitive: f.isSensitive,
    }))
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : '加载字段失败'
    toast.error(message)
  } finally {
    isLoadingFields.value = false
  }
}

watch(() => props.open, async (isOpen) => {
  if (isOpen) {
    if (props.editEntry) {
      step.value = 'form'
      selectedType.value = props.editEntry.entryType
      title.value = props.editEntry.title
      subtitle.value = props.editEntry.subtitle || ''
      tags.value = props.editEntry.tags.join(', ')
      await loadEditFields(props.editEntry.id)
    } else {
      resetForm()
    }
  }
})

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
  if (isLoadingFields.value) return

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
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : '保存失败'
    toast.error(message)
  }
}

function resetForm() {
  step.value = 'type'
  title.value = ''
  subtitle.value = ''
  tags.value = ''
  fields.value = []
  selectedType.value = 'login'
  isLoadingFields.value = false
}
</script>

<template>
  <KvModal :open="open" width="560px" blur @close="emit('close')">
    <template #header>
      <h3 class="form-title">
        {{ editEntry ? '编辑条目' : step === 'type' ? '新建条目' : `新建 · ${TEMPLATES[selectedType].name}` }}
      </h3>
    </template>

    <EntryTypePicker v-if="step === 'type'" @select="selectType" />

    <div v-else class="form-body">
      <p v-if="isLoadingFields" class="fields-loading">正在加载字段…</p>
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
          <button type="button" class="add-field-btn" @click="addCustomField">+ 添加字段</button>
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
          <button
            v-if="field.fieldType === 'password'"
            type="button"
            class="icon-btn"
            title="生成密码"
            @click="field.value = passwordGen.generate()"
          >
            <KvIcon name="autorenew" :size="16" />
          </button>
          <button type="button" class="icon-btn icon-btn--danger" @click="removeField(i)">
            <KvIcon name="close" :size="16" />
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
        <KvButton :disabled="isLoadingFields" @click="handleSave">
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

.form-body {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.fields-loading {
  font-size: var(--text-xs);
  color: var(--text-secondary);
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

.icon-btn {
  padding: var(--space-1);
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
  flex-shrink: 0;
}

.icon-btn:hover {
  color: var(--accent-blue);
  background: var(--bg-elevated);
}

.icon-btn--danger:hover {
  color: var(--color-danger);
}
</style>
