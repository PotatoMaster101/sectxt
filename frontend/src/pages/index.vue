<template>
  <div class="mx-auto w-[50%]">
    <template v-if="messageUrl">
      <MessageUrlView
        :message-url="messageUrl"
        @copy-url="onCopyUrl"
        @create-new="onCreateNew"
      />
    </template>
    <template v-else>
      <MessageForm
        :loading="loading"
        @submit="onSubmit"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { fromByteArray } from 'base64-js';
import { ref } from 'vue';

import { createMessage, type CreateMessageRequest } from '@/api/generated';
import MessageForm, { type SubmitPayload } from '@/components/MessageForm.vue';
import MessageUrlView from '@/components/MessageUrlView.vue';
import { useCrypto } from '@/composables/useCrypto.ts';
import { useNotification } from '@/composables/useNotification.ts';
import { toCreateMessageRequest } from '@/mappers/message.js';

const loading = ref<boolean>(false);
const messageUrl = ref<string | undefined>(undefined);
const notis = useNotification();
const crypto = useCrypto();

const copyUrl = async () => {
  await navigator.clipboard.writeText(messageUrl.value ?? '');
  notis.info('URL copied to clipboard');
};

const submitMessage = async (payload: SubmitPayload) => {
  loading.value = true;
  try {
    const encrypted = await crypto.encrypt(payload.message ?? '', payload.password);
    const body = toCreateMessageRequest(encrypted, payload.burnAfterRead, payload.ttlSeconds);
    const url = await uploadMessage(body, encrypted.rawKey);
    if (url) {
      messageUrl.value = url;
    }
  } catch (e) {
    notis.error(e);
  } finally {
    loading.value = false;
  }
};

const uploadMessage = async (body: CreateMessageRequest, rawKey: Uint8Array | undefined): Promise<string | undefined> => {
  try {
    const { data, error } = await createMessage({ body });
    if (data && !error) {
      notis.success('Message created');
      return getMessageUrl(data.id, rawKey);
    } else {
      notis.error(error);
      return undefined;
    }
  } catch (e) {
    notis.error(e);
    return undefined;
  }
};

const getMessageUrl = (id: string, rawKey: Uint8Array | undefined): string => {
  if (rawKey) {
    const url = new URL(encodeURIComponent(id), window.location.origin);
    url.hash = fromByteArray(rawKey);
    return url.href;
  } else {
    return new URL(encodeURIComponent(id), window.location.origin).href;
  }
};

const onCopyUrl = async () => {
  await copyUrl();
};

const onCreateNew = async () => {
  await copyUrl();
  messageUrl.value = undefined;
};

const onSubmit = async (payload: SubmitPayload) => {
  await submitMessage(payload);
};
</script>
