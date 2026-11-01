use eframe::egui;
use egui::{pos2, vec2, Color32, CursorIcon, Pos2, Rect, Sense, Stroke, Vec2};
use std::path::PathBuf;

// ============================================================================
// 1. HANDLE DEFINITION & GEOMETRIC ANCHORS
// ============================================================================

/// The 8 anchor handles for image resizing and cropping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandleId {
    TopLeft,
    TopMid,
    TopRight,
    RightMid,
    BottomRight,
    BottomMid,
    BottomLeft,
    LeftMid,
}

impl HandleId {
    /// Ordered list of all 8 handles for iteration.
    pub const ALL: [HandleId; 8] = [
        HandleId::TopLeft,
        HandleId::TopMid,
        HandleId::TopRight,
        HandleId::RightMid,
        HandleId::BottomRight,
        HandleId::BottomMid,
        HandleId::BottomLeft,
        HandleId::LeftMid,
    ];

    /// Returns a human-readable display name.
    pub fn name(&self) -> &'static str {
        match self {
            HandleId::TopLeft => "Top-Left",
            HandleId::TopMid => "Top-Center",
            HandleId::TopRight => "Top-Right",
            HandleId::RightMid => "Right-Center",
            HandleId::BottomRight => "Bottom-Right",
            HandleId::BottomMid => "Bottom-Center",
            HandleId::BottomLeft => "Bottom-Left",
            HandleId::LeftMid => "Left-Center",
        }
    }

    /// Evaluates the handle's current 2D screen position on a given bounding box.
    pub fn position(&self, rect: Rect) -> Pos2 {
        match self {
            HandleId::TopLeft => rect.left_top(),
            HandleId::TopMid => pos2(rect.center().x, rect.top()),
            HandleId::TopRight => rect.right_top(),
            HandleId::RightMid => pos2(rect.right(), rect.center().y),
            HandleId::BottomRight => rect.right_bottom(),
            HandleId::BottomMid => pos2(rect.center().x, rect.bottom()),
            HandleId::BottomLeft => rect.left_bottom(),
            HandleId::LeftMid => pos2(rect.left(), rect.center().y),
        }
    }

    /// Mouse cursor to display when hovering or dragging this handle.
    pub fn cursor_icon(&self) -> CursorIcon {
        match self {
            HandleId::TopLeft | HandleId::BottomRight => CursorIcon::ResizeNwSe,
            HandleId::TopRight | HandleId::BottomLeft => CursorIcon::ResizeNeSw,
            HandleId::TopMid | HandleId::BottomMid => CursorIcon::ResizeVertical,
            HandleId::LeftMid | HandleId::RightMid => CursorIcon::ResizeHorizontal,
        }
    }

    /// Returns a description of the fixed anchor locked during dragging.
    pub fn anchor_description(&self) -> &'static str {
        match self {
            HandleId::TopLeft => "Bottom-Right point locked",
            HandleId::TopMid => "Bottom edge locked",
            HandleId::TopRight => "Bottom-Left point locked",
            HandleId::RightMid => "Left edge locked",
            HandleId::BottomRight => "Top-Left point locked",
            HandleId::BottomMid => "Top edge locked",
            HandleId::BottomLeft => "Top-Right point locked",
            HandleId::LeftMid => "Right edge locked",
        }
    }
}

// ============================================================================
// 2. MATHEMATICAL CORE: ANCHOR-BASED RESIZE & CROP
// ============================================================================

/// Minimum allowed dimensions in pixels to prevent panics, zero divisions, or inverted rects.
pub const MIN_RECT_SIZE: f32 = 10.0;

