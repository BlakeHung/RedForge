/**
 * Import Store
 *
 * Manages data import functionality for offline collaboration
 * Handles file reading, decryption, parsing, and deduplication
 */

import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';
import type {
  ImportOptions,
  ImportResult,
  ExportData,
} from '../types/offline-collaboration';
import { encryptionService } from '../services/encryption';
import { markdownService } from '../services/markdown';

interface ImportState {
  isImporting: boolean;
  progress: number;
  error: string | null;
  lastImportResult: ImportResult | null;
  previewData: ExportData | null;
}

export const useImportStore = defineStore('import', {
  state: (): ImportState => ({
    isImporting: false,
    progress: 0,
    error: null,
    lastImportResult: null,
    previewData: null,
  }),

  actions: {
    /**
     * Import scan data from encrypted Markdown file
     *
     * @param options - Import options
     * @param filePath - Optional file path (if not provided, will show file picker)
     */
    async importData(
      options: ImportOptions,
      filePath?: string
    ): Promise<ImportResult> {
      this.isImporting = true;
      this.progress = 0;
      this.error = null;

      try {
        // Step 1: Read file (20%)
        this.progress = 20;
        const markdown = await this.readMarkdownFile(filePath);

        // Step 2: Parse Markdown (40%)
        this.progress = 40;
        const parsed = markdownService.parseMarkdown(markdown);

        // Step 3: Decrypt if needed (60%)
        this.progress = 60;
        let exportData: ExportData;

        if (parsed.encryptedBlock) {
          if (!options.passphrase) {
            throw new Error('Passphrase required for encrypted file');
          }

          const decrypted = await encryptionService.decrypt(
            parsed.encryptedBlock,
            options.passphrase
          );
          exportData = JSON.parse(decrypted);
        } else {
          throw new Error('No encrypted data block found in file');
        }

        // Step 4: Deduplicate and import (80%)
        this.progress = 80;
        const result = await this.importToDatabase(exportData, options);

        // Step 5: Complete (100%)
        this.progress = 100;
        this.lastImportResult = result;

        console.log('Import completed:', result);
        return result;
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Import failed';
        throw error;
      } finally {
        this.isImporting = false;
      }
    },

    /**
     * Preview import file without importing
     *
     * @param filePath - Optional file path
     * @param passphrase - Optional passphrase for encrypted files
     */
    async previewImport(
      filePath?: string,
      passphrase?: string
    ): Promise<ExportData> {
      try {
        const markdown = await this.readMarkdownFile(filePath);
        const parsed = markdownService.parseMarkdown(markdown);

        let exportData: ExportData;

        if (parsed.encryptedBlock) {
          if (!passphrase) {
            // Return summary only if no passphrase
            throw new Error('Passphrase required to preview encrypted content');
          }

          const decrypted = await encryptionService.decrypt(
            parsed.encryptedBlock,
            passphrase
          );
          exportData = JSON.parse(decrypted);
        } else {
          throw new Error('No data found in file');
        }

        this.previewData = exportData;
        return exportData;
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Preview failed';
        throw error;
      }
    },

    /**
     * Read Markdown file using Tauri
     */
    async readMarkdownFile(filePath?: string): Promise<string> {
      try {
        let path = filePath;

        if (!path) {
          const selected = await open({
            multiple: false,
            filters: [
              {
                name: 'Markdown Files',
                extensions: ['md', 'md.enc'],
              },
            ],
          });

          if (!selected || typeof selected !== 'string') {
            throw new Error('No file selected');
          }

          path = selected;
        }

        const content = await readTextFile(path);
        return content;
      } catch (error) {
        throw new Error(`Failed to read file: ${error}`);
      }
    },

    /**
     * Import data to database via Rust backend
     */
    async importToDatabase(
      data: ExportData,
      options: ImportOptions
    ): Promise<ImportResult> {
      try {
        // First, deduplicate the data
        const deduplicatedData = await invoke<ExportData>(
          'deduplicate_import_data',
          { data }
        );

        // Then import to database
        const result = await invoke<ImportResult>('import_scan_data', {
          data: deduplicatedData,
          skipDuplicates: options.skipDuplicates ?? true,
          mergeStrategy: options.mergeStrategy ?? 'skip',
        });

        return result;
      } catch (error) {
        throw new Error(`Failed to import to database: ${error}`);
      }
    },

    /**
     * Verify file integrity before import
     */
    async verifyFile(filePath?: string): Promise<{
      valid: boolean;
      metadata?: any;
      errors: string[];
    }> {
      const errors: string[] = [];

      try {
        const markdown = await this.readMarkdownFile(filePath);
        const parsed = markdownService.parseMarkdown(markdown);

        // Check version compatibility
        const version = parsed.frontMatter.version;
        if (!version || !this.isVersionCompatible(version)) {
          errors.push(`Incompatible version: ${version}`);
        }

        // Check format
        if (parsed.frontMatter.format !== 'encrypted-markdown') {
          errors.push(`Unsupported format: ${parsed.frontMatter.format}`);
        }

        // Check if encrypted block exists when encryption is specified
        if (parsed.frontMatter.encryption && !parsed.encryptedBlock) {
          errors.push('Encrypted block missing in encrypted file');
        }

        return {
          valid: errors.length === 0,
          metadata: parsed.frontMatter,
          errors,
        };
      } catch (error) {
        errors.push(error instanceof Error ? error.message : 'File verification failed');
        return {
          valid: false,
          errors,
        };
      }
    },

    /**
     * Check version compatibility
     */
    isVersionCompatible(version: string): boolean {
      const [major] = version.split('.').map(Number);
      return major === 1; // Currently only support v1.x.x
    },

    /**
     * Clear preview data
     */
    clearPreview(): void {
      this.previewData = null;
    },

    /**
     * Reset import state
     */
    reset(): void {
      this.isImporting = false;
      this.progress = 0;
      this.error = null;
      this.previewData = null;
    },
  },

  getters: {
    /**
     * Get import statistics from last import
     */
    lastImportStats(): {
      imported: number;
      skipped: number;
      errors: number;
    } | null {
      if (!this.lastImportResult) return null;

      const { imported, skipped, errors } = this.lastImportResult;

      return {
        imported:
          imported.scans + imported.findings + imported.annotations + imported.assets,
        skipped:
          skipped.scans + skipped.findings + skipped.annotations + skipped.assets,
        errors: errors.length,
      };
    },

    /**
     * Check if preview is available
     */
    hasPreview(): boolean {
      return this.previewData !== null;
    },
  },
});
