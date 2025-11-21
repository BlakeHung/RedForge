/**
 * Encryption Service
 *
 * Provides AES-256-GCM encryption/decryption for offline collaboration
 * Using Web Crypto API for secure, browser-native cryptography
 */

import type { EncryptedData, EncryptionOptions } from '../types/offline-collaboration';

export class EncryptionService {
  private readonly algorithm = 'AES-GCM';
  private readonly keyLength = 256;
  private readonly defaultIterations = 100000;

  /**
   * Encrypt data using AES-256-GCM
   *
   * @param data - Plain text data to encrypt
   * @param passphrase - Encryption passphrase
   * @param options - Optional encryption settings
   * @returns Encrypted data with IV and salt
   */
  async encrypt(
    data: string,
    passphrase: string,
    options?: Partial<EncryptionOptions>
  ): Promise<EncryptedData> {
    try {
      const encoder = new TextEncoder();
      const dataBuffer = encoder.encode(data);

      // Generate random salt for key derivation
      const salt = crypto.getRandomValues(new Uint8Array(16));

      // Derive encryption key from passphrase
      const iterations = options?.iterations ?? this.defaultIterations;
      const key = await this.deriveKey(passphrase, salt, iterations);

      // Generate random IV (Initialization Vector)
      const iv = crypto.getRandomValues(new Uint8Array(12));

      // Encrypt the data
      const encryptedBuffer = await crypto.subtle.encrypt(
        {
          name: this.algorithm,
          iv,
        },
        key,
        dataBuffer
      );

      return {
        ciphertext: this.bufferToBase64(encryptedBuffer),
        iv: this.bufferToBase64(iv),
        salt: this.bufferToBase64(salt),
      };
    } catch (error) {
      throw new Error(`Encryption failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Decrypt data using AES-256-GCM
   *
   * @param encryptedData - Encrypted data with IV and salt
   * @param passphrase - Decryption passphrase
   * @param options - Optional decryption settings
   * @returns Decrypted plain text
   */
  async decrypt(
    encryptedData: EncryptedData,
    passphrase: string,
    options?: Partial<EncryptionOptions>
  ): Promise<string> {
    try {
      const cipherBuffer = this.base64ToBuffer(encryptedData.ciphertext);
      const iv = this.base64ToBuffer(encryptedData.iv);
      const salt = this.base64ToBuffer(encryptedData.salt);

      // Derive the same key from passphrase and salt
      const iterations = options?.iterations ?? this.defaultIterations;
      const key = await this.deriveKey(passphrase, salt, iterations);

      // Decrypt the data
      const decryptedBuffer = await crypto.subtle.decrypt(
        {
          name: this.algorithm,
          iv,
        },
        key,
        cipherBuffer
      );

      const decoder = new TextDecoder();
      return decoder.decode(decryptedBuffer);
    } catch (error) {
      if (error instanceof Error && error.name === 'OperationError') {
        throw new Error('Decryption failed: Invalid passphrase or corrupted data');
      }
      throw new Error(`Decryption failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Derive encryption key from passphrase using PBKDF2
   *
   * @param passphrase - User-provided passphrase
   * @param salt - Random salt for key derivation
   * @param iterations - Number of PBKDF2 iterations (default: 100,000)
   * @returns Derived CryptoKey
   */
  private async deriveKey(
    passphrase: string,
    salt: Uint8Array,
    iterations: number
  ): Promise<CryptoKey> {
    const encoder = new TextEncoder();
    const passphraseBuffer = encoder.encode(passphrase);

    // Import passphrase as key material
    const keyMaterial = await crypto.subtle.importKey(
      'raw',
      passphraseBuffer,
      'PBKDF2',
      false,
      ['deriveKey']
    );

    // Derive the actual encryption key
    return await crypto.subtle.deriveKey(
      {
        name: 'PBKDF2',
        salt,
        iterations,
        hash: 'SHA-256',
      },
      keyMaterial,
      {
        name: this.algorithm,
        length: this.keyLength,
      },
      false,
      ['encrypt', 'decrypt']
    );
  }

  /**
   * Convert ArrayBuffer to Base64 string
   */
  private bufferToBase64(buffer: ArrayBuffer): string {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.length; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary);
  }

  /**
   * Convert Base64 string to Uint8Array
   */
  private base64ToBuffer(base64: string): Uint8Array {
    const binary = atob(base64);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
  }

  /**
   * Validate passphrase strength
   *
   * @param passphrase - Passphrase to validate
   * @returns Validation result with strength score and suggestions
   */
  validatePassphrase(passphrase: string): {
    valid: boolean;
    strength: 'weak' | 'medium' | 'strong';
    suggestions: string[];
  } {
    const suggestions: string[] = [];
    let score = 0;

    // Length check
    if (passphrase.length < 8) {
      suggestions.push('Use at least 8 characters');
    } else if (passphrase.length >= 12) {
      score += 2;
    } else {
      score += 1;
    }

    // Complexity checks
    if (/[a-z]/.test(passphrase)) score += 1;
    if (/[A-Z]/.test(passphrase)) score += 1;
    if (/[0-9]/.test(passphrase)) score += 1;
    if (/[^a-zA-Z0-9]/.test(passphrase)) score += 1;

    if (!/[a-z]/.test(passphrase) || !/[A-Z]/.test(passphrase)) {
      suggestions.push('Mix uppercase and lowercase letters');
    }
    if (!/[0-9]/.test(passphrase)) {
      suggestions.push('Include numbers');
    }
    if (!/[^a-zA-Z0-9]/.test(passphrase)) {
      suggestions.push('Add special characters');
    }

    // Determine strength
    let strength: 'weak' | 'medium' | 'strong';
    if (score >= 5) {
      strength = 'strong';
    } else if (score >= 3) {
      strength = 'medium';
    } else {
      strength = 'weak';
    }

    return {
      valid: passphrase.length >= 8 && score >= 3,
      strength,
      suggestions,
    };
  }
}

// Export singleton instance
export const encryptionService = new EncryptionService();