/// **FEATURE 1: Anchor-Based Resize System**
///
/// Recalculates the screen bounding box by locking the diagonally opposite point/edge as
/// a fixed anchor.
///
/// Math breakdown:
/// - Corner handles: The diagonally opposite corner is fixed in both X and Y.
///   Width and height are calculated relative to that anchor, clamped to >= `MIN_RECT_SIZE`,
///   and the new bounding box is constructed from `anchor` and the clamped dragged coordinate.
/// - Mid-edge handles: The opposite edge coordinate is fixed in 1D. The perpendicular axis is
///   completely unaffected.
pub fn apply_resize(image_rect: Rect, handle: HandleId, delta: Vec2) -> Rect {
    let mut min = image_rect.min;
    let mut max = image_rect.max;

    match handle {
        // Dragging TopLeft: Fixed Anchor = BottomRight (max.x, max.y)
        HandleId::TopLeft => {
            let anchor = max;
            let target = min + delta;
            let new_w = (anchor.x - target.x).max(MIN_RECT_SIZE);
            let new_h = (anchor.y - target.y).max(MIN_RECT_SIZE);
            min.x = anchor.x - new_w;
            min.y = anchor.y - new_h;
        }

        // Dragging TopMid: Fixed Anchor = Bottom edge (max.y)
        HandleId::TopMid => {
            let anchor_y = max.y;
            let target_y = min.y + delta.y;
            let new_h = (anchor_y - target_y).max(MIN_RECT_SIZE);
            min.y = anchor_y - new_h;
        }

        // Dragging TopRight: Fixed Anchor = BottomLeft (min.x, max.y)
        HandleId::TopRight => {
            let anchor_x = min.x;
            let anchor_y = max.y;
            let target_x = max.x + delta.x;
            let target_y = min.y + delta.y;
            let new_w = (target_x - anchor_x).max(MIN_RECT_SIZE);
            let new_h = (anchor_y - target_y).max(MIN_RECT_SIZE);
            max.x = anchor_x + new_w;
            min.y = anchor_y - new_h;
        }

        // Dragging RightMid: Fixed Anchor = Left edge (min.x)
        HandleId::RightMid => {
            let anchor_x = min.x;
            let target_x = max.x + delta.x;
            let new_w = (target_x - anchor_x).max(MIN_RECT_SIZE);
            max.x = anchor_x + new_w;
        }

        // Dragging BottomRight: Fixed Anchor = TopLeft (min.x, min.y)
        HandleId::BottomRight => {
            let anchor = min;
            let target = max + delta;
            let new_w = (target.x - anchor.x).max(MIN_RECT_SIZE);
            let new_h = (target.y - anchor.y).max(MIN_RECT_SIZE);
            max.x = anchor.x + new_w;
            max.y = anchor.y + new_h;
        }

        // Dragging BottomMid: Fixed Anchor = Top edge (min.y)
        HandleId::BottomMid => {
            let anchor_y = min.y;
            let target_y = max.y + delta.y;
            let new_h = (target_y - anchor_y).max(MIN_RECT_SIZE);
            max.y = anchor_y + new_h;
        }

        // Dragging BottomLeft: Fixed Anchor = TopRight (max.x, min.y)
        HandleId::BottomLeft => {
            let anchor_x = max.x;
            let anchor_y = min.y;
            let target_x = min.x + delta.x;
            let target_y = max.y + delta.y;
            let new_w = (anchor_x - target_x).max(MIN_RECT_SIZE);
            let new_h = (target_y - anchor_y).max(MIN_RECT_SIZE);
            min.x = anchor_x - new_w;
            max.y = anchor_y + new_h;
        }

        // Dragging LeftMid: Fixed Anchor = Right edge (max.x)
        HandleId::LeftMid => {
            let anchor_x = max.x;
            let target_x = min.x + delta.x;
            let new_w = (anchor_x - target_x).max(MIN_RECT_SIZE);
            min.x = anchor_x - new_w;
        }
    }

    Rect::from_min_max(min, max)
}

