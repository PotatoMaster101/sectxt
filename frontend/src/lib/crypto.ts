import type { CryptoKeyResult, DecryptedMessage, EncryptedMessage } from '@/lib/model.ts';

const AES = 'AES-GCM';
const KEY_USAGES: KeyUsage[] = ['encrypt', 'decrypt'];
const PBKDF2 = 'PBKDF2';
const PBKDF2_ITERATIONS = 600000;

export async function encrypt(text: string, password: string = ''): Promise<EncryptedMessage> {
  const salt = crypto.getRandomValues(new Uint8Array(16));
  const nonce = crypto.getRandomValues(new Uint8Array(12));
  const encoder = new TextEncoder();

  let cryptoKey: CryptoKeyResult | null;
  if (password === '') {
    cryptoKey = await generateEphemeralKey();
  } else {
    const passwordBytes = encoder.encode(password);
    cryptoKey = { key: await derivePasswordKey(passwordBytes, salt) };
  }

  const textBytes = encoder.encode(text);
  const ciphertext = await crypto.subtle.encrypt({ name: 'AES-GCM', iv: nonce }, cryptoKey.key, textBytes);
  return {
    ciphertext: new Uint8Array(ciphertext),
    nonce,
    salt,
    rawKey: cryptoKey.rawKey,
  };
}

export async function decrypt(message: EncryptedMessage, password = ''): Promise<DecryptedMessage> {
  let key: CryptoKey | null;
  if (password === '') {
    if (!message.rawKey) {
      throw new Error('rawKey missing');
    }
    key = await importRawKey(message.rawKey as BufferSource);
  } else {
    key = await derivePasswordKey(new TextEncoder().encode(password), message.salt as BufferSource);
  }

  const decryptedBuffer = await crypto.subtle.decrypt(
    { name: AES, iv: message.nonce as BufferSource },
    key,
    message.ciphertext as BufferSource
  );
  return { plaintext: new TextDecoder().decode(decryptedBuffer) };
}

async function generateEphemeralKey(): Promise<CryptoKeyResult> {
  const rawKey = crypto.getRandomValues(new Uint8Array(32));
  const key = await importRawKey(rawKey);
  return { key, rawKey };
}

async function importRawKey(rawKey: BufferSource): Promise<CryptoKey> {
  return await crypto.subtle.importKey('raw', rawKey, AES, false, KEY_USAGES);
}

async function derivePasswordKey(passwordBytes: BufferSource, salt: BufferSource): Promise<CryptoKey> {
  const baseKey = await crypto.subtle.importKey('raw', passwordBytes, PBKDF2, false, ['deriveKey']);
  return await crypto.subtle.deriveKey(
    {
      name: PBKDF2,
      salt,
      iterations: PBKDF2_ITERATIONS,
      hash: 'SHA-256',
    },
    baseKey,
    { name: AES, length: 256 },
    false,
    KEY_USAGES
  );
}
