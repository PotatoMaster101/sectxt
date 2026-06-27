<template>
  <div class="grid mx-auto place-items-stretch w-[50vw]">
    <MessageView
      v-if="message"
      :message="message"
    />
  </div>
</template>

<script setup lang="ts">
import { toByteArray } from 'base64-js';
import { useConfirm } from 'primevue';
import { onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';

import { getMessage, type GetMessageMetadataResponse,getMetadata } from '@/api/generated';
import MessageView from '@/components/MessageView.vue';
import type { EncryptionOutput } from '@/composables/useCrypto.ts';
import { useNotification } from '@/composables/useNotification.ts';
import { toEncryptionOutput } from '@/mappers/message.js';

const loading = ref<boolean>(false);
const message = ref<EncryptionOutput | undefined>(undefined);
const confirm = useConfirm();
const notis = useNotification();
const route = useRoute();
const router = useRouter();

onMounted(async () => {
  const params = route.params as { id: string };
  const id = params.id;
  const hash = route.hash.substring(1);
  message.value = await load(id, hash);
});

const loadMessage = async (id: string, hash: string): Promise<EncryptionOutput | undefined> => {
  const rawKey = hash ? toByteArray(hash) : undefined;
  try {
    const { data, error } = await getMessage({ path: { id } });
    if (data && !error) return toEncryptionOutput(data.message, rawKey);
    notis.error(error);
  } catch (e) {
    notis.error(e);
  }
};

const loadMetadata = async (id: string): Promise<GetMessageMetadataResponse | undefined> => {
  try {
    const { data, error } = await getMetadata({ path: { id } });
    if (data && !error) return data;
    notis.error(error);
  } catch (e) {
    notis.error(e);
  }
};

const load = async (id: string, hash: string): Promise<EncryptionOutput | undefined> => {
  loading.value = true;
  try {
    const metadata = await loadMetadata(id);
    if (metadata) {
      if (!metadata.burnOnRead || (await requireConfirmation())) {
        return await loadMessage(id, hash);
      }
      redirectHome();
    }
  } finally {
    loading.value = false;
  }
};

const redirectHome = () => {
  router.push('/');
};

const requireConfirmation = (): Promise<boolean> => {
  return new Promise<boolean>((resolve) => {
    confirm.require({
      message: 'Message will be deleted after reading. Continue?',
      header: 'Confirmation',
      icon: 'pi pi-exclamation-triangle',
      accept: () => resolve(true),
      reject: () => resolve(false),
    });
  });
};
</script>