/// **FEATURE 2: Non-Destructive Crop System**
///
/// In Crop Mode:
/// 1. The handles adjust the normalized texture coordinates `uv_rect` ([0.0..1.0]).
/// 2. The screen `image_rect` shrinks/expands proportionally in lockstep.
/// 3. The pixel scale (`screen_pixels / uv_span`) remains invariant, ensuring that
///    the visible image content NEVER stretches, jumps, or distorts.
/// 4. The diagonally opposite point/edge is strictly locked as the fixed anchor.
///
/// Returns: `(new_image_rect, new_uv_rect)`
pub fn apply_crop(
    image_rect: Rect,
    uv_rect: Rect,
    handle: HandleId,
    delta: Vec2,
) -> (Rect, Rect) {
    let w_screen = image_rect.width();
    let h_screen = image_rect.height();
    let w_uv = uv_rect.width();
    let h_uv = uv_rect.height();

    // Guard against degraded zero rects
    if w_screen <= 0.0 || h_screen <= 0.0 || w_uv <= 0.0 || h_uv <= 0.0 {
        return (image_rect, uv_rect);
    }

    // Invariant ratios: screen pixels per unit UV and UV per screen pixel
    let px_per_uv_x = w_screen / w_uv;
    let px_per_uv_y = h_screen / h_uv;
    let uv_per_px_x = w_uv / w_screen;
    let uv_per_px_y = h_uv / h_screen;

    // Minimum UV span corresponding to MIN_RECT_SIZE pixels
    let min_uv_w = (MIN_RECT_SIZE * uv_per_px_x).min(w_uv * 0.95);
    let min_uv_h = (MIN_RECT_SIZE * uv_per_px_y).min(h_uv * 0.95);

    let mut new_screen = image_rect;
    let mut new_uv = uv_rect;

    // Identify which geometric edges are affected by this handle
    let affects_left = matches!(
        handle,
        HandleId::TopLeft | HandleId::LeftMid | HandleId::BottomLeft
    );
    let affects_right = matches!(
        handle,
        HandleId::TopRight | HandleId::RightMid | HandleId::BottomRight
    );
    let affects_top = matches!(
        handle,
        HandleId::TopLeft | HandleId::TopMid | HandleId::TopRight
    );
    let affects_bottom = matches!(
        handle,
        HandleId::BottomLeft | HandleId::BottomMid | HandleId::BottomRight
    );

    // ------------------------------------------------------------------------
    // HORIZONTAL (X) CROP CALCULATION
    // ------------------------------------------------------------------------
    if affects_left {
        // Dragging Left: Fixed Anchor is the Right edge
        let screen_anchor_x = image_rect.max.x;
        let uv_anchor_u = uv_rect.max.x;

        // Apply pointer delta in UV space
        let target_u_min = uv_rect.min.x + delta.x * uv_per_px_x;

        // Clamp: Cannot uncrop past 0.0 (texture edge), nor exceed anchor - min_uv_w
        let clamped_u_min = target_u_min.clamp(0.0, uv_anchor_u - min_uv_w);

        // Recalculate screen bounding box from anchor using invariant scale
        let new_uv_w = uv_anchor_u - clamped_u_min;
        let new_screen_w = new_uv_w * px_per_uv_x;

        new_uv.min.x = clamped_u_min;
        new_screen.min.x = screen_anchor_x - new_screen_w;
        new_screen.max.x = screen_anchor_x; // Explicitly locked
    } else if affects_right {
        // Dragging Right: Fixed Anchor is the Left edge
        let screen_anchor_x = image_rect.min.x;
        let uv_anchor_u = uv_rect.min.x;

        let target_u_max = uv_rect.max.x + delta.x * uv_per_px_x;

        // Clamp: Cannot crop narrower than anchor + min_uv_w, nor exceed 1.0 (texture edge)
        let clamped_u_max = target_u_max.clamp(uv_anchor_u + min_uv_w, 1.0);

        let new_uv_w = clamped_u_max - uv_anchor_u;
        let new_screen_w = new_uv_w * px_per_uv_x;

        new_uv.max.x = clamped_u_max;
        new_screen.min.x = screen_anchor_x; // Explicitly locked
        new_screen.max.x = screen_anchor_x + new_screen_w;
    }

    // ------------------------------------------------------------------------
    // VERTICAL (Y) CROP CALCULATION
    // ------------------------------------------------------------------------
    if affects_top {
        // Dragging Top: Fixed Anchor is the Bottom edge
        let screen_anchor_y = image_rect.max.y;
        let uv_anchor_v = uv_rect.max.y;

        let target_v_min = uv_rect.min.y + delta.y * uv_per_px_y;

        // Clamp: Cannot uncrop past 0.0, nor exceed anchor - min_uv_h
        let clamped_v_min = target_v_min.clamp(0.0, uv_anchor_v - min_uv_h);

        let new_uv_h = uv_anchor_v - clamped_v_min;
        let new_screen_h = new_uv_h * px_per_uv_y;

        new_uv.min.y = clamped_v_min;
        new_screen.min.y = screen_anchor_y - new_screen_h;
        new_screen.max.y = screen_anchor_y; // Explicitly locked
    } else if affects_bottom {
        // Dragging Bottom: Fixed Anchor is the Top edge
        let screen_anchor_y = image_rect.min.y;
        let uv_anchor_v = uv_rect.min.y;

        let target_v_max = uv_rect.max.y + delta.y * uv_per_px_y;

        // Clamp: Cannot crop smaller than anchor + min_uv_h, nor exceed 1.0
        let clamped_v_max = target_v_max.clamp(uv_anchor_v + min_uv_h, 1.0);

        let new_uv_h = clamped_v_max - uv_anchor_v;
        let new_screen_h = new_uv_h * px_per_uv_y;

        new_uv.max.y = clamped_v_max;
        new_screen.min.y = screen_anchor_y; // Explicitly locked
        new_screen.max.y = screen_anchor_y + new_screen_h;
    }

    (new_screen, new_uv)
}

// ============================================================================
// 3. IMAGE LOADING
// ============================================================================

/// Loads a PNG/JPG/WEBP/BMP file from disk into an `egui::TextureHandle`.
fn load_texture_from_file(
    ctx: &egui::Context,
    path: &std::path::Path,
) -> Result<(egui::TextureHandle, [usize; 2]), String> {
    let img = image::open(path).map_err(|e| format!("Failed to decode image: {e}"))?;
    let size = [img.width() as usize, img.height() as usize];
    let rgba = img.to_rgba8();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());
    let handle = ctx.load_texture(
        path.file_name().unwrap_or_default().to_string_lossy(),
        color_image,
        egui::TextureOptions::LINEAR,
    );
    Ok((handle, size))
}

// ============================================================================
// 4. APPLICATION STATE & LOGIC
// ============================================================================

pub struct ImageSandboxApp {
    /// Texture handle to the loaded image, or None if no image has been browsed yet.
    texture: Option<egui::TextureHandle>,
    /// Native pixel dimensions of the source image [width, height].
    orig_size: [usize; 2],
    /// File path of currently loaded image, if loaded from disk.
    loaded_file_path: Option<PathBuf>,

