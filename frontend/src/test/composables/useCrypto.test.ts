import { describe, expect, it } from 'vitest';

import { useCrypto } from '@/composables/useCrypto.ts';

describe('encrypt', () => {
  const secretText = 'Hello, World!';
  const password = 'password123';
  const crypto = useCrypto();

  it('encryption with a password', async () => {
    const sut = await crypto.encrypt(secretText, password);

    expect(sut.ciphertext.length).toBeGreaterThan(0);
    expect(sut.nonce.length).toBe(12);
    expect(sut.salt.length).toBe(16);
    expect(sut.rawKey).toBeUndefined();
  });

  it('encryption without a password', async () => {
    const sut = await crypto.encrypt(secretText, undefined);

    expect(sut.ciphertext.length).toBeGreaterThan(0);
    expect(sut.nonce.length).toBe(12);
    expect(sut.salt.length).toBe(16);
    expect(sut.rawKey?.length).toBe(32);
  });

  it('encryption produces unique outputs', async () => {
    const sut1 = await crypto.encrypt(secretText, password);
    const sut2 = await crypto.encrypt(secretText, password);

    expect(sut1.ciphertext).not.toEqual(sut2.ciphertext);
    expect(sut1.nonce).not.toEqual(sut2.nonce);
    expect(sut1.salt).not.toEqual(sut2.salt);
  });
});

describe('decrypt', () => {
  const secretText = 'Hello, World!';
  const password = 'password123';
  const crypto = useCrypto();

  it('decryption with a password', async () => {
    const encrypted = await crypto.encrypt(secretText, password);
    const sut = await crypto.decrypt(encrypted, password);
    expect(sut.toText()).toBe(secretText);
  });

  it('decryption without a password', async () => {
    const encrypted = await crypto.encrypt(secretText, undefined);
    const sut = await crypto.decrypt(encrypted, undefined);
    expect(sut.toText()).toBe(secretText);
  });

  it('decryption fails with wrong password', async () => {
    const encrypted = await crypto.encrypt(secretText, password);
    await expect(crypto.decrypt(encrypted, 'wrong_password')).rejects.toThrow();
  });

  it('decryption fails when rawKey is missing in no password mode', async () => {
    const encrypted = await crypto.encrypt(secretText, undefined);
    const stripped = {
      ...encrypted,
      rawKey: undefined,
    };
    await expect(crypto.decrypt(stripped, undefined)).rejects.toThrow();
  });
});
