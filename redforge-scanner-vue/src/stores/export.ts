/**
 * Export Store
 *
 * Manages data export functionality for offline collaboration
 * Handles encryption, Markdown generation, and file saving
 */

import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';
import type { ExportOptions, ExportData } from '../types/offline-collaboration';
import { encryptionService } from '../services/encryption';
import { markdownService } from '../services/markdown';

interface ExportState {
  isExporting: boolean;
  progress: number;
  error: string | null;
  lastExportPath: string | null;
  lastExportTimestamp: string | null;
}

export const useExportStore = defineStore('export', {
  state: (): ExportState => ({
    isExporting: false,
    progress: 0,
    error: null,
    lastExportPath: null,
    lastExportTimestamp: null,
  }),

  actions: {
    /**
     * Export scan data to encrypted Markdown file
     *
     * @param options - Export options
     */
    async exportData(options: ExportOptions): Promise<void> {
      this.isExporting = true;
      this.progress = 0;
      this.error = null;

      try {
        // Step 1: Fetch data from backend (20%)
        this.progress = 20;
        const exportData = await this.fetchExportData(options);

        // Step 2: Encrypt if needed (40%)
        this.progress = 40;
        let encryptedData;
        if (options.encrypt && options.passphrase) {
          const jsonData = JSON.stringify(exportData, null, 2);
          encryptedData = await encryptionService.encrypt(
            jsonData,
            options.passphrase
          );
        }

        // Step 3: Generate Markdown (60%)
        this.progress = 60;
        const markdown = markdownService.generateMarkdown(
          exportData,
          encryptedData
        );

        // Step 4: Save file (80%)
        this.progress = 80;
        await this.saveMarkdownFile(markdown, options.encrypt);

        // Step 5: Complete (100%)
        this.progress = 100;
        this.lastExportTimestamp = new Date().toISOString();

        console.log('Export completed successfully');
      } catch (error) {
        this.error = error instanceof Error ? error.message : 'Export failed';
        throw error;
      } finally {
        this.isExporting = false;
      }
    },

    /**
     * Fetch export data from Rust backend
     */
    async fetchExportData(options: ExportOptions): Promise<ExportData> {
      try {
        const data = await invoke<ExportData>('export_scan_data', {
          scanIds: options.scanIds,
          includeFindingsOnly: options.includeFindingsOnly ?? false,
          includeAnnotations: options.includeAnnotations ?? true,
          includeAssets: options.includeAssets ?? true,
          since: options.since?.toISOString(),
        });

        // Add metadata
        data.metadata = {
          version: '1.0.0',
          format: 'encrypted-markdown',
          encryption: options.encrypt ? 'AES-256-GCM' : undefined,
          exported_by: options.exportedBy,
          team_id: options.teamId,
          exported_at: new Date().toISOString(),
        };

        return data;
      } catch (error) {
        throw new Error(`Failed to fetch export data: ${error}`);
      }
    },

    /**
     * Save Markdown file using Tauri dialog
     */
    async saveMarkdownFile(markdown: string, encrypted: boolean = false): Promise<void> {
      try {
        const defaultFilename = `redforge_export_${Date.now()}${encrypted ? '.md.enc' : '.md'}`;

        const filePath = await save({
          defaultPath: defaultFilename,
          filters: [
            {
              name: encrypted ? 'Encrypted Markdown' : 'Markdown',
              extensions: encrypted ? ['md.enc', 'md'] : ['md'],
            },
          ],
        });

        if (!filePath) {
          throw new Error('File save cancelled');
        }

        await writeTextFile(filePath, markdown);
        this.lastExportPath = filePath;

        console.log(`File saved to: ${filePath}`);
      } catch (error) {
        throw new Error(`Failed to save file: ${error}`);
      }
    },

    /**
     * Export incremental data (only new data since last export)
     */
    async exportIncremental(options: ExportOptions): Promise<void> {
      if (!this.lastExportTimestamp) {
        throw new Error('No previous export found. Use full export instead.');
      }

      const incrementalOptions: ExportOptions = {
        ...options,
        since: new Date(this.lastExportTimestamp),
      };

      await this.exportData(incrementalOptions);
    },

    /**
     * Validate export options
     */
    validateExportOptions(options: ExportOptions): {
      valid: boolean;
      errors: string[];
    } {
      const errors: string[] = [];

      if (options.encrypt && !options.passphrase) {
        errors.push('Passphrase is required for encryption');
      }

      if (options.encrypt && options.passphrase) {
        const validation = encryptionService.validatePassphrase(options.passphrase);
        if (!validation.valid) {
          errors.push('Passphrase is too weak');
          errors.push(...validation.suggestions);
        }
      }

      if (!options.exportedBy || options.exportedBy.trim() === '') {
        errors.push('Exporter name is required');
      }

      return {
        valid: errors.length === 0,
        errors,
      };
    },

    /**
     * Reset export state
     */
    reset(): void {
      this.isExporting = false;
      this.progress = 0;
      this.error = null;
    },
  },

  getters: {
    /**
     * Check if incremental export is available
     */
    canExportIncremental(): boolean {
      return this.lastExportTimestamp !== null;
    },

    /**
     * Get last export info
     */
    lastExportInfo(): { path: string | null; timestamp: string | null } {
      return {
        path: this.lastExportPath,
        timestamp: this.lastExportTimestamp,
      };
    },
  },
});
