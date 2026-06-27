<template>
  <div class="flex flex-col">
    <FloatLabel variant="on">
      <Textarea
        v-model="decryptedMessage"
        fluid
        input-id="message"
        readonly
        rows="10"
        style="resize: none"
      />
      <label for="message">Message</label>
    </FloatLabel>

    <div v-if="!props.message.rawKey">
      <FloatLabel variant="on">
        <Password
          v-model="password"
          class="mt-4"
          fluid
          input-id="password"
          toggle-mask
        />
        <label for="password">Decryption Password</label>
      </FloatLabel>
      <Button
        class="justify-self-start mt-4"
        label="Decrypt"
        @click="onDecrypt"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { useToast } from 'primevue';
import { onMounted, ref } from 'vue';

import { type EncryptionOutput, useCrypto } from '@/composables/useCrypto.ts';

const props = defineProps<{ message: EncryptionOutput }>();
const password = ref<string | undefined>(undefined);
const decryptedMessage = ref<string | undefined>(undefined);
const toast = useToast();
const crypto = useCrypto();

onMounted(async () => {
  if (props.message.rawKey) {
    await onDecrypt();
  }
});

const onDecrypt = async () => {
  decryptedMessage.value = undefined;
  try {
    const decrypted = await crypto.decrypt(props.message, password.value);
    decryptedMessage.value = decrypted.toText();
  } catch (e) {
    password.value = undefined;
    toast.add({ severity: 'error', summary: 'Error', detail: 'Failed to decrypt message', life: 5000 });
    console.error(e);
  }
};
</script>
