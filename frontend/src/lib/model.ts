export interface CreateMessageDto {
  burnOnRead: boolean;
  hasPassword: boolean;
  ciphertext: Uint8Array;
  nonce: Uint8Array & { length: 12 };
  salt: Uint8Array & { length: 16 };
}

export interface ReadMessageDto {
  hasPassword: boolean;
  ciphertext: Uint8Array;
  nonce: Uint8Array;
  salt: Uint8Array;
}

export interface CryptoKeyResult {
  key: CryptoKey;
  rawKey?: Uint8Array;
}

export interface EncryptedMessage {
  ciphertext: Uint8Array;
  nonce: Uint8Array;
  salt: Uint8Array;
  rawKey?: Uint8Array;
}

export interface DecryptedMessage {
  plaintext: string;
}