    /// Current screen-space position and size of the image.
    image_rect: Rect,
    /// Normalized UV texture coordinates ([0.0, 0.0] to [1.0, 1.0]).
    uv_rect: Rect,

    /// Whether Crop Mode is active.
    crop_mode: bool,

    /// Currently active handle being dragged, if any.
    active_handle: Option<HandleId>,

    /// Request to fit the newly loaded image into the viewport once canvas size is known.
    need_fit: bool,
}

impl ImageSandboxApp {
    pub fn new(_ctx: &egui::Context) -> Self {
        Self {
            texture: None,
            orig_size: [0, 0],
            loaded_file_path: None,
            image_rect: Rect::NOTHING,
            uv_rect: Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            crop_mode: false,
            active_handle: None,
            need_fit: false,
        }
    }

    /// Opens an OS file dialog to select and load an image into the sandbox.
    fn browse_and_load_image(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Images (*.png, *.jpg, *.jpeg, *.webp, *.bmp)",
                &["png", "jpg", "jpeg", "webp", "bmp"],
            )
            .pick_file()
        {
            match load_texture_from_file(ctx, &path) {
                Ok((handle, size)) => {
                    self.texture = Some(handle);
                    self.orig_size = size;
                    self.loaded_file_path = Some(path);
                    self.uv_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
                    self.crop_mode = false;
                    self.need_fit = true;
                }
                Err(err) => {
                    eprintln!("Error loading image: {err}");
                }
            }
        }
    }

    /// Resets the crop boundary back to [0.0..1.0] and expands `image_rect`
    /// to reveal the full image at the current screen pixel scale.
    fn reset_crop(&mut self) {
        if self.texture.is_none() || self.uv_rect.width() <= 0.0 || self.uv_rect.height() <= 0.0 {
            self.uv_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            return;
        }

        // Current scale factor
        let px_per_uv_x = self.image_rect.width() / self.uv_rect.width();
        let px_per_uv_y = self.image_rect.height() / self.uv_rect.height();

        // Expand screen bounding box to full UV coordinates without shifting existing pixels
        let full_min_x = self.image_rect.min.x - self.uv_rect.min.x * px_per_uv_x;
        let full_min_y = self.image_rect.min.y - self.uv_rect.min.y * px_per_uv_y;
        let full_w = px_per_uv_x;
        let full_h = px_per_uv_y;

        self.image_rect = Rect::from_min_size(pos2(full_min_x, full_min_y), vec2(full_w, full_h));
        self.uv_rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    }

    /// Resets image dimensions to 1:1 pixel scale (cropped portion sized to its actual pixel count).
    fn reset_size_1_to_1(&mut self) {
        if self.texture.is_none() || self.orig_size[0] == 0 || self.orig_size[1] == 0 {
            return;
        }
        let center = self.image_rect.center();
        let cropped_w = self.orig_size[0] as f32 * self.uv_rect.width();
        let cropped_h = self.orig_size[1] as f32 * self.uv_rect.height();
        self.image_rect = Rect::from_center_size(center, vec2(cropped_w, cropped_h));
    }

    /// Fits the image comfortably inside the canvas viewport while preserving its aspect ratio.
    fn fit_to_viewport(&mut self, canvas_rect: Rect) {
        if self.texture.is_none() || self.orig_size[0] == 0 || self.orig_size[1] == 0 {
            return;
        }

        let margin = 80.0;
        let avail_w = (canvas_rect.width() - margin * 2.0).max(100.0);
        let avail_h = (canvas_rect.height() - margin * 2.0).max(100.0);

        let aspect = (self.orig_size[0] as f32 * self.uv_rect.width())
            / (self.orig_size[1] as f32 * self.uv_rect.height()).max(1.0);

        let (target_w, target_h) = if avail_w / avail_h > aspect {
            (avail_h * aspect, avail_h)
        } else {
            (avail_w, avail_w / aspect)
        };

        self.image_rect = Rect::from_center_size(canvas_rect.center(), vec2(target_w, target_h));
    }
}

// ============================================================================
// 5. EFRAME APP RENDERING & INTERACTION
// ============================================================================

