mod config;
mod error;
mod db;
mod rules;
mod queue_manager;
mod log_parser;

use config::Config;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use std::path::Path;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_env()?;
    
    // Set up logging
    setup_logging(&config.logging)?;
    
    tracing::info!("IRC Log Parser - Rust Implementation");
    tracing::info!("Configuration loaded successfully");
    
    // Create database connection pool
    tracing::info!("Creating database connection pool...");
    let pool = db::connection::create_pool(&config.database).await?;
    tracing::info!("Database connection pool created successfully");
    
    // Spawn queue_manager task
    let queue_pool = pool.clone();
    let queue_config = config.queue_manager.clone();
    let queue_handle = tokio::spawn(async move {
        tracing::info!("Queue manager task starting");
        match queue_manager::run(queue_pool, queue_config).await {
            Ok(()) => {
                tracing::info!("Queue manager task completed");
                Ok(())
            }
            Err(e) => {
                tracing::error!("Queue manager task failed: {}", e);
                Err(e)
            }
        }
    });
    
    // Spawn log_parser task
    let parser_pool = pool.clone();
    let parser_config = config.log_parser.clone();
    let parser_handle = tokio::spawn(async move {
        tracing::info!("Log parser task starting");
        match log_parser::LogParser::new(parser_pool, parser_config).await {
            Ok(parser) => {
                match parser.run().await {
                    Ok(()) => {
                        tracing::info!("Log parser task completed");
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("Log parser task failed: {}", e);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to initialize log parser: {}", e);
                Err(e)
            }
        }
    });
    
    tracing::info!("All tasks spawned successfully");
    tracing::info!("Press Ctrl+C to shutdown gracefully");
    
    // Set up signal handlers for graceful shutdown
    tokio::select! {
        result = queue_handle => {
            match result {
                Ok(Ok(())) => tracing::info!("Queue manager task exited normally"),
                Ok(Err(e)) => tracing::error!("Queue manager task exited with error: {}", e),
                Err(e) => tracing::error!("Queue manager task panicked: {}", e),
            }
        }
        result = parser_handle => {
            match result {
                Ok(Ok(())) => tracing::info!("Log parser task exited normally"),
                Ok(Err(e)) => tracing::error!("Log parser task exited with error: {}", e),
                Err(e) => tracing::error!("Log parser task panicked: {}", e),
            }
        }
        _ = signal::ctrl_c() => {
            tracing::info!("Received SIGINT (Ctrl+C), initiating graceful shutdown...");
        }
        _ = wait_for_sigterm() => {
            tracing::info!("Received SIGTERM, initiating graceful shutdown...");
        }
    }
    
    tracing::info!("Shutdown complete");
    Ok(())
}

/// Wait for SIGTERM signal (Unix only)
#[cfg(unix)]
async fn wait_for_sigterm() {
    use tokio::signal::unix::{signal, SignalKind};
    
    let mut sigterm = signal(SignalKind::terminate())
        .expect("Failed to register SIGTERM handler");
    
    sigterm.recv().await;
}

/// Wait for SIGTERM signal (Windows - not supported, never returns)
#[cfg(not(unix))]
async fn wait_for_sigterm() {
    // SIGTERM is not available on Windows, so this will never complete
    // The select! will rely on Ctrl+C instead
    std::future::pending::<()>().await
}

/// Set up tracing subscriber with rotating file appenders
/// 
/// Note: The tracing-appender crate supports time-based rotation (daily) rather than
/// size-based rotation. The max_log_size_bytes and max_log_backups configuration values
/// are loaded but not currently used. Daily rotation is used as a reasonable alternative.
/// For production use, consider implementing a custom size-based rotating appender or
/// using an external log rotation tool like logrotate.
fn setup_logging(config: &config::LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create log directories if they don't exist
    let error_log_dir = Path::new(&config.error_log_path).parent()
        .ok_or("Invalid error log path")?;
    let debug_log_dir = Path::new(&config.debug_log_path).parent()
        .ok_or("Invalid debug log path")?;
    
    std::fs::create_dir_all(error_log_dir)?;
    std::fs::create_dir_all(debug_log_dir)?;
    
    // Extract file names from paths
    let error_log_name = Path::new(&config.error_log_path).file_name()
        .ok_or("Invalid error log filename")?
        .to_str()
        .ok_or("Invalid UTF-8 in error log filename")?;
    let debug_log_name = Path::new(&config.debug_log_path).file_name()
        .ok_or("Invalid debug log filename")?
        .to_str()
        .ok_or("Invalid UTF-8 in debug log filename")?;
    
    // Create rotating file appenders
    // Note: tracing-appender doesn't support exact byte limits or backup counts
    // It rotates daily by default. We'll use daily rotation as a reasonable alternative.
    // The max_log_size_bytes and max_log_backups from config are not used here.
    // For production, consider using external log rotation tools like logrotate.
    let error_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(error_log_name)
        .build(error_log_dir)?;
    
    let debug_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(debug_log_name)
        .build(debug_log_dir)?;
    
    // Create error log layer (ERROR and WARN levels only)
    let error_layer = fmt::layer()
        .with_writer(error_appender)
        .with_ansi(false)
        .with_filter(EnvFilter::new("warn"));
    
    // Create debug log layer (all levels)
    let debug_layer = fmt::layer()
        .with_writer(debug_appender)
        .with_ansi(false)
        .with_filter(EnvFilter::new("debug"));
    
    // Create stdout layer for console output
    let stdout_layer = fmt::layer()
        .with_filter(EnvFilter::new("info"));
    
    // Initialize the subscriber with all layers
    tracing_subscriber::registry()
        .with(error_layer)
        .with(debug_layer)
        .with(stdout_layer)
        .init();
    
    Ok(())
}
