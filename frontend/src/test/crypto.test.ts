import { decrypt, encrypt } from '@/lib/crypto.ts';
import { describe, expect, it } from 'vitest';

describe('Encrypt', () => {
  const secretText = 'Hello, World!';
  const password = 'password123';

  it('encryption with a password', async () => {
    const result = await encrypt(secretText, password);

    expect(result.ciphertext).toBeInstanceOf(Uint8Array);
    expect(result.nonce).toBeInstanceOf(Uint8Array);
    expect(result.salt).toBeInstanceOf(Uint8Array);
    expect(result.ciphertext.byteLength).toBeGreaterThan(0);
    expect(result.nonce.byteLength).toBe(12);
    expect(result.salt.byteLength).toBe(16);
    expect(result.rawKey).toBeUndefined();
  });

  it('encryption without a password', async () => {
    const result = await encrypt(secretText, '');

    expect(result.ciphertext).toBeInstanceOf(Uint8Array);
    expect(result.rawKey).toBeInstanceOf(Uint8Array);
    expect(result.rawKey?.byteLength).toBe(32);
  });

  it('encryption produces unique outputs', async () => {
    const result1 = await encrypt(secretText, password);
    const result2 = await encrypt(secretText, password);

    expect(result1.ciphertext).not.toEqual(result2.ciphertext);
    expect(result1.nonce).not.toEqual(result2.nonce);
    expect(result1.salt).not.toEqual(result2.salt);
  });
});

describe('Decrypt', () => {
  const secretText = 'Hello, World!';
  const password = 'password123';

  it('decryption with a password', async () => {
    const encrypted = await encrypt(secretText, password);
    const decrypted = await decrypt(encrypted, password);
    expect(decrypted.plaintext).toBe(secretText);
  });

  it('decryption without a password', async () => {
    const encrypted = await encrypt(secretText, '');
    const decrypted = await decrypt(encrypted, '');
    expect(decrypted.plaintext).toBe(secretText);
  });

  it('decryption fails with wrong password', async () => {
    const encrypted = await encrypt(secretText, password);
    await expect(decrypt(encrypted, 'wrong_password')).rejects.toThrow();
  });

  it('decryption fails when rawKey is missing in no password mode', async () => {
    const encrypted = await encrypt(secretText, '');
    const stripped = {
      ...encrypted,
      rawKey: undefined,
    };
    await expect(decrypt(stripped, '')).rejects.toThrow();
  });
});