impl eframe::App for ImageSandboxApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let has_image = self.texture.is_some();

        // --------------------------------------------------------------------
        // TOP CONTROL TOOLBAR
        // --------------------------------------------------------------------
        egui::Panel::top("top_toolbar").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = true;

                // 1. Browse Image Button
                if ui
                    .button("📁 Browse Image...")
                    .on_hover_text("Open a PNG or JPG file via file dialog")
                    .clicked()
                {
                    self.browse_and_load_image(ui.ctx());
                }

                ui.separator();

                // 2. Crop Mode Toggle (active only when image is loaded)
                let crop_btn_color = if self.crop_mode {
                    Color32::from_rgb(230, 126, 34) // Warm amber
                } else {
                    ui.visuals().widgets.inactive.bg_fill
                };

                let mode_label = if self.crop_mode {
                    "✂ Mode: CROP (Active)"
                } else {
                    "⤢ Mode: RESIZE"
                };

                if ui
                    .add_enabled(has_image, egui::Button::new(mode_label).fill(crop_btn_color))
                    .on_hover_text("Toggle between 8-Handle Resize Mode and Non-Destructive Crop Mode")
                    .clicked()
                {
                    self.crop_mode = !self.crop_mode;
                }

                ui.separator();

                // 3. Reset Crop
                let has_crop = has_image
                    && ((self.uv_rect.min.x > 1e-4)
                        || (self.uv_rect.min.y > 1e-4)
                        || (self.uv_rect.max.x < 0.9999)
                        || (self.uv_rect.max.y < 0.9999));

                if ui
                    .add_enabled(has_crop, egui::Button::new("↺ Reset Crop"))
                    .on_hover_text("Restore full uncropped image bounds (UV [0.0..1.0])")
                    .clicked()
                {
                    self.reset_crop();
                }

                // 4. Reset 1:1 Scale
                if ui
                    .add_enabled(has_image, egui::Button::new("1:1 Pixel Scale"))
                    .on_hover_text("Match screen size to exact source image pixel count")
                    .clicked()
                {
                    self.reset_size_1_to_1();
                }

                // 5. Fit to Window
                if ui
                    .add_enabled(has_image, egui::Button::new("🎯 Center & Fit"))
                    .on_hover_text("Center and scale image to fit canvas")
                    .clicked()
                {
                    let avail = ui.available_rect_before_wrap();
                    self.fit_to_viewport(avail);
                }

                ui.separator();

                // Information readout chips
                if has_image {
                    let w = self.image_rect.width().round();
                    let h = self.image_rect.height().round();
                    ui.label(format!("Screen: {w} × {h} px"));

                    let crop_pct_w = (self.uv_rect.width() * 100.0).round();
                    let crop_pct_h = (self.uv_rect.height() * 100.0).round();
                    ui.label(format!("Crop UV: {crop_pct_w}% × {crop_pct_h}%"));
                } else {
                    ui.label(
                        egui::RichText::new("No image loaded")
                            .italics()
                            .color(Color32::from_rgb(140, 145, 155)),
                    );
                }
            });
        });

        // --------------------------------------------------------------------
        // BOTTOM STATUS BAR / INSPECTOR
        // --------------------------------------------------------------------
        egui::Panel::bottom("bottom_status").show(ui, |ui| {
            ui.horizontal(|ui| {
                if has_image {
                    if let Some(handle) = self.active_handle {
                        ui.colored_label(
                            Color32::from_rgb(241, 196, 15),
                            format!("Dragging: {} ({})", handle.name(), handle.anchor_description()),
                        );
                    } else if self.crop_mode {
                        ui.label("✂ Crop Mode: Drag handles to adjust crop box. Bounding box wraps cropped pixels without stretching.");
                    } else {
                        ui.label("⤢ Resize Mode: Drag handles to resize image from diagonally opposite anchor. Drag center to pan.");
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(path) = &self.loaded_file_path {
                            ui.label(format!("File: {}", path.file_name().unwrap_or_default().to_string_lossy()));
                        } else {
                            ui.label(format!("Source: {}×{} px", self.orig_size[0], self.orig_size[1]));
                        }
                    });
                } else {
                    ui.label("No image loaded. Please browse and select an image file to begin.");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("File: [None]");
                    });
                }
            });
        });

        // --------------------------------------------------------------------
        // MAIN INTERACTIVE CANVAS
        // --------------------------------------------------------------------
        egui::CentralPanel::default().show(ui, |ui| {
            let canvas_size = ui.available_size();
            let (canvas_rect, _canvas_response) =
                ui.allocate_exact_size(canvas_size, Sense::hover());
            let painter = ui.painter_at(canvas_rect);

            // 1. Draw empty canvas background with subtle dotted engineering grid
            painter.rect_filled(canvas_rect, 0.0, Color32::from_rgb(20, 22, 27));
            let grid_step = 25.0;
            let start_x = canvas_rect.min.x + (canvas_rect.min.x % grid_step);
            let start_y = canvas_rect.min.y + (canvas_rect.min.y % grid_step);
            let mut gx = start_x;
            while gx < canvas_rect.max.x {
                let mut gy = start_y;
                while gy < canvas_rect.max.y {
                    painter.circle_filled(pos2(gx, gy), 1.0, Color32::from_rgb(38, 42, 50));
                    gy += grid_step;
                }
                gx += grid_step;
            }

            // Perform initial fit and centering upon loading
            if self.texture.is_some() && self.need_fit && canvas_rect.width() > 100.0 {
                self.fit_to_viewport(canvas_rect);
                self.need_fit = false;
            }

            // ONLY display the image, crop ghost, bounding box, and 8 handles if an image is loaded
            if let Some(texture) = &self.texture {
                // 2. In Crop Mode: Render faint ghost of the full uncropped image
                if self.crop_mode && self.uv_rect.width() > 0.0 && self.uv_rect.height() > 0.0 {
                    let px_per_uv_x = self.image_rect.width() / self.uv_rect.width();
                    let px_per_uv_y = self.image_rect.height() / self.uv_rect.height();

                    let full_min_x = self.image_rect.min.x - self.uv_rect.min.x * px_per_uv_x;
                    let full_min_y = self.image_rect.min.y - self.uv_rect.min.y * px_per_uv_y;
                    let full_rect = Rect::from_min_size(
                        pos2(full_min_x, full_min_y),
                        vec2(px_per_uv_x, px_per_uv_y),
                    );

                    // Dimmed full image underneath
                    painter.image(
                        texture.id(),
                        full_rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::from_white_alpha(55),
                    );

                    // Dashed outline of the full bounds
                    painter.rect_stroke(
                        full_rect,
                        0.0,
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(200, 200, 200, 80)),
                        egui::StrokeKind::Outside,
                    );
                }

                // 3. Render the active image inside `image_rect` using `uv_rect`
                painter.image(
                    texture.id(),
                    self.image_rect,
                    self.uv_rect,
                    Color32::WHITE,
                );

                // 4. Render Bounding Box Outline
                let (accent_color, border_width) = if self.crop_mode {
                    (Color32::from_rgb(230, 126, 34), 2.0) // Vibrant orange for Crop Mode
                } else {
                    (Color32::from_rgb(52, 152, 219), 1.5) // Crisp blue for Resize Mode
                };

                painter.rect_stroke(
                    self.image_rect,
                    0.0,
                    Stroke::new(border_width, accent_color),
                    egui::StrokeKind::Outside,
                );

                // 5. In Crop Mode: Render Rule-of-Thirds Grid
                if self.crop_mode {
                    let w = self.image_rect.width();
                    let h = self.image_rect.height();
                    let grid_stroke =
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(230, 126, 34, 110));

                    // Vertical grid lines
                    let x1 = self.image_rect.min.x + w / 3.0;
                    let x2 = self.image_rect.min.x + (2.0 * w) / 3.0;
                    painter.line_segment(
                        [
                            pos2(x1, self.image_rect.min.y),
                            pos2(x1, self.image_rect.max.y),
                        ],
                        grid_stroke,
                    );
                    painter.line_segment(
                        [
                            pos2(x2, self.image_rect.min.y),
                            pos2(x2, self.image_rect.max.y),
                        ],
                        grid_stroke,
                    );

                    // Horizontal grid lines
                    let y1 = self.image_rect.min.y + h / 3.0;
                    let y2 = self.image_rect.min.y + (2.0 * h) / 3.0;
                    painter.line_segment(
                        [
                            pos2(self.image_rect.min.x, y1),
                            pos2(self.image_rect.max.x, y1),
                        ],
                        grid_stroke,
                    );
                    painter.line_segment(
                        [
                            pos2(self.image_rect.min.x, y2),
                            pos2(self.image_rect.max.x, y2),
                        ],
                        grid_stroke,
                    );
                }

                // ------------------------------------------------------------
                // 6. 8-HANDLE INTERACTION & RENDERING
                // ------------------------------------------------------------
                let handle_radius = 6.5;
                let hit_size = 18.0; // Comfortable hit target area for grabbing handles
                let mut handle_drag_occurred = false;

                for &handle in &HandleId::ALL {
                    let handle_pos = handle.position(self.image_rect);
                    let hit_rect = Rect::from_center_size(handle_pos, vec2(hit_size, hit_size));
                    let handle_id = ui.scope_id().with("resize_crop_handle").with(handle);

                    let handle_response = ui.interact(hit_rect, handle_id, Sense::drag());

                    // Set cursor icon on hover or drag
                    if handle_response.hovered() || handle_response.dragged() {
                        ui.ctx().set_cursor_icon(handle.cursor_icon());
                    }

                    // Handle Dragging Logic
                    if handle_response.dragged() {
                        self.active_handle = Some(handle);
                        handle_drag_occurred = true;

                        // Read pointer delta
                        let delta = ui.input(|i| i.pointer.delta());

                        if self.crop_mode {
                            // FEATURE 2: Crop Mode Math
                            let (new_screen, new_uv) =
                                apply_crop(self.image_rect, self.uv_rect, handle, delta);
                            self.image_rect = new_screen;
                            self.uv_rect = new_uv;
                        } else {
                            // FEATURE 1: Resize Mode Math
                            self.image_rect = apply_resize(self.image_rect, handle, delta);
                        }
                    }

                    // Render Handle Circles
                    let is_active = self.active_handle == Some(handle);
                    let is_hovered = handle_response.hovered();

                    let circle_radius = if is_active {
                        handle_radius + 2.0
                    } else if is_hovered {
                        handle_radius + 1.2
                    } else {
                        handle_radius
                    };

                    let fill_color = if self.crop_mode {
                        if is_active || is_hovered {
                            Color32::from_rgb(255, 230, 150)
                        } else {
                            Color32::from_rgb(255, 255, 255)
                        }
                    } else if is_active || is_hovered {
                        Color32::from_rgb(210, 235, 255)
                    } else {
                        Color32::from_rgb(255, 255, 255)
                    };

                    // Draw handle shadow/glow
                    painter.circle_filled(
                        handle_pos,
                        circle_radius + 1.5,
                        Color32::from_rgba_unmultiplied(0, 0, 0, 80),
                    );

                    // Draw handle circle
                    painter.circle_filled(handle_pos, circle_radius, fill_color);
                    painter.circle_stroke(
                        handle_pos,
                        circle_radius,
                        Stroke::new(1.8, accent_color),
                    );
                }

                // Reset active handle when dragging stops
                if !handle_drag_occurred {
                    self.active_handle = None;
                }

                // ------------------------------------------------------------
                // 7. IMAGE PANNING / REPOSITIONING
                // ------------------------------------------------------------
                if !handle_drag_occurred {
                    let body_id = ui.scope_id().with("image_body_drag");
                    let body_response = ui.interact(self.image_rect, body_id, Sense::drag());

                    if body_response.hovered() && !self.crop_mode {
                        ui.ctx().set_cursor_icon(CursorIcon::Grab);
                    }

                    if body_response.dragged() {
                        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                        let delta = ui.input(|i| i.pointer.delta());
                        self.image_rect = self.image_rect.translate(delta);
                    }
                }
            } else {
                // EMPTY STATE: Nothing is displayed except a helpful prompt
                let center = canvas_rect.center();
                painter.text(
                    pos2(center.x, center.y - 12.0),
                    egui::Align2::CENTER_CENTER,
                    "No image loaded",
                    egui::FontId::proportional(22.0),
                    Color32::from_rgb(130, 135, 150),
                );
                painter.text(
                    pos2(center.x, center.y + 18.0),
                    egui::Align2::CENTER_CENTER,
                    "Click 'Browse Image...' to select a PNG or JPG file",
                    egui::FontId::proportional(14.0),
                    Color32::from_rgb(85, 90, 105),
                );
            }
        });
    }
}

