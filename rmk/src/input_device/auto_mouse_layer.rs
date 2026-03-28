//! Auto mouse layer processor for RMK
//!
//! This processor automatically activates a configured keyboard layer when the
//! pointing device (trackball, mouse, etc.) moves, and deactivates it after a
//! configurable idle period.
//!
//! # Behavior
//!
//! - When the trackball moves for at least `activate_after`, the `mouse_layer_index` layer is activated.
//! - The layer stays active until `deactivate_after` ms of no movement.
//! - Continuous movement resets the deactivation timer.

use embassy_time::Instant;
use rmk_macro::processor;

use crate::config::AutoMouseLayerConfig;
use crate::event::PointingEvent;
use crate::keymap::KeyMap;

/// Processor that automatically activates/deactivates a layer based on pointing device movement.
///
/// Subscribes to [`PointingEvent`] and polls every 10ms to check for deactivation.
#[processor(subscribe = [PointingEvent], poll_interval = 10)]
pub struct AutoMouseLayerProcessor<'a> {
    keymap: &'a KeyMap<'a>,
    config: AutoMouseLayerConfig,
    /// Time of the first motion in the current movement sequence (None when idle)
    first_motion_time: Option<Instant>,
    /// Time of the most recent motion event (None when idle)
    last_motion_time: Option<Instant>,
    /// Whether the mouse layer is currently active
    layer_active: bool,
}

impl<'a> AutoMouseLayerProcessor<'a> {
    pub fn new(keymap: &'a KeyMap<'a>, config: AutoMouseLayerConfig) -> Self {
        Self {
            keymap,
            config,
            first_motion_time: None,
            last_motion_time: None,
            layer_active: false,
        }
    }

    async fn on_pointing_event(&mut self, _event: PointingEvent) {
        let now = Instant::now();

        // Record the start of a new movement sequence if not already tracking
        if self.first_motion_time.is_none() {
            self.first_motion_time = Some(now);
        }
        // Always update the last motion time to reset the deactivation timer
        self.last_motion_time = Some(now);

        // Activate the layer once the movement has lasted long enough
        if !self.layer_active {
            let elapsed = now.duration_since(self.first_motion_time.unwrap());
            if elapsed >= self.config.activate_after {
                self.keymap.activate_layer(self.config.mouse_layer_index);
                self.layer_active = true;
            }
        }
    }

    async fn poll(&mut self) {
        // Check whether the layer should be deactivated due to inactivity
        if let Some(last) = self.last_motion_time {
            if last.elapsed() >= self.config.deactivate_after {
                if self.layer_active {
                    self.keymap.deactivate_layer(self.config.mouse_layer_index);
                    self.layer_active = false;
                }
                // Reset tracking state so the next movement starts fresh
                self.first_motion_time = None;
                self.last_motion_time = None;
            }
        }
    }
}
