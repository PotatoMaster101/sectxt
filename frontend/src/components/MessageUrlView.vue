<template>
  <div class="flex flex-col gap-2">
    <FloatLabel variant="on">
      <InputText
        :model-value="props.messageUrl"
        fluid
        input-id="response"
        readonly
        @click="$emit('copy-url')"
      />
      <label for="response">Message URL</label>
    </FloatLabel>

    <Button
      class="w-fit self-start"
      label="Copy & Create New"
      @click="$emit('create-new')"
    />
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{ messageUrl: string }>();

defineEmits<{
  (e: 'create-new'): void;
  (e: 'copy-url'): void;
}>();
</script>
