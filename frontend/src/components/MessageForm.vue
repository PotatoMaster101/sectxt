<template>
  <Form @submit="onSubmit" class="flex flex-col gap-4">
    <div class="flex gap-1 w-full">
      <FloatLabel class="w-full" variant="on">
        <Textarea v-model="message" fluid input-id="message" rows="10" style="resize: none" />
        <label for="message">Message</label>
      </FloatLabel>
    </div>

    <div class="flex gap-1 w-full">
      <FloatLabel class="w-full" variant="on">
        <Password v-model="password" class="w-full" input-id="password" fluid toggleMask />
        <label for="password">Password</label>
      </FloatLabel>
    </div>

    <div class="flex gap-2 items-center w-full">
      <FloatLabel class="w-full" variant="on">
        <Select
          v-model="ttlSeconds"
          :options="ttlOptions"
          fluid
          input-id="options"
          option-label="name"
          option-value="value"
        />
        <label for="options">Time to Live</label>
      </FloatLabel>
    </div>

    <div class="flex items-center justify-between w-full">
      <Button label="Create" type="submit" />

      <div class="flex items-center gap-2">
        <Checkbox v-model="burnAfterRead" input-id="burn" binary />
        <label for="burn">Burn After Reading</label>
      </div>
    </div>
  </Form>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const message = ref('');
const password = ref('');
const burnAfterRead = ref(false);
const ttlSeconds = ref(60 * 60);
const ttlOptions = ref([
  { name: '15 minutes', value: 15 * 60 },
  { name: '30 minutes', value: 30 * 60 },
  { name: '1 hour', value: 60 * 60 },
  { name: '1 day', value: 24 * 60 * 60 },
  { name: '7 days', value: 7 * 24 * 60 * 60 },
  { name: '30 days', value: 30 * 24 * 60 * 60 },
]);

const onSubmit = () => {
  console.log('form submitted');
};
</script>
