import { fromByteArray, toByteArray } from 'base64-js';

import type { CreateAttachmentRequest, GetAttachmentResponse } from '@/api/generated';
import type { EncryptionOutput } from '@/composables/useCrypto.ts';

export function toCreateAttachmentRequest(attachment: EncryptionOutput): CreateAttachmentRequest {
  if (attachment.nonce.length !== 12) {
    throw new Error('nonce must be 12 bytes');
  }
  if (attachment.salt.length !== 16) {
    throw new Error('salt must be 16 bytes');
  }
  if (attachment.ciphertext.length === 0) {
    throw new Error('ciphertext must not be empty');
  }
  return {
    ciphertext: fromByteArray(attachment.ciphertext),
    salt: fromByteArray(attachment.salt),
    nonce: fromByteArray(attachment.nonce),
  };
}

export function toEncryptionOutput(dto: GetAttachmentResponse, rawKey: Uint8Array | undefined): EncryptionOutput {
  return {
    ciphertext: toByteArray(dto.ciphertext),
    salt: toByteArray(dto.salt),
    nonce: toByteArray(dto.nonce),
    rawKey,
  };
}