// ============================================================================
// 6. MAIN ENTRY POINT
// ============================================================================

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 780.0])
            .with_min_inner_size([850.0, 600.0])
            .with_title("egui Image Resize & Crop Sandbox"),
        ..Default::default()
    };

    eframe::run_native(
        "egui Image Resize & Crop Sandbox",
        options,
        Box::new(|cc| {
            // Install image loaders from egui_extras
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(ImageSandboxApp::new(&cc.egui_ctx)))
        }),
    )
}

// ============================================================================
// 7. UNIT TESTS: MATHEMATICAL VERIFICATION
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_bottom_right_anchor() {
        // Dragging BottomRight should lock TopLeft (min)
        let r = Rect::from_min_max(pos2(100.0, 100.0), pos2(300.0, 200.0));
        let res = apply_resize(r, HandleId::BottomRight, vec2(50.0, 25.0));
        assert_eq!(res.min, pos2(100.0, 100.0), "TopLeft anchor must remain fixed");
        assert_eq!(res.max, pos2(350.0, 225.0), "BottomRight must expand by delta");
    }

    #[test]
    fn test_resize_top_left_anchor() {
        // Dragging TopLeft should lock BottomRight (max)
        let r = Rect::from_min_max(pos2(100.0, 100.0), pos2(300.0, 200.0));
        let res = apply_resize(r, HandleId::TopLeft, vec2(20.0, 15.0));
        assert_eq!(res.max, pos2(300.0, 200.0), "BottomRight anchor must remain fixed");
        assert_eq!(res.min, pos2(120.0, 115.0), "TopLeft must move by delta");
    }

    #[test]
    fn test_resize_clamping_prevents_inversion() {
        // Dragging TopLeft past BottomRight should clamp at MIN_RECT_SIZE
        let r = Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 200.0));
        let res = apply_resize(r, HandleId::TopLeft, vec2(500.0, 500.0));
        assert_eq!(res.max, pos2(200.0, 200.0));
        assert_eq!(res.min, pos2(200.0 - MIN_RECT_SIZE, 200.0 - MIN_RECT_SIZE));
        assert!(res.width() >= MIN_RECT_SIZE);
        assert!(res.height() >= MIN_RECT_SIZE);
    }

    #[test]
    fn test_resize_top_right_anchor() {
        // Dragging TopRight should lock BottomLeft (min.x, max.y)
        let r = Rect::from_min_max(pos2(100.0, 100.0), pos2(300.0, 200.0));
        let res = apply_resize(r, HandleId::TopRight, vec2(10.0, -10.0));
        assert_eq!(res.min.x, 100.0, "min.x (BL) anchor fixed");
        assert_eq!(res.max.y, 200.0, "max.y (BL) anchor fixed");
        assert_eq!(res.max.x, 310.0);
        assert_eq!(res.min.y, 90.0);
    }

    #[test]
    fn test_resize_bottom_left_anchor() {
        // Dragging BottomLeft should lock TopRight (max.x, min.y)
        let r = Rect::from_min_max(pos2(100.0, 100.0), pos2(300.0, 200.0));
        let res = apply_resize(r, HandleId::BottomLeft, vec2(-15.0, 20.0));
        assert_eq!(res.max.x, 300.0, "max.x (TR) anchor fixed");
        assert_eq!(res.min.y, 100.0, "min.y (TR) anchor fixed");
        assert_eq!(res.min.x, 85.0);
        assert_eq!(res.max.y, 220.0);
    }

    #[test]
    fn test_resize_mid_edges() {
        let r = Rect::from_min_max(pos2(100.0, 100.0), pos2(300.0, 200.0));

        // TopMid: anchor bottom
        let tm = apply_resize(r, HandleId::TopMid, vec2(50.0, -30.0));
        assert_eq!(tm.min.x, 100.0, "X min unaffected");
        assert_eq!(tm.max.x, 300.0, "X max unaffected");
        assert_eq!(tm.max.y, 200.0, "Y max anchor locked");
        assert_eq!(tm.min.y, 70.0);

        // RightMid: anchor left
        let rm = apply_resize(r, HandleId::RightMid, vec2(40.0, 50.0));
        assert_eq!(rm.min.x, 100.0, "X min anchor locked");
        assert_eq!(rm.max.x, 340.0);
        assert_eq!(rm.min.y, 100.0, "Y unaffected");
        assert_eq!(rm.max.y, 200.0, "Y unaffected");
    }

    #[test]
    fn test_crop_lockstep_ratio_preservation() {
        let screen = Rect::from_min_max(pos2(100.0, 100.0), pos2(500.0, 400.0)); // W=400, H=300
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));

        let initial_px_per_uv_x = screen.width() / uv.width();
        let initial_px_per_uv_y = screen.height() / uv.height();

        // Crop 80px from the right edge
        let (new_screen, new_uv) = apply_crop(screen, uv, HandleId::RightMid, vec2(-80.0, 0.0));

        // Screen width shrunk by exactly 80px
        assert_eq!(new_screen.min.x, 100.0, "Left edge anchored");
        assert_eq!(new_screen.max.x, 420.0, "Right edge moved in by 80px");
        assert_eq!(new_screen.width(), 320.0);

        // UV coordinate reduced to 320/400 = 0.8
        assert_eq!(new_uv.min.x, 0.0);
        assert!((new_uv.max.x - 0.8).abs() < 1e-4);

        // The pixel scale MUST be identical (no stretching)
        let new_px_per_uv_x = new_screen.width() / new_uv.width();
        assert!(
            (new_px_per_uv_x - initial_px_per_uv_x).abs() < 1e-4,
            "Pixel scale must remain invariant in crop mode"
        );

        let new_px_per_uv_y = new_screen.height() / new_uv.height();
        assert!(
            (new_px_per_uv_y - initial_px_per_uv_y).abs() < 1e-4,
            "Pixel scale Y must remain invariant"
        );
    }

    #[test]
    fn test_crop_left_edge_anchoring() {
        let screen = Rect::from_min_max(pos2(100.0, 100.0), pos2(500.0, 400.0)); // W=400, H=300
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));

        // Crop 50px inward from the left
        let (new_screen, new_uv) = apply_crop(screen, uv, HandleId::LeftMid, vec2(50.0, 0.0));

        // Right edge anchored at 500.0
        assert_eq!(new_screen.max.x, 500.0, "Right edge must remain strictly anchored");
        assert_eq!(new_screen.min.x, 150.0, "Left edge must move by 50px");
        assert_eq!(new_screen.width(), 350.0);

        // UV min should be 50 / 400 = 0.125
        assert!((new_uv.min.x - 0.125).abs() < 1e-4);
        assert_eq!(new_uv.max.x, 1.0);
    }

    #[test]
    fn test_crop_uncrop_clamped_at_zero_and_one() {
        let screen = Rect::from_min_max(pos2(100.0, 100.0), pos2(500.0, 400.0));
        let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));

        // Attempt to drag left edge outward past 0.0
        let (new_screen, new_uv) = apply_crop(screen, uv, HandleId::LeftMid, vec2(-100.0, 0.0));
        assert_eq!(new_uv.min.x, 0.0, "UV min cannot go below 0.0");
        assert_eq!(new_screen.min.x, 100.0, "Screen min clamped at initial position");

        // Attempt to drag right edge outward past 1.0
        let (new_screen2, new_uv2) = apply_crop(screen, uv, HandleId::RightMid, vec2(100.0, 0.0));
        assert_eq!(new_uv2.max.x, 1.0, "UV max cannot exceed 1.0");
        assert_eq!(new_screen2.max.x, 500.0, "Screen max clamped at initial position");
    }
}
