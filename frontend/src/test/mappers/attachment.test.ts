import { describe, expect, it } from 'vitest';

import type { GetAttachmentResponse } from '@/api/generated';
import { toCreateAttachmentRequest } from '@/mappers/attachment.ts';
import { toEncryptionOutput } from '@/mappers/attachment.ts';

describe('toCreateAttachmentRequest', () => {
  it('creates a dto', () => {
    const attachment = {
      ciphertext: new Uint8Array(10),
      nonce: new Uint8Array(12),
      salt: new Uint8Array(16),
      rawKey: new Uint8Array(32),
    };

    const sut = toCreateAttachmentRequest(attachment);
    expect(sut).toEqual({
      ciphertext: 'AAAAAAAAAAAAAA==',
      nonce: 'AAAAAAAAAAAAAAAA',
      salt: 'AAAAAAAAAAAAAAAAAAAAAA==',
    });
  });

  it('throw on invalid nonce', () => {
    const attachment = {
      ciphertext: new Uint8Array(10),
      nonce: new Uint8Array(3),
      salt: new Uint8Array(16),
      rawKey: new Uint8Array(32),
    };
    expect(() => toCreateAttachmentRequest(attachment)).toThrow();
  });

  it('throw on invalid salt', () => {
    const attachment = {
      ciphertext: new Uint8Array(10),
      nonce: new Uint8Array(12),
      salt: new Uint8Array(3),
      rawKey: new Uint8Array(32),
    };
    expect(() => toCreateAttachmentRequest(attachment)).toThrow();
  });

  it('throw on invalid ciphertext', () => {
    const attachment = {
      ciphertext: new Uint8Array(0),
      nonce: new Uint8Array(12),
      salt: new Uint8Array(16),
      rawKey: new Uint8Array(32),
    };
    expect(() => toCreateAttachmentRequest(attachment)).toThrow();
  });
});

describe('toEncryptionOutput', () => {
  it('creates an encrypted attachment', () => {
    const dto: GetAttachmentResponse = {
      id: '',
      ciphertext: 'AAAAAAAAAAAAAA==',
      nonce: 'AAAAAAAAAAAAAAAA',
      salt: 'AAAAAAAAAAAAAAAAAAAAAA==',
    };

    const sut = toEncryptionOutput(dto, undefined);
    expect(sut).toEqual({
      ciphertext: new Uint8Array(10),
      nonce: new Uint8Array(12),
      salt: new Uint8Array(16),
    });
  });

  it('creates an encrypted attachment (no password)', () => {
    const dto: GetAttachmentResponse = {
      id: '',
      ciphertext: 'AAAAAAAAAAAAAA==',
      nonce: 'AAAAAAAAAAAAAAAA',
      salt: 'AAAAAAAAAAAAAAAAAAAAAA==',
    };

    const sut = toEncryptionOutput(dto, new Uint8Array(32));
    expect(sut).toEqual({
      ciphertext: new Uint8Array(10),
      nonce: new Uint8Array(12),
      salt: new Uint8Array(16),
      rawKey: new Uint8Array(32),
    });
  });
});
