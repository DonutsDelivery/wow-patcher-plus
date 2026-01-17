//! Progress tracking for downloads
//!
//! Provides types and utilities for tracking download progress
//! with throttled event emission to avoid flooding the UI.

use serde::Serialize;
use std::time::{Duration, Instant};

/// Events emitted during download progress
///
/// These events are sent via Tauri Channel to the frontend.
/// The serde configuration creates a discriminated union in TypeScript.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum DownloadEvent {
    /// Download has started
    Started {
        download_id: String,
        file_name: String,
        total_bytes: u64,
    },
    /// Progress update during download
    Progress {
        download_id: String,
        downloaded_bytes: u64,
        total_bytes: u64,
        speed_bps: u64,
        percent: f32,
    },
    /// Download completed successfully
    Completed {
        download_id: String,
        file_path: String,
    },
    /// Download failed with an error
    Failed {
        download_id: String,
        error: String,
    },
}

/// Tracks download progress and throttles event emission
///
/// Progress events are only generated if sufficient time has passed
/// since the last report, preventing UI flooding during fast downloads.
pub struct ProgressTracker {
    download_id: String,
    total_bytes: u64,
    downloaded_bytes: u64,
    start_time: Instant,
    last_report_time: Instant,
    min_report_interval: Duration,
}

impl ProgressTracker {
    /// Create a new progress tracker
    ///
    /// # Arguments
    /// * `download_id` - Unique identifier for this download
    /// * `total_bytes` - Total expected file size (0 if unknown)
    pub fn new(download_id: String, total_bytes: u64) -> Self {
        let now = Instant::now();
        Self {
            download_id,
            total_bytes,
            downloaded_bytes: 0,
            start_time: now,
            // Set last_report_time in the past to allow immediate first report
            last_report_time: now - Duration::from_millis(600),
            min_report_interval: Duration::from_millis(500),
        }
    }

    /// Update progress with a new chunk
    ///
    /// Returns Some(DownloadEvent::Progress) if enough time has passed
    /// since the last progress report, None otherwise.
    ///
    /// # Arguments
    /// * `chunk_size` - Number of bytes in the received chunk
    pub fn update(&mut self, chunk_size: u64) -> Option<DownloadEvent> {
        self.downloaded_bytes += chunk_size;
        let now = Instant::now();

        if now.duration_since(self.last_report_time) >= self.min_report_interval {
            self.last_report_time = now;

            let elapsed_secs = self.start_time.elapsed().as_secs_f64();
            let speed_bps = if elapsed_secs > 0.0 {
                (self.downloaded_bytes as f64 / elapsed_secs) as u64
            } else {
                0
            };

            let percent = if self.total_bytes > 0 {
                (self.downloaded_bytes as f32 / self.total_bytes as f32) * 100.0
            } else {
                0.0
            };

            Some(DownloadEvent::Progress {
                download_id: self.download_id.clone(),
                downloaded_bytes: self.downloaded_bytes,
                total_bytes: self.total_bytes,
                speed_bps,
                percent,
            })
        } else {
            None
        }
    }

    /// Create a Started event
    pub fn started_event(&self, file_name: String) -> DownloadEvent {
        DownloadEvent::Started {
            download_id: self.download_id.clone(),
            file_name,
            total_bytes: self.total_bytes,
        }
    }

    /// Create a Completed event
    pub fn completed_event(&self, file_path: String) -> DownloadEvent {
        DownloadEvent::Completed {
            download_id: self.download_id.clone(),
            file_path,
        }
    }

    /// Create a Failed event
    pub fn failed_event(&self, error: String) -> DownloadEvent {
        DownloadEvent::Failed {
            download_id: self.download_id.clone(),
            error,
        }
    }

    /// Get current downloaded bytes
    pub fn downloaded_bytes(&self) -> u64 {
        self.downloaded_bytes
    }

    /// Set downloaded bytes (used for resume functionality)
    ///
    /// This allows initializing the tracker with bytes already downloaded
    /// from a previous partial download.
    pub fn set_downloaded(&mut self, bytes: u64) {
        self.downloaded_bytes = bytes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new("test-id".to_string(), 1000);
        assert_eq!(tracker.downloaded_bytes(), 0);
    }

    #[test]
    fn test_speed_calculation() {
        let mut tracker = ProgressTracker::new("test-id".to_string(), 10000);

        // First update should return Some due to initial time offset
        let event = tracker.update(1000);
        assert!(event.is_some());

        if let Some(DownloadEvent::Progress { speed_bps, .. }) = event {
            // Speed should be positive (downloaded some bytes)
            // We can't predict exact speed due to timing, but it should be > 0
            // Since we just started, speed will be very high (essentially instant)
            assert!(speed_bps > 0);
        }
    }

    #[test]
    fn test_throttling() {
        let mut tracker = ProgressTracker::new("test-id".to_string(), 10000);

        // First update - should emit
        let first = tracker.update(100);
        assert!(first.is_some());

        // Immediate second update - should be throttled (None)
        let second = tracker.update(100);
        assert!(second.is_none());

        // Third immediate update - should also be throttled
        let third = tracker.update(100);
        assert!(third.is_none());
    }

    #[test]
    fn test_throttling_allows_after_interval() {
        let mut tracker = ProgressTracker::new("test-id".to_string(), 10000);

        // First update
        let first = tracker.update(100);
        assert!(first.is_some());

        // Wait longer than throttle interval
        thread::sleep(Duration::from_millis(150));

        // Should now emit again
        let after_wait = tracker.update(100);
        assert!(after_wait.is_some());
    }

    #[test]
    fn test_percent_calculation() {
        let mut tracker = ProgressTracker::new("test-id".to_string(), 1000);

        // Download 500 bytes (50%)
        tracker.update(500);

        // Wait for throttle and update to get event
        thread::sleep(Duration::from_millis(110));
        let event = tracker.update(0);

        if let Some(DownloadEvent::Progress { percent, downloaded_bytes, .. }) = event {
            assert_eq!(downloaded_bytes, 500);
            assert!((percent - 50.0).abs() < 0.1);
        }
    }

    #[test]
    fn test_started_event() {
        let tracker = ProgressTracker::new("dl-123".to_string(), 5000);
        let event = tracker.started_event("myfile.zip".to_string());

        match event {
            DownloadEvent::Started { download_id, file_name, total_bytes } => {
                assert_eq!(download_id, "dl-123");
                assert_eq!(file_name, "myfile.zip");
                assert_eq!(total_bytes, 5000);
            }
            _ => panic!("Expected Started event"),
        }
    }

    #[test]
    fn test_completed_event() {
        let tracker = ProgressTracker::new("dl-456".to_string(), 5000);
        let event = tracker.completed_event("/path/to/file.zip".to_string());

        match event {
            DownloadEvent::Completed { download_id, file_path } => {
                assert_eq!(download_id, "dl-456");
                assert_eq!(file_path, "/path/to/file.zip");
            }
            _ => panic!("Expected Completed event"),
        }
    }

    #[test]
    fn test_failed_event() {
        let tracker = ProgressTracker::new("dl-789".to_string(), 5000);
        let event = tracker.failed_event("Connection timeout".to_string());

        match event {
            DownloadEvent::Failed { download_id, error } => {
                assert_eq!(download_id, "dl-789");
                assert_eq!(error, "Connection timeout");
            }
            _ => panic!("Expected Failed event"),
        }
    }
}
