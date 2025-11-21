/**
 * Database Module
 *
 * Handles SQLite database initialization for RedForge Scanner
 * Database operations are performed from frontend using tauri-plugin-sql
 */

use tauri_plugin_sql::{Migration, MigrationKind};

/// Get database migrations
///
/// Returns migrations to be executed on database initialization
pub fn get_migrations() -> Vec<Migration> {
    vec![
        // Migration 1: Create initial tables
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: include_str!("migrations/001_create_initial_tables.sql"),
            kind: MigrationKind::Up,
        },
    ]
}
