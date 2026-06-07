import type { CreateMessageDto, EncryptedMessage } from '@/lib/model.ts';

export function toCreateMessageDto(message: EncryptedMessage, burnOnRead: boolean): CreateMessageDto {
  if (message.nonce.length !== 12) {
    throw new Error('nonce must be 12 bytes');
  }
  if (message.ciphertext.length === 0) {
    throw new Error('ciphertext must not be empty');
  }
  return {
    burnOnRead: burnOnRead,
    hasPassword: message.rawKey !== undefined,
    ciphertext: message.ciphertext as Uint8Array,
    nonce: message.nonce as Uint8Array & { length: 12 },
    salt: message.salt as Uint8Array & { length: 16 },
  }
}
