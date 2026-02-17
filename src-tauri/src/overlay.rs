//! Overlay module - Creates a floating overlay window on macOS
//! Supports animated sprite sheets with fall animation sequence
//! Supports fullscreen overlay via LSUIElement agent mode
//! Includes chat functionality with Groq AI integration

#![allow(unexpected_cfgs)]

#[cfg(target_os = "macos")]
#[allow(deprecated)]
use cocoa::appkit::{
    NSBackingStoreType, NSWindowCollectionBehavior,
    NSWindowStyleMask,
};
#[cfg(target_os = "macos")]
#[allow(deprecated)]
use cocoa::base::{id, nil, YES, NO};
#[cfg(target_os = "macos")]
#[allow(deprecated)]
use cocoa::foundation::{NSPoint, NSRect, NSSize, NSString};
#[cfg(target_os = "macos")]
use objc::runtime::{Class, Object, Sel};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};
#[cfg(target_os = "macos")]
use objc::declare::ClassDecl;
#[cfg(target_os = "macos")]
use std::sync::Mutex;

#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[cfg(target_os = "macos")]
#[allow(deprecated)]
struct SafeId(id);
#[cfg(target_os = "macos")]
unsafe impl Send for SafeId {}
#[cfg(target_os = "macos")]
unsafe impl Sync for SafeId {}

#[cfg(target_os = "macos")]
static OVERLAY_PANEL: Mutex<Option<SafeId>> = Mutex::new(None);

#[cfg(target_os = "macos")]
static IMAGE_VIEW: Mutex<Option<SafeId>> = Mutex::new(None);

#[cfg(target_os = "macos")]
static SPEECH_BUBBLE: Mutex<Option<SafeId>> = Mutex::new(None);

// Chat UI elements
#[cfg(target_os = "macos")]
static CHAT_PANEL: Mutex<Option<SafeId>> = Mutex::new(None); // Separate centered panel for chat
#[cfg(target_os = "macos")]
static CHAT_CONTAINER: Mutex<Option<SafeId>> = Mutex::new(None);
#[cfg(target_os = "macos")]
static CHAT_INPUT: Mutex<Option<SafeId>> = Mutex::new(None);
#[cfg(target_os = "macos")]
static SEND_BUTTON: Mutex<Option<SafeId>> = Mutex::new(None);
#[cfg(target_os = "macos")]
static RESPONSE_BOX: Mutex<Option<SafeId>> = Mutex::new(None);
#[cfg(target_os = "macos")]
static THINKING_LABEL: Mutex<Option<SafeId>> = Mutex::new(None);

// Chat state: 0=Idle, 1=InputOpen, 2=Thinking, 3=Responding
#[cfg(target_os = "macos")]
static CHAT_STATE: AtomicUsize = AtomicUsize::new(0);

// Last click timestamp for debouncing (in milliseconds)
#[cfg(target_os = "macos")]
static LAST_CLICK_TIME: AtomicUsize = AtomicUsize::new(0);

// Track if hand cursor is currently shown (for global mouse move handler)
#[cfg(target_os = "macos")]
static CURSOR_IS_HAND: AtomicBool = AtomicBool::new(false);

// Store current response text for typing effect
#[cfg(target_os = "macos")]
static CURRENT_RESPONSE: Mutex<String> = Mutex::new(String::new());
#[cfg(target_os = "macos")]
static RESPONSE_CHAR_INDEX: AtomicUsize = AtomicUsize::new(0);
#[cfg(target_os = "macos")]
static TYPING_RUNNING: AtomicBool = AtomicBool::new(false);

// Separate frame storage for spawn, yawn, and idle animations
#[cfg(target_os = "macos")]
static SPAWN_FRAMES: Mutex<Vec<SafeId>> = Mutex::new(Vec::new());
#[cfg(target_os = "macos")]
static YAWN_FRAMES: Mutex<Vec<SafeId>> = Mutex::new(Vec::new());
#[cfg(target_os = "macos")]
static IDLE_FRAMES: Mutex<Vec<SafeId>> = Mutex::new(Vec::new());

#[cfg(target_os = "macos")]
static OVERLAY_WIDTH: Mutex<f64> = Mutex::new(320.0);
#[cfg(target_os = "macos")]
static OVERLAY_HEIGHT: Mutex<f64> = Mutex::new(320.0);

// Track if screen monitor is running
#[cfg(target_os = "macos")]
static MONITOR_RUNNING: AtomicBool = AtomicBool::new(false);

// Track if animation is running
#[cfg(target_os = "macos")]
static ANIMATION_RUNNING: AtomicBool = AtomicBool::new(false);

// Track if frames are loaded (animation waits for this)
#[cfg(target_os = "macos")]
static FRAMES_LOADED: AtomicBool = AtomicBool::new(false);

// Current animation frame index
#[cfg(target_os = "macos")]
static CURRENT_FRAME: AtomicUsize = AtomicUsize::new(0);

// Animation phase: 0=spawn once, 1=yawn once, 2=idle loop
#[cfg(target_os = "macos")]
static ANIMATION_PHASE: AtomicUsize = AtomicUsize::new(0);

// Track if click monitor is running
#[cfg(target_os = "macos")]
static CLICK_MONITOR_RUNNING: AtomicBool = AtomicBool::new(false);

// Window levels - use maximum level to appear above fullscreen apps
#[cfg(target_os = "macos")]
const KCGMAXIMUM_WINDOW_LEVEL: i64 = 2147483631;

// Groq API key - should be set via environment variable or config
#[cfg(target_os = "macos")]
static GROQ_API_KEY: Mutex<String> = Mutex::new(String::new());

/// Helper to create NSColor from RGB values (0-255)
#[cfg(target_os = "macos")]
#[allow(deprecated)]
unsafe fn create_color(r: f64, g: f64, b: f64, a: f64) -> id {
    let color_class = Class::get("NSColor").unwrap();
    msg_send![color_class, colorWithRed: r/255.0 green: g/255.0 blue: b/255.0 alpha: a]
}

/// Helper to load custom font
#[cfg(target_os = "macos")]
#[allow(deprecated)]
unsafe fn load_chicle_font(size: f64) -> id {
    use std::sync::Once;
    static FONT_REGISTERED: Once = Once::new();

    let exe_path = std::env::current_exe().unwrap();
    let resources_path = exe_path.parent().unwrap()
        .join("../Resources/resources/fonts/Chicle-Regular.ttf");
    let dev_path = std::path::PathBuf::from("resources/fonts/Chicle-Regular.ttf");
    let fallback_path = std::path::PathBuf::from("src-tauri/resources/fonts/Chicle-Regular.ttf");

    let font_path = if resources_path.exists() {
        resources_path
    } else if dev_path.exists() {
        dev_path
    } else if fallback_path.exists() {
        fallback_path
    } else {
        log::warn!("Chicle font file not found in any path");
        return msg_send![Class::get("NSFont").unwrap(), systemFontOfSize: size];
    };

    log::info!("Trying to load font from: {:?}", font_path);

    // Register font only once using NSFontManager
    FONT_REGISTERED.call_once(|| {
        let path_str = font_path.to_str().unwrap();
        let ns_path = NSString::alloc(nil).init_str(path_str);
        let url: id = msg_send![Class::get("NSURL").unwrap(), fileURLWithPath: ns_path];

        // Use CTFontManagerRegisterFontsForURL C function
        #[link(name = "CoreText", kind = "framework")]
        extern "C" {
            fn CTFontManagerRegisterFontsForURL(fontURL: id, scope: u32, error: *mut id) -> bool;
        }

        let result = CTFontManagerRegisterFontsForURL(url, 1, std::ptr::null_mut());
        log::info!("Font registration result: {}", result);
    });

    // Try to load Chicle font by name
    let font_name = NSString::alloc(nil).init_str("Chicle");
    let font: id = msg_send![Class::get("NSFont").unwrap(), fontWithName: font_name size: size];

    if font != nil {
        log::info!("Successfully loaded Chicle font at size {}", size);
        font
    } else {
        // Try with -Regular suffix
        let font_name2 = NSString::alloc(nil).init_str("Chicle-Regular");
        let font2: id = msg_send![Class::get("NSFont").unwrap(), fontWithName: font_name2 size: size];

        if font2 != nil {
            log::info!("Successfully loaded Chicle-Regular font at size {}", size);
            font2
        } else {
            // Fallback to system font
            log::warn!("Could not load Chicle font, using system font");
            msg_send![Class::get("NSFont").unwrap(), systemFontOfSize: size]
        }
    }
}

/// Set the Groq API key
#[cfg(target_os = "macos")]
pub fn set_groq_api_key(key: &str) {
    let mut guard = GROQ_API_KEY.lock().unwrap();
    *guard = key.to_string();
}

/// Creates a custom NSPanel subclass that can become key window for text input
#[cfg(target_os = "macos")]
fn get_or_create_key_panel_class() -> &'static Class {
    static REGISTER: std::sync::Once = std::sync::Once::new();

    REGISTER.call_once(|| {
        let superclass = Class::get("NSPanel").unwrap();
        let mut decl = ClassDecl::new("KeyablePanel", superclass).unwrap();

        // Override canBecomeKeyWindow to return YES
        extern "C" fn can_become_key_window(_this: &Object, _sel: Sel) -> bool {
            true
        }

        unsafe {
            decl.add_method(
                sel!(canBecomeKeyWindow),
                can_become_key_window as extern "C" fn(&Object, Sel) -> bool,
            );
        }

        decl.register();
    });

    Class::get("KeyablePanel").unwrap()
}

/// Creates a custom NSView subclass that changes cursor on mouse enter/exit
#[cfg(target_os = "macos")]
/// Create a custom NSImageView subclass with hand cursor on hover
fn get_or_create_cursor_image_view_class() -> &'static Class {
    static REGISTER: std::sync::Once = std::sync::Once::new();

    REGISTER.call_once(|| {
        let superclass = Class::get("NSImageView").unwrap();
        let mut decl = ClassDecl::new("CursorImageView", superclass).unwrap();

        // Override mouseEntered to show hand cursor
        extern "C" fn mouse_entered(_this: &Object, _sel: Sel, _event: id) {
            unsafe {
                let hand_cursor: id = msg_send![Class::get("NSCursor").unwrap(), pointingHandCursor];
                let _: () = msg_send![hand_cursor, push];
            }
        }

        // Override mouseExited to restore arrow cursor
        extern "C" fn mouse_exited(_this: &Object, _sel: Sel, _event: id) {
            unsafe {
                let _: () = msg_send![Class::get("NSCursor").unwrap(), pop];
            }
        }

        // Override updateTrackingAreas to add tracking area on view setup
        extern "C" fn update_tracking_areas(this: &Object, _sel: Sel) {
            unsafe {
                // Call super
                let superclass = Class::get("NSImageView").unwrap();
                let _: () = msg_send![super(this, superclass), updateTrackingAreas];

                // Remove old tracking areas
                let tracking_areas: id = msg_send![this, trackingAreas];
                let count: usize = msg_send![tracking_areas, count];
                for i in 0..count {
                    let area: id = msg_send![tracking_areas, objectAtIndex: i];
                    let _: () = msg_send![this, removeTrackingArea: area];
                }

                // Add new tracking area
                let bounds: NSRect = msg_send![this, bounds];
                // NSTrackingMouseEnteredAndExited | NSTrackingActiveAlways | NSTrackingInVisibleRect
                let options: u64 = 0x01 | 0x80 | 0x200;
                let tracking_area: id = msg_send![Class::get("NSTrackingArea").unwrap(), alloc];
                let tracking_area: id = msg_send![tracking_area,
                    initWithRect: bounds
                    options: options
                    owner: this
                    userInfo: nil
                ];
                let _: () = msg_send![this, addTrackingArea: tracking_area];
            }
        }

        unsafe {
            decl.add_method(
                sel!(mouseEntered:),
                mouse_entered as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(mouseExited:),
                mouse_exited as extern "C" fn(&Object, Sel, id),
            );
            decl.add_method(
                sel!(updateTrackingAreas),
                update_tracking_areas as extern "C" fn(&Object, Sel),
            );
        }

        decl.register();
    });

    Class::get("CursorImageView").unwrap()
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn create_overlay(width: f64, height: f64) {
    eprintln!("[DEBUG] create_overlay called with width={}, height={}", width, height);
    log::info!("create_overlay called with width={}, height={}", width, height);

    // Store dimensions for later use
    {
        let mut w = OVERLAY_WIDTH.lock().unwrap();
        *w = width;
        let mut h = OVERLAY_HEIGHT.lock().unwrap();
        *h = height;
    }

    unsafe {
        // Set application to accessory mode (no dock icon, can overlay fullscreen)
        // NSApplicationActivationPolicyAccessory = 1
        let app: id = msg_send![Class::get("NSApplication").unwrap(), sharedApplication];
        let _: () = msg_send![app, setActivationPolicy: 1_i64];

        // Use custom panel class that can become key window
        let panel_class = get_or_create_key_panel_class();

        // Get the screen with the mouse cursor (active screen)
        let mouse_location: NSPoint = msg_send![Class::get("NSEvent").unwrap(), mouseLocation];
        let screens: id = msg_send![Class::get("NSScreen").unwrap(), screens];
        let screen_count: usize = msg_send![screens, count];
        let mut screen: id = msg_send![Class::get("NSScreen").unwrap(), mainScreen];

        // Find the screen containing the mouse
        for i in 0..screen_count {
            let s: id = msg_send![screens, objectAtIndex: i];
            let s_frame: NSRect = msg_send![s, frame];
            if mouse_location.x >= s_frame.origin.x
                && mouse_location.x < s_frame.origin.x + s_frame.size.width
                && mouse_location.y >= s_frame.origin.y
                && mouse_location.y < s_frame.origin.y + s_frame.size.height {
                screen = s;
                break;
            }
        }

        let screen_frame: NSRect = msg_send![screen, frame];

        // Position bottom-right, moved 50px right (closer to edge) and 60px down
        let x = screen_frame.origin.x + screen_frame.size.width - width + 40.0;
        let y = screen_frame.origin.y - 60.0; // 60px down (below screen edge)
        let frame = NSRect::new(NSPoint::new(x, y), NSSize::new(width, height));

        let style = NSWindowStyleMask::NSBorderlessWindowMask;
        let panel: id = msg_send![panel_class, alloc];
        let panel: id = msg_send![panel, initWithContentRect:frame
                                        styleMask:style
                                        backing:NSBackingStoreType::NSBackingStoreBuffered
                                        defer:NO];

        // Use maximum window level to appear above everything including fullscreen
        let _: () = msg_send![panel, setLevel: KCGMAXIMUM_WINDOW_LEVEL];

        // Enhanced collection behavior for fullscreen support
        // NSWindowCollectionBehaviorMoveToActiveSpace = 1 << 1 (2)
        let behavior = NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle
            | NSWindowCollectionBehavior::NSWindowCollectionBehaviorTransient;
        let _: () = msg_send![panel, setCollectionBehavior: behavior];

        let _: () = msg_send![panel, setOpaque: NO];
        let clear_color: id = msg_send![Class::get("NSColor").unwrap(), clearColor];
        let _: () = msg_send![panel, setBackgroundColor: clear_color];
        let _: () = msg_send![panel, setHasShadow: NO]; // No shadow - clean oval only
        let _: () = msg_send![panel, setIgnoresMouseEvents: YES];
        let _: () = msg_send![panel, setFloatingPanel: YES];
        let _: () = msg_send![panel, setHidesOnDeactivate: NO];
        let _: () = msg_send![panel, setWorksWhenModal: YES];
        let _: () = msg_send![panel, setCanHide: NO];
        let _: () = msg_send![panel, setReleasedWhenClosed: NO];
        // Additional settings for screen recording level
        let _: () = msg_send![panel, setStyleMask: 0_i64]; // Non-activating
        let _: () = msg_send![panel, setAnimationBehavior: 0_i64]; // None

        // Transparent container view - just for the red panda, no background
        let content_frame = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width, height));
        let container_view: id = msg_send![Class::get("NSView").unwrap(), alloc];
        let container_view: id = msg_send![container_view, initWithFrame: content_frame];
        let _: () = msg_send![container_view, setWantsLayer: YES];

        // Load idle and yawn frames from GIFs (512x512 source)
        // Display at 320x320 logical pixels with high quality scaling
        let img_width: f64 = 320.0;
        let img_height: f64 = 320.0;
        let source_size: f64 = 512.0; // Source frames are 512x512
        let exe_path = std::env::current_exe().unwrap();

        // Helper to load a frame
        let load_frame = |prefix: &str, i: usize| -> Option<SafeId> {
            let frame_name = format!("{}_frame{}.png", prefix, i);
            let resources_path = exe_path.parent().unwrap()
                .join(format!("../Resources/resources/frames/{}", frame_name));
            let dev_path = std::path::PathBuf::from(format!("resources/frames/{}", frame_name));
            let fallback_path = std::path::PathBuf::from(format!("src-tauri/resources/frames/{}", frame_name));

            let image_path = if resources_path.exists() {
                resources_path
            } else if dev_path.exists() {
                dev_path
            } else {
                fallback_path
            };

            let path_str = image_path.to_str().unwrap();
            let ns_path = NSString::alloc(nil).init_str(path_str);
            let source_image: id = msg_send![Class::get("NSImage").unwrap(), alloc];
            let source_image: id = msg_send![source_image, initWithContentsOfFile: ns_path];

            if source_image != nil {
                let target_size = NSSize::new(img_width, img_height);
                let scaled_image: id = msg_send![Class::get("NSImage").unwrap(), alloc];
                let scaled_image: id = msg_send![scaled_image, initWithSize: target_size];
                let _: () = msg_send![scaled_image, lockFocus];
                let context: id = msg_send![Class::get("NSGraphicsContext").unwrap(), currentContext];
                if context != nil {
                    // NSImageInterpolationHigh (3) for high quality scaling from 512 to 160
                    let _: () = msg_send![context, setImageInterpolation: 3_i64];
                }
                let source_rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(source_size, source_size));
                let dest_rect = NSRect::new(NSPoint::new(0.0, 0.0), target_size);
                let _: () = msg_send![source_image, drawInRect: dest_rect fromRect: source_rect operation: 1_i64 fraction: 1.0_f64];
                let _: () = msg_send![scaled_image, unlockFocus];
                Some(SafeId(scaled_image))
            } else {
                log::warn!("Failed to load {} frame {}", prefix, i);
                None
            }
        };

        // Mark frames as not loaded yet
        FRAMES_LOADED.store(false, Ordering::SeqCst);

        // Load spawn frames (dynamic count)
        let mut spawn_frames: Vec<SafeId> = Vec::new();
        for i in 1..=100 {
            if let Some(frame) = load_frame("spawn", i) {
                spawn_frames.push(frame);
            } else {
                break; // Stop when no more frames found
            }
        }
        log::info!("Loaded {} spawn frames", spawn_frames.len());

        // Load yawn frames (dynamic count)
        let mut yawn_frames: Vec<SafeId> = Vec::new();
        for i in 1..=100 {
            if let Some(frame) = load_frame("yawn", i) {
                yawn_frames.push(frame);
            } else {
                break; // Stop when no more frames found
            }
        }
        log::info!("Loaded {} yawn frames", yawn_frames.len());

        // Load idle frames (dynamic count)
        let mut idle_frames: Vec<SafeId> = Vec::new();
        for i in 1..=100 {
            if let Some(frame) = load_frame("idle", i) {
                idle_frames.push(frame);
            } else {
                break; // Stop when no more frames found
            }
        }
        log::info!("Loaded {} idle frames", idle_frames.len());

        // Store frames
        {
            let mut guard = SPAWN_FRAMES.lock().unwrap();
            *guard = spawn_frames;
        }
        {
            let mut guard = YAWN_FRAMES.lock().unwrap();
            *guard = yawn_frames;
        }
        {
            let mut guard = IDLE_FRAMES.lock().unwrap();
            *guard = idle_frames;
        }
        CURRENT_FRAME.store(0, Ordering::SeqCst);
        ANIMATION_PHASE.store(0, Ordering::SeqCst); // Start with spawn phase

        // Create NSImageView for the cat - positioned at bottom of the view
        // This leaves room above the cat for chat elements
        let img_x = (width - img_width) / 2.0;
        let img_y = 0.0; // At the bottom of the panel (macOS y=0 is at bottom)
        let img_frame = NSRect::new(
            NSPoint::new(img_x, img_y),
            NSSize::new(img_width, img_height),
        );

        // Use custom CursorImageView class for hand cursor on hover
        let cursor_image_class = get_or_create_cursor_image_view_class();
        let image_view: id = msg_send![cursor_image_class, alloc];
        let image_view: id = msg_send![image_view, initWithFrame: img_frame];
        // Use NSImageScaleNone (0) - images are pre-scaled with high quality
        let _: () = msg_send![image_view, setImageScaling: 0_i64];

        // Set first spawn frame (animation will start from here)
        {
            let frames_guard = SPAWN_FRAMES.lock().unwrap();
            if !frames_guard.is_empty() {
                let _: () = msg_send![image_view, setImage: frames_guard[0].0];
            }
        }

        let _: () = msg_send![container_view, addSubview: image_view];

        // Trigger tracking area setup
        let _: () = msg_send![image_view, updateTrackingAreas];

        // Store image view reference for animation
        {
            let mut iv_guard = IMAGE_VIEW.lock().unwrap();
            *iv_guard = Some(SafeId(image_view));
        }

        // Create speech bubble above the cat
        let bubble_width = 200.0;
        let bubble_height = 50.0;
        let bubble_x = (width - bubble_width) / 2.0;
        let bubble_y = img_y + img_height - 20.0; // Position above the cat
        let bubble_frame = NSRect::new(
            NSPoint::new(bubble_x, bubble_y),
            NSSize::new(bubble_width, bubble_height),
        );

        let text_field: id = msg_send![Class::get("NSTextField").unwrap(), alloc];
        let text_field: id = msg_send![text_field, initWithFrame: bubble_frame];
        let _: () = msg_send![text_field, setEditable: NO];
        let _: () = msg_send![text_field, setBordered: NO];
        let _: () = msg_send![text_field, setDrawsBackground: YES];
        let _: () = msg_send![text_field, setWantsLayer: YES];

        // Set background color (white with rounded corners)
        let white_color: id = msg_send![Class::get("NSColor").unwrap(), whiteColor];
        let _: () = msg_send![text_field, setBackgroundColor: white_color];

        // Set text color (dark gray)
        let text_color: id = msg_send![Class::get("NSColor").unwrap(), blackColor];
        let _: () = msg_send![text_field, setTextColor: text_color];

        // Center text
        let _: () = msg_send![text_field, setAlignment: 1_i64]; // NSTextAlignmentCenter = 1

        // Set font
        let font: id = msg_send![Class::get("NSFont").unwrap(), systemFontOfSize: 14.0_f64];
        let _: () = msg_send![text_field, setFont: font];

        // Round corners via layer
        let layer: id = msg_send![text_field, layer];
        let _: () = msg_send![layer, setCornerRadius: 12.0_f64];
        let _: () = msg_send![layer, setMasksToBounds: YES];

        // Initially hidden
        let _: () = msg_send![text_field, setHidden: YES];

        let _: () = msg_send![container_view, addSubview: text_field];

        // Store speech bubble reference
        {
            let mut sb_guard = SPEECH_BUBBLE.lock().unwrap();
            *sb_guard = Some(SafeId(text_field));
        }

        // ========== CHAT UI CREATION ==========
        // Get screen dimensions for center positioning
        let screen: id = msg_send![Class::get("NSScreen").unwrap(), mainScreen];
        let screen_frame: NSRect = msg_send![screen, frame];
        let screen_center_x = screen_frame.size.width / 2.0;
        let screen_center_y = screen_frame.size.height / 2.0;

        // Chat box - sleek modern design positioned at CENTER OF SCREEN
        let chat_box_width = 320.0;
        let chat_box_height = 56.0;
        // Position in screen coordinates (will be converted when showing)
        // Store these for later use
        let _screen_chat_x = screen_center_x - chat_box_width / 2.0;
        let _screen_chat_y = screen_center_y - chat_box_height / 2.0;

        // For now, position relative to panel (will reposition when showing)
        let chat_box_x = (width - chat_box_width) / 2.0;
        let chat_box_y = img_y + img_height + 20.0;
        let chat_frame = NSRect::new(
            NSPoint::new(chat_box_x, chat_box_y),
            NSSize::new(chat_box_width, chat_box_height),
        );

        // Create chat container view
        let chat_container: id = msg_send![Class::get("NSView").unwrap(), alloc];
        let chat_container: id = msg_send![chat_container, initWithFrame: chat_frame];
        let _: () = msg_send![chat_container, setWantsLayer: YES];

        // Style: WHITE background with BLACK text, sleek curved border
        let chat_layer: id = msg_send![chat_container, layer];
        let _: () = msg_send![chat_layer, setCornerRadius: 28.0_f64]; // Pill shape
        let _: () = msg_send![chat_layer, setMasksToBounds: NO];

        // White background
        let white: id = msg_send![Class::get("NSColor").unwrap(), whiteColor];
        let cg_white: id = msg_send![white, CGColor];
        let _: () = msg_send![chat_layer, setBackgroundColor: cg_white];

        // Subtle gray border for sleek look
        let border_color = create_color(220.0, 220.0, 220.0, 1.0);
        let cg_border_color: id = msg_send![border_color, CGColor];
        let _: () = msg_send![chat_layer, setBorderColor: cg_border_color];
        let _: () = msg_send![chat_layer, setBorderWidth: 1.0_f64];

        // Elegant shadow for depth
        let _: () = msg_send![chat_layer, setShadowOpacity: 0.15_f32];
        let _: () = msg_send![chat_layer, setShadowRadius: 20.0_f64];
        let _: () = msg_send![chat_layer, setShadowOffset: NSSize::new(0.0, -5.0)];
        let black: id = msg_send![Class::get("NSColor").unwrap(), blackColor];
        let cg_black: id = msg_send![black, CGColor];
        let _: () = msg_send![chat_layer, setShadowColor: cg_black];

        // Create text input field (sleek inline style)
        let input_padding = 20.0;
        let btn_space = 50.0;
        let input_width = chat_box_width - input_padding * 2.0 - btn_space;
        let input_height = 30.0;
        let input_x = input_padding;
        let input_y = (chat_box_height - input_height) / 2.0;
        let input_frame = NSRect::new(
            NSPoint::new(input_x, input_y),
            NSSize::new(input_width, input_height),
        );

        let chat_input: id = msg_send![Class::get("NSTextField").unwrap(), alloc];
        let chat_input: id = msg_send![chat_input, initWithFrame: input_frame];
        let _: () = msg_send![chat_input, setEditable: YES];
        let _: () = msg_send![chat_input, setSelectable: YES];
        let _: () = msg_send![chat_input, setBordered: NO];
        let _: () = msg_send![chat_input, setDrawsBackground: NO];
        let _: () = msg_send![chat_input, setWantsLayer: YES];
        let _: () = msg_send![chat_input, setFocusRingType: 0_i64];
        let _: () = msg_send![chat_input, setAllowsEditingTextAttributes: NO];

        // BLACK text color
        let black_text: id = msg_send![Class::get("NSColor").unwrap(), blackColor];
        let _: () = msg_send![chat_input, setTextColor: black_text];

        // Set placeholder with gray color
        let placeholder_str = NSString::alloc(nil).init_str("Ask me anything...");
        let placeholder_color = create_color(150.0, 150.0, 150.0, 1.0);
        let placeholder_font: id = msg_send![Class::get("NSFont").unwrap(), systemFontOfSize: 15.0_f64 weight: 0.0_f64];

        let foreground_color_key = NSString::alloc(nil).init_str("NSColor");
        let font_key = NSString::alloc(nil).init_str("NSFont");

        let keys: [id; 2] = [foreground_color_key, font_key];
        let objects: [id; 2] = [placeholder_color, placeholder_font];
        let placeholder_dict: id = msg_send![Class::get("NSDictionary").unwrap(),
            dictionaryWithObjects: objects.as_ptr()
            forKeys: keys.as_ptr()
            count: 2_usize
        ];
        let attributed_placeholder: id = msg_send![Class::get("NSAttributedString").unwrap(), alloc];
        let attributed_placeholder: id = msg_send![attributed_placeholder, initWithString:placeholder_str attributes:placeholder_dict];
        let _: () = msg_send![chat_input, setPlaceholderAttributedString: attributed_placeholder];

        // Modern system font for input
        let input_font: id = msg_send![Class::get("NSFont").unwrap(), systemFontOfSize: 15.0_f64 weight: 0.0_f64];
        let _: () = msg_send![chat_input, setFont: input_font];

        let _: () = msg_send![chat_container, addSubview: chat_input];

        // Create send button (pill-shaped on right side)
        let btn_width = 40.0;
        let btn_height = 40.0;
        let btn_x = chat_box_width - btn_width - 8.0;
        let btn_y = (chat_box_height - btn_height) / 2.0;
        let btn_frame = NSRect::new(
            NSPoint::new(btn_x, btn_y),
            NSSize::new(btn_width, btn_height),
        );

        let send_btn: id = msg_send![Class::get("NSButton").unwrap(), alloc];
        let send_btn: id = msg_send![send_btn, initWithFrame: btn_frame];
        let _: () = msg_send![send_btn, setWantsLayer: YES];
        let _: () = msg_send![send_btn, setBordered: NO];
        let _: () = msg_send![send_btn, setTitle: NSString::alloc(nil).init_str("↑")];

        // Set font for send icon (bold arrow)
        let send_font: id = msg_send![Class::get("NSFont").unwrap(), boldSystemFontOfSize: 18.0_f64];
        let _: () = msg_send![send_btn, setFont: send_font];

        // Sleek black circular button
        let btn_layer: id = msg_send![send_btn, layer];
        let _: () = msg_send![btn_layer, setCornerRadius: 20.0_f64];
        let black_bg: id = msg_send![Class::get("NSColor").unwrap(), blackColor];
        let cg_black_bg: id = msg_send![black_bg, CGColor];
        let _: () = msg_send![btn_layer, setBackgroundColor: cg_black_bg];

        // White text on black button
        let white_text: id = msg_send![Class::get("NSColor").unwrap(), whiteColor];
        // Use attributed string for button title
        let btn_title = NSString::alloc(nil).init_str("↑");
        let btn_font: id = msg_send![Class::get("NSFont").unwrap(), boldSystemFontOfSize: 18.0_f64];
        let btn_keys: [id; 2] = [foreground_color_key, font_key];
        let btn_objects: [id; 2] = [white_text, btn_font];
        let btn_dict: id = msg_send![Class::get("NSDictionary").unwrap(),
            dictionaryWithObjects: btn_objects.as_ptr()
            forKeys: btn_keys.as_ptr()
            count: 2_usize
        ];
        let btn_attr_title: id = msg_send![Class::get("NSAttributedString").unwrap(), alloc];
        let btn_attr_title: id = msg_send![btn_attr_title, initWithString:btn_title attributes:btn_dict];
        let _: () = msg_send![send_btn, setAttributedTitle: btn_attr_title];

        let _: () = msg_send![chat_container, addSubview: send_btn];

        // Initially hidden
        let _: () = msg_send![chat_container, setHidden: YES];
        let _: () = msg_send![chat_container, setAlphaValue: 0.0_f64];

        let _: () = msg_send![container_view, addSubview: chat_container];

        // Store chat UI references
        {
            let mut guard = CHAT_CONTAINER.lock().unwrap();
            *guard = Some(SafeId(chat_container));
        }
        {
            let mut guard = CHAT_INPUT.lock().unwrap();
            *guard = Some(SafeId(chat_input));
        }
        {
            let mut guard = SEND_BUTTON.lock().unwrap();
            *guard = Some(SafeId(send_btn));
        }

        // Create "Thinking..." label
        let thinking_width = 120.0;
        let thinking_height = 35.0;
        let thinking_x = (width - thinking_width) / 2.0;
        let thinking_y = img_y + img_height + 10.0;
        let thinking_frame = NSRect::new(
            NSPoint::new(thinking_x, thinking_y),
            NSSize::new(thinking_width, thinking_height),
        );

        let thinking_label: id = msg_send![Class::get("NSTextField").unwrap(), alloc];
        let thinking_label: id = msg_send![thinking_label, initWithFrame: thinking_frame];
        let _: () = msg_send![thinking_label, setEditable: NO];
        let _: () = msg_send![thinking_label, setBordered: NO];
        let _: () = msg_send![thinking_label, setDrawsBackground: NO];
        let _: () = msg_send![thinking_label, setWantsLayer: YES];
        let _: () = msg_send![thinking_label, setAlignment: 1_i64]; // Center

        // White text with black stroke effect (use shadow for stroke effect)
        let white: id = msg_send![Class::get("NSColor").unwrap(), whiteColor];
        let _: () = msg_send![thinking_label, setTextColor: white];
        let thinking_font = load_chicle_font(26.0);
        let _: () = msg_send![thinking_label, setFont: thinking_font];
        let _: () = msg_send![thinking_label, setStringValue: NSString::alloc(nil).init_str("Thinking...")];

        // Add shadow for stroke effect
        let thinking_layer: id = msg_send![thinking_label, layer];
        let shadow: id = msg_send![Class::get("NSShadow").unwrap(), alloc];
        let shadow: id = msg_send![shadow, init];
        let black: id = msg_send![Class::get("NSColor").unwrap(), blackColor];
        let _: () = msg_send![shadow, setShadowColor: black];
        let _: () = msg_send![shadow, setShadowBlurRadius: 2.0_f64];
        let _: () = msg_send![shadow, setShadowOffset: NSSize::new(0.0, 0.0)];
        let _: () = msg_send![thinking_label, setShadow: shadow];
        let _: () = msg_send![thinking_layer, setShadowOpacity: 1.0_f32];
        let _: () = msg_send![thinking_layer, setShadowRadius: 1.5_f64];

        // Initially hidden
        let _: () = msg_send![thinking_label, setHidden: YES];
        let _: () = msg_send![thinking_label, setAlphaValue: 0.0_f64];

        let _: () = msg_send![container_view, addSubview: thinking_label];

        {
            let mut guard = THINKING_LABEL.lock().unwrap();
            *guard = Some(SafeId(thinking_label));
        }

        // Create response box (similar to chat input but larger, for showing response)
        let response_width = 300.0;
        let response_height = 120.0;
        let response_x = (width - response_width) / 2.0;
        let response_y = img_y + img_height + 10.0;
        let response_frame = NSRect::new(
            NSPoint::new(response_x, response_y),
            NSSize::new(response_width, response_height),
        );

        let response_box: id = msg_send![Class::get("NSTextField").unwrap(), alloc];
        let response_box: id = msg_send![response_box, initWithFrame: response_frame];
        let _: () = msg_send![response_box, setEditable: NO];
        let _: () = msg_send![response_box, setBordered: NO];
        let _: () = msg_send![response_box, setDrawsBackground: YES];
        let _: () = msg_send![response_box, setWantsLayer: YES];

        // Style response box - white background with black text
        let response_layer: id = msg_send![response_box, layer];
        let _: () = msg_send![response_layer, setCornerRadius: 20.0_f64];
        let _: () = msg_send![response_layer, setMasksToBounds: NO];

        let white: id = msg_send![Class::get("NSColor").unwrap(), whiteColor];
        let _: () = msg_send![response_box, setBackgroundColor: white];
        let cg_white: id = msg_send![white, CGColor];
        let _: () = msg_send![response_layer, setBackgroundColor: cg_white];

        // Subtle border
        let resp_border_color = create_color(230.0, 230.0, 230.0, 1.0);
        let cg_resp_border: id = msg_send![resp_border_color, CGColor];
        let _: () = msg_send![response_layer, setBorderColor: cg_resp_border];
        let _: () = msg_send![response_layer, setBorderWidth: 1.0_f64];

        // Shadow
        let _: () = msg_send![response_layer, setShadowOpacity: 0.15_f32];
        let _: () = msg_send![response_layer, setShadowRadius: 15.0_f64];
        let _: () = msg_send![response_layer, setShadowOffset: NSSize::new(0.0, -5.0)];
        let black_shadow: id = msg_send![Class::get("NSColor").unwrap(), blackColor];
        let cg_black_shadow: id = msg_send![black_shadow, CGColor];
        let _: () = msg_send![response_layer, setShadowColor: cg_black_shadow];

        let black_text: id = msg_send![Class::get("NSColor").unwrap(), blackColor];
        let _: () = msg_send![response_box, setTextColor: black_text];
        let response_font: id = msg_send![Class::get("NSFont").unwrap(), systemFontOfSize: 15.0_f64 weight: 0.0_f64];
        let _: () = msg_send![response_box, setFont: response_font];
        let _: () = msg_send![response_box, setAlignment: 0_i64]; // Left align

        // Enable word wrapping
        let cell: id = msg_send![response_box, cell];
        let _: () = msg_send![cell, setWraps: YES];
        let _: () = msg_send![cell, setLineBreakMode: 0_i64]; // NSLineBreakByWordWrapping

        // Initially hidden
        let _: () = msg_send![response_box, setHidden: YES];
        let _: () = msg_send![response_box, setAlphaValue: 0.0_f64];

        let _: () = msg_send![container_view, addSubview: response_box];

        {
            let mut guard = RESPONSE_BOX.lock().unwrap();
            *guard = Some(SafeId(response_box));
        }

        // ========== END CHAT UI CREATION ==========

        let _: () = msg_send![panel, setContentView: container_view];
        let _: () = msg_send![panel, orderFrontRegardless];

        let mut guard = OVERLAY_PANEL.lock().unwrap();
        *guard = Some(SafeId(panel));
    }

    // Start animation thread (it will wait for FRAMES_LOADED)
    start_animation();

    // Start click monitor
    start_click_monitor();

    // Now mark frames as loaded - this triggers the animation to actually start
    // Small delay to ensure the panel and first frame are visible
    std::thread::sleep(std::time::Duration::from_millis(50));
    FRAMES_LOADED.store(true, Ordering::SeqCst);
    log::info!("All frames loaded and displayed, animation starting");
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn show_overlay() {
    let guard = OVERLAY_PANEL.lock().unwrap();
    if let Some(ref safe_panel) = *guard {
        unsafe {
            let _: () = msg_send![safe_panel.0, orderFrontRegardless];
        }
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn hide_overlay() {
    let guard = OVERLAY_PANEL.lock().unwrap();
    if let Some(ref safe_panel) = *guard {
        unsafe {
            let _: () = msg_send![safe_panel.0, orderOut: nil];
        }
    }
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn close_overlay() {
    // Stop animation first
    stop_animation();

    let mut guard = OVERLAY_PANEL.lock().unwrap();
    if let Some(ref safe_panel) = *guard {
        unsafe {
            let _: () = msg_send![safe_panel.0, close];
        }
    }
    *guard = None;

    // Clear image view and frames
    {
        let mut iv_guard = IMAGE_VIEW.lock().unwrap();
        *iv_guard = None;
    }
    {
        let mut guard = SPAWN_FRAMES.lock().unwrap();
        guard.clear();
    }
    {
        let mut guard = YAWN_FRAMES.lock().unwrap();
        guard.clear();
    }
    {
        let mut guard = IDLE_FRAMES.lock().unwrap();
        guard.clear();
    }
    FRAMES_LOADED.store(false, Ordering::SeqCst);
}

#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn is_visible() -> bool {
    let guard = OVERLAY_PANEL.lock().unwrap();
    if let Some(ref safe_panel) = *guard {
        unsafe { msg_send![safe_panel.0, isVisible] }
    } else {
        false
    }
}

/// Move the overlay to the screen where the mouse cursor is located
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn move_to_active_screen() {
    let guard = OVERLAY_PANEL.lock().unwrap();
    if let Some(ref safe_panel) = *guard {
        let width = *OVERLAY_WIDTH.lock().unwrap();
        let _height = *OVERLAY_HEIGHT.lock().unwrap();

        unsafe {
            // Get mouse location to find active screen
            let mouse_location: NSPoint = msg_send![Class::get("NSEvent").unwrap(), mouseLocation];
            let screens: id = msg_send![Class::get("NSScreen").unwrap(), screens];
            let screen_count: usize = msg_send![screens, count];
            let mut screen: id = msg_send![Class::get("NSScreen").unwrap(), mainScreen];

            // Find the screen containing the mouse
            for i in 0..screen_count {
                let s: id = msg_send![screens, objectAtIndex: i];
                let s_frame: NSRect = msg_send![s, frame];
                if mouse_location.x >= s_frame.origin.x
                    && mouse_location.x < s_frame.origin.x + s_frame.size.width
                    && mouse_location.y >= s_frame.origin.y
                    && mouse_location.y < s_frame.origin.y + s_frame.size.height {
                    screen = s;
                    break;
                }
            }

            let screen_frame: NSRect = msg_send![screen, frame];

            // Position bottom-right, moved 50px right (closer to edge) and 60px down
            let x = screen_frame.origin.x + screen_frame.size.width - width + 40.0;
            let y = screen_frame.origin.y - 60.0; // 60px down (below screen edge)
            let new_origin = NSPoint::new(x, y);

            let _: () = msg_send![safe_panel.0, setFrameOrigin: new_origin];

            // Ensure it's still on top
            let _: () = msg_send![safe_panel.0, setLevel: KCGMAXIMUM_WINDOW_LEVEL];
            let _: () = msg_send![safe_panel.0, orderFrontRegardless];
        }
    }
}

/// Start monitoring for screen/app changes to keep overlay on active screen
#[cfg(target_os = "macos")]
pub fn start_screen_monitor() {
    if MONITOR_RUNNING.swap(true, Ordering::SeqCst) {
        return; // Already running
    }

    std::thread::spawn(|| {
        while MONITOR_RUNNING.load(Ordering::SeqCst) {
            if is_visible() {
                move_to_active_screen();
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}

/// Stop the screen monitor
#[cfg(target_os = "macos")]
pub fn stop_screen_monitor() {
    MONITOR_RUNNING.store(false, Ordering::SeqCst);
}

/// Start the animation loop - simple and optimized
/// Phase 0: Spawn animation once
/// Phase 1: Yawn animation once
/// Phase 2: Idle loop forever
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn start_animation() {
    if ANIMATION_RUNNING.swap(true, Ordering::SeqCst) {
        return; // Already running
    }

    CURRENT_FRAME.store(0, Ordering::SeqCst);
    ANIMATION_PHASE.store(0, Ordering::SeqCst);

    std::thread::spawn(|| {
        const SPAWN_DELAY: u64 = 50;  // 50ms per frame for spawn effect
        const YAWN_DELAY: u64 = 50;   // 50ms per frame for yawn
        const IDLE_DELAY: u64 = 80;   // 80ms per frame for idle (slower)

        // Wait for frames to be loaded before starting animation
        while !FRAMES_LOADED.load(Ordering::SeqCst) {
            if !ANIMATION_RUNNING.load(Ordering::SeqCst) {
                return; // Animation was stopped while waiting
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        log::info!("Frames loaded, waiting briefly before starting spawn animation");

        // Wait a moment to ensure the first frame is visible before starting animation
        std::thread::sleep(std::time::Duration::from_millis(100));

        while ANIMATION_RUNNING.load(Ordering::SeqCst) {
            let phase = ANIMATION_PHASE.load(Ordering::SeqCst);
            let frame = CURRENT_FRAME.load(Ordering::SeqCst);

            // Get the appropriate frames based on phase
            let (frames_guard, frame_count) = match phase {
                0 => {
                    let g = SPAWN_FRAMES.lock().unwrap();
                    let c = g.len();
                    (g, c)
                }
                1 => {
                    let g = YAWN_FRAMES.lock().unwrap();
                    let c = g.len();
                    (g, c)
                }
                _ => {
                    let g = IDLE_FRAMES.lock().unwrap();
                    let c = g.len();
                    (g, c)
                }
            };

            if frame_count > 0 {
                let idx = frame % frame_count;
                let image = frames_guard[idx].0;

                let iv_guard = IMAGE_VIEW.lock().unwrap();
                if let Some(ref iv) = *iv_guard {
                    unsafe {
                        let sel = sel!(setImage:);
                        let _: () = msg_send![iv.0,
                            performSelectorOnMainThread: sel
                            withObject: image
                            waitUntilDone: NO];
                    }
                }
                drop(iv_guard);
            }
            drop(frames_guard);

            // Advance frame
            let next = frame + 1;
            if phase == 0 && next >= frame_count {
                // Spawn complete, switch to yawn
                log::info!("Spawn complete, switching to yawn");
                ANIMATION_PHASE.store(1, Ordering::SeqCst);
                CURRENT_FRAME.store(0, Ordering::SeqCst);
            } else if phase == 1 && next >= frame_count {
                // Yawn complete, switch to idle loop
                log::info!("Yawn complete, switching to idle loop");
                ANIMATION_PHASE.store(2, Ordering::SeqCst);
                CURRENT_FRAME.store(0, Ordering::SeqCst);
            } else if phase == 2 {
                // Idle loop - wrap around
                CURRENT_FRAME.store(next % frame_count.max(1), Ordering::SeqCst);
            } else {
                CURRENT_FRAME.store(next, Ordering::SeqCst);
            }

            // Use different delays for each phase
            let delay = match phase {
                0 => SPAWN_DELAY,
                1 => YAWN_DELAY,
                _ => IDLE_DELAY,
            };
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
    });
}

/// Stop the animation loop
#[cfg(target_os = "macos")]
pub fn stop_animation() {
    ANIMATION_RUNNING.store(false, Ordering::SeqCst);
}

/// Show speech bubble with text
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn show_speech_bubble(text: &str) {
    let guard = SPEECH_BUBBLE.lock().unwrap();
    if let Some(ref safe_bubble) = *guard {
        unsafe {
            let ns_string = NSString::alloc(nil).init_str(text);
            let _: () = msg_send![safe_bubble.0, setStringValue: ns_string];
            let _: () = msg_send![safe_bubble.0, setHidden: NO];
        }
    }
}

/// Hide speech bubble
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn hide_speech_bubble() {
    let guard = SPEECH_BUBBLE.lock().unwrap();
    if let Some(ref safe_bubble) = *guard {
        unsafe {
            let _: () = msg_send![safe_bubble.0, setHidden: YES];
        }
    }
}

/// Handle mouse move for hover cursor effect
#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn handle_mouse_move() {
    unsafe {
        let screen_loc: NSPoint = msg_send![Class::get("NSEvent").unwrap(), mouseLocation];

        // Check if mouse is over the cat using screen coordinates directly
        let is_over_cat = {
            let panel_guard = OVERLAY_PANEL.lock().unwrap();
            if let Some(ref panel) = *panel_guard {
                let panel_frame: NSRect = msg_send![panel.0, frame];

                // Cat is 320x320 centered at bottom of panel
                // Calculate cat bounds in screen coordinates
                let cat_width = 320.0;
                let cat_height = 320.0;
                let cat_screen_x = panel_frame.origin.x + (panel_frame.size.width - cat_width) / 2.0;
                let cat_screen_y = panel_frame.origin.y; // Cat is at bottom of panel

                // Check if mouse is within cat bounds (screen coordinates)
                screen_loc.x >= cat_screen_x
                    && screen_loc.x <= cat_screen_x + cat_width
                    && screen_loc.y >= cat_screen_y
                    && screen_loc.y <= cat_screen_y + cat_height
            } else {
                false
            }
        };

        let currently_hand = CURSOR_IS_HAND.load(Ordering::SeqCst);

        if is_over_cat && !currently_hand {
            // Show hand cursor
            let hand_cursor: id = msg_send![Class::get("NSCursor").unwrap(), pointingHandCursor];
            let _: () = msg_send![hand_cursor, set];
            CURSOR_IS_HAND.store(true, Ordering::SeqCst);
        } else if !is_over_cat && currently_hand {
            // Restore arrow cursor
            let arrow_cursor: id = msg_send![Class::get("NSCursor").unwrap(), arrowCursor];
            let _: () = msg_send![arrow_cursor, set];
            CURSOR_IS_HAND.store(false, Ordering::SeqCst);
        }
    }
}

/// Handle click at screen location
#[cfg(target_os = "macos")]
fn handle_click_at_location(screen_loc: NSPoint) {
    // Debounce: ignore clicks within 800ms of the last click
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as usize;
    let last_click = LAST_CLICK_TIME.load(Ordering::SeqCst);
    let time_since_last = if now > last_click { now - last_click } else { 0 };

    // Check if this is a duplicate event (within 800ms)
    if time_since_last < 800 && last_click > 0 {
        eprintln!("[DEBUG] Debouncing click ({}ms since last)", time_since_last);
        log::info!("Debouncing click ({}ms since last)", time_since_last);
        return;
    }

    // Store the current time AFTER checking to ensure first click goes through
    LAST_CLICK_TIME.store(now, Ordering::SeqCst);

    let state = CHAT_STATE.load(Ordering::SeqCst);

    eprintln!("[DEBUG] Click detected at ({}, {}), state={}", screen_loc.x, screen_loc.y, state);
    log::info!("Click detected at ({}, {}), state={}", screen_loc.x, screen_loc.y, state);

    // Calculate click targets using screen coordinates directly
    eprintln!("[DEBUG] Acquiring OVERLAY_PANEL lock in handle_click");
    let (is_on_cat, is_on_send_btn) = {
        let panel_guard = OVERLAY_PANEL.lock().unwrap();
        eprintln!("[DEBUG] Got OVERLAY_PANEL lock in handle_click");
        if let Some(ref panel) = *panel_guard {
            unsafe {
                let panel_frame: NSRect = msg_send![panel.0, frame];

                eprintln!("[DEBUG] Panel frame: x={}, y={}, w={}, h={}",
                    panel_frame.origin.x, panel_frame.origin.y,
                    panel_frame.size.width, panel_frame.size.height);
                log::info!("Panel frame: x={}, y={}, w={}, h={}",
                    panel_frame.origin.x, panel_frame.origin.y,
                    panel_frame.size.width, panel_frame.size.height);

                // Use screen coordinates directly (more reliable than convertPointFromScreen)
                let cat_width = 320.0;
                let cat_height = 320.0;
                // Cat position in screen coordinates
                let cat_screen_x = panel_frame.origin.x + (panel_frame.size.width - cat_width) / 2.0;
                let cat_screen_y = panel_frame.origin.y; // Cat is at bottom of panel

                eprintln!("[DEBUG] Click at screen: ({}, {})", screen_loc.x, screen_loc.y);
                eprintln!("[DEBUG] Cat screen bounds: x={}-{}, y={}-{}",
                    cat_screen_x, cat_screen_x + cat_width, cat_screen_y, cat_screen_y + cat_height);
                log::info!("Click at screen: ({}, {})", screen_loc.x, screen_loc.y);
                log::info!("Cat screen bounds: x={}-{}, y={}-{}",
                    cat_screen_x, cat_screen_x + cat_width, cat_screen_y, cat_screen_y + cat_height);

                let c1 = screen_loc.x >= cat_screen_x;
                let c2 = screen_loc.x <= cat_screen_x + cat_width;
                let c3 = screen_loc.y >= cat_screen_y;
                let c4 = screen_loc.y <= cat_screen_y + cat_height;
                eprintln!("[DEBUG] Cat check: x>={}: {}, x<={}: {}, y>={}: {}, y<={}: {}",
                    cat_screen_x, c1, cat_screen_x + cat_width, c2, cat_screen_y, c3, cat_screen_y + cat_height, c4);
                log::info!("Cat check: x>={}: {}, x<={}: {}, y>={}: {}, y<={}: {}",
                    cat_screen_x, c1, cat_screen_x + cat_width, c2, cat_screen_y, c3, cat_screen_y + cat_height, c4);
                let is_on_cat = c1 && c2 && c3 && c4;

                // Check if click is on send button - button is NOT on overlay panel anymore
                // The chat panel is separate, so this check is not valid here
                // We'll handle send button clicks through the chat panel
                let is_on_send_btn = false;

                (is_on_cat, is_on_send_btn)
            }
        } else {
            eprintln!("[DEBUG] No panel found");
            log::warn!("No panel found");
            return;
        }
    }; // panel_guard is dropped here at end of scope
    eprintln!("[DEBUG] Released OVERLAY_PANEL lock in handle_click");

    eprintln!("[DEBUG] is_on_cat={}, is_on_send_btn={}", is_on_cat, is_on_send_btn);
    log::info!("is_on_cat={}, is_on_send_btn={}", is_on_cat, is_on_send_btn);

    // Now handle the click action - all locks are released
    if is_on_send_btn {
        eprintln!("[DEBUG] Submitting chat input");
        log::info!("Submitting chat input");
        submit_chat_input();
    } else if is_on_cat {
        // Always animate click feedback on cat
        animate_cat_click_feedback();

        match state {
            0 => {
                eprintln!("[DEBUG] Opening chat input");
                log::info!("Opening chat input");
                show_chat_input();
                eprintln!("[DEBUG] show_chat_input returned");
            }
            1 => {
                eprintln!("[DEBUG] Closing chat input");
                log::info!("Closing chat input");
                hide_chat_input();
            }
            2 | 3 => {
                eprintln!("[DEBUG] Ignoring click - thinking or responding");
                log::info!("Ignoring click - thinking or responding");
            }
            _ => {}
        }
    }
    eprintln!("[DEBUG] handle_click_at_location complete");
}

/// Start click monitor for cat interaction
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn start_click_monitor() {
    if CLICK_MONITOR_RUNNING.swap(true, Ordering::SeqCst) {
        log::info!("Click monitor already running");
        eprintln!("[DEBUG] Click monitor already running");
        return;
    }

    log::info!("Starting click monitor");
    eprintln!("[DEBUG] Starting click monitor");

    std::thread::spawn(|| {
        use block::ConcreteBlock;

        unsafe {
            eprintln!("[DEBUG] Inside click monitor thread");

            // NSEventMaskLeftMouseDown = 1 << 1
            let mouse_mask: u64 = 1 << 1;

            // Global monitor - catches clicks when OTHER apps are focused
            let global_handler = ConcreteBlock::new(move |_event: id| {
                eprintln!("[DEBUG] GLOBAL click detected!");
                let screen_loc: NSPoint = msg_send![Class::get("NSEvent").unwrap(), mouseLocation];
                handle_click_at_location(screen_loc);
            });
            let global_handler = global_handler.copy();

            let global_monitor: id = msg_send![Class::get("NSEvent").unwrap(),
                addGlobalMonitorForEventsMatchingMask: mouse_mask
                handler: &*global_handler
            ];
            eprintln!("[DEBUG] Global mouse monitor: {:?}", global_monitor);
            log::info!("Global mouse monitor registered: {:?}", global_monitor);

            // NOTE: Local monitor removed - global monitor should catch all clicks
            // Local monitors cause double-click issues when both are registered

            // Add mouse move monitor for hover cursor effect (both global and local)
            let move_mask: u64 = 1 << 5; // NSEventMaskMouseMoved
            let move_handler = ConcreteBlock::new(move |_event: id| {
                handle_mouse_move();
            });
            let move_handler = move_handler.copy();

            let _: id = msg_send![Class::get("NSEvent").unwrap(),
                addGlobalMonitorForEventsMatchingMask: move_mask
                handler: &*move_handler
            ];

            let move_handler_local = ConcreteBlock::new(move |event: id| -> id {
                handle_mouse_move();
                event
            });
            let move_handler_local = move_handler_local.copy();

            let _: id = msg_send![Class::get("NSEvent").unwrap(),
                addLocalMonitorForEventsMatchingMask: move_mask
                handler: &*move_handler_local
            ];
            log::info!("Mouse move monitors registered for hover cursor");
            eprintln!("[DEBUG] Mouse move monitors registered");

            // Add key event monitors for Enter key
            let key_mask: u64 = 1 << 10; // NSEventMaskKeyDown

            let global_key_handler = ConcreteBlock::new(move |_event: id| {
                let state = CHAT_STATE.load(Ordering::SeqCst);
                if state == 1 {
                    let key_code: u16 = msg_send![_event, keyCode];
                    if key_code == 36 || key_code == 76 {
                        log::info!("Enter key pressed (global)");
                        submit_chat_input();
                    }
                }
            });
            let global_key_handler = global_key_handler.copy();

            let _: id = msg_send![Class::get("NSEvent").unwrap(),
                addGlobalMonitorForEventsMatchingMask: key_mask
                handler: &*global_key_handler
            ];

            let local_key_handler = ConcreteBlock::new(move |event: id| -> id {
                let state = CHAT_STATE.load(Ordering::SeqCst);
                if state == 1 {
                    let key_code: u16 = msg_send![event, keyCode];
                    if key_code == 36 || key_code == 76 {
                        log::info!("Enter key pressed (local)");
                        submit_chat_input();
                    }
                }
                event
            });
            let local_key_handler = local_key_handler.copy();

            let _: id = msg_send![Class::get("NSEvent").unwrap(),
                addLocalMonitorForEventsMatchingMask: key_mask
                handler: &*local_key_handler
            ];
            log::info!("Key monitors registered");
            eprintln!("[DEBUG] Key monitors registered");
            eprintln!("[DEBUG] Click monitor setup complete, entering run loop");

            // Keep the thread alive with proper run loop
            loop {
                if !CLICK_MONITOR_RUNNING.load(Ordering::SeqCst) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            eprintln!("[DEBUG] Click monitor thread exiting");
        }
    });

    eprintln!("[DEBUG] Click monitor spawned");
}

/// Stop click monitor
#[cfg(target_os = "macos")]
pub fn stop_click_monitor() {
    CLICK_MONITOR_RUNNING.store(false, Ordering::SeqCst);
}

/// Animate click feedback on the cat image (scale bounce effect)
#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn animate_cat_click_feedback() {
    unsafe {
        let guard = IMAGE_VIEW.lock().unwrap();
        if let Some(ref img_view) = *guard {
            // Quick scale-down then scale-up animation for click feedback
            // Use NSAnimationContext with completion handler
            let _: () = msg_send![Class::get("NSAnimationContext").unwrap(), beginGrouping];
            let context: id = msg_send![Class::get("NSAnimationContext").unwrap(), currentContext];
            let _: () = msg_send![context, setDuration: 0.08_f64];

            // Get the layer for transform
            let layer: id = msg_send![img_view.0, layer];
            if layer != nil {
                // Create CABasicAnimation for transform.scale
                let animation: id = msg_send![Class::get("CABasicAnimation").unwrap(),
                    animationWithKeyPath: NSString::alloc(nil).init_str("transform.scale")];
                let _: () = msg_send![animation, setFromValue: {
                    let num: id = msg_send![Class::get("NSNumber").unwrap(), numberWithFloat: 1.0_f32];
                    num
                }];
                let _: () = msg_send![animation, setToValue: {
                    let num: id = msg_send![Class::get("NSNumber").unwrap(), numberWithFloat: 0.92_f32];
                    num
                }];
                let _: () = msg_send![animation, setDuration: 0.08_f64];
                let _: () = msg_send![animation, setAutoreverses: YES];
                let _: () = msg_send![animation, setTimingFunction: {
                    let timing: id = msg_send![Class::get("CAMediaTimingFunction").unwrap(),
                        functionWithName: NSString::alloc(nil).init_str("easeInEaseOut")];
                    timing
                }];
                let _: () = msg_send![layer, addAnimation: animation forKey: NSString::alloc(nil).init_str("clickBounce")];
            }

            let _: () = msg_send![Class::get("NSAnimationContext").unwrap(), endGrouping];
        }
    }
}

/// Show chat input box with fade in animation - CENTERED ON SCREEN
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn show_chat_input() {
    log::info!("show_chat_input called");
    CHAT_STATE.store(1, Ordering::SeqCst); // InputOpen

    unsafe {
        // Check if chat panel exists, create if not
        let mut panel_exists = false;
        {
            let guard = CHAT_PANEL.lock().unwrap();
            panel_exists = guard.is_some();
        }

        if !panel_exists {
            // Create a new centered panel for chat
            let screen: id = msg_send![Class::get("NSScreen").unwrap(), mainScreen];
            let screen_frame: NSRect = msg_send![screen, frame];

            let chat_width = 400.0;
            let chat_height = 56.0;
            let chat_x = (screen_frame.size.width - chat_width) / 2.0;
            let chat_y = (screen_frame.size.height - chat_height) / 2.0;

            let panel_frame = NSRect::new(
                NSPoint::new(chat_x, chat_y),
                NSSize::new(chat_width, chat_height),
            );

            // Create custom panel class for keyboard input
            let panel_class = get_or_create_key_panel_class();
            let chat_panel: id = msg_send![panel_class, alloc];

            // NSBorderlessWindowMask | NSNonactivatingPanelMask
            let style_mask: u64 = 0 | (1 << 7);
            let chat_panel: id = msg_send![chat_panel, initWithContentRect:panel_frame
                styleMask:style_mask
                backing:2_i64 // NSBackingStoreBuffered
                defer:NO];

            // Make panel float above everything
            let _: () = msg_send![chat_panel, setLevel: KCGMAXIMUM_WINDOW_LEVEL];
            let _: () = msg_send![chat_panel, setOpaque: NO];
            let clear: id = msg_send![Class::get("NSColor").unwrap(), clearColor];
            let _: () = msg_send![chat_panel, setBackgroundColor: clear];
            let _: () = msg_send![chat_panel, setHasShadow: NO];
            let _: () = msg_send![chat_panel, setCollectionBehavior: 1_u64 | 16_u64];

            // Get content view
            let content_view: id = msg_send![chat_panel, contentView];
            let _: () = msg_send![content_view, setWantsLayer: YES];

            // Create sleek chat container
            let container_frame = NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(chat_width, chat_height),
            );

            let chat_container: id = msg_send![Class::get("NSView").unwrap(), alloc];
            let chat_container: id = msg_send![chat_container, initWithFrame: container_frame];
            let _: () = msg_send![chat_container, setWantsLayer: YES];

            // Style: WHITE background, sleek pill shape
            let chat_layer: id = msg_send![chat_container, layer];
            let _: () = msg_send![chat_layer, setCornerRadius: 28.0_f64];
            let _: () = msg_send![chat_layer, setMasksToBounds: NO];

            let white: id = msg_send![Class::get("NSColor").unwrap(), whiteColor];
            let cg_white: id = msg_send![white, CGColor];
            let _: () = msg_send![chat_layer, setBackgroundColor: cg_white];

            // Subtle border
            let border_color = create_color(230.0, 230.0, 230.0, 1.0);
            let cg_border: id = msg_send![border_color, CGColor];
            let _: () = msg_send![chat_layer, setBorderColor: cg_border];
            let _: () = msg_send![chat_layer, setBorderWidth: 1.0_f64];

            // Elegant shadow
            let _: () = msg_send![chat_layer, setShadowOpacity: 0.2_f32];
            let _: () = msg_send![chat_layer, setShadowRadius: 25.0_f64];
            let _: () = msg_send![chat_layer, setShadowOffset: NSSize::new(0.0, -8.0)];
            let black: id = msg_send![Class::get("NSColor").unwrap(), blackColor];
            let cg_black: id = msg_send![black, CGColor];
            let _: () = msg_send![chat_layer, setShadowColor: cg_black];

            // Create text input
            let input_padding = 24.0;
            let btn_space = 56.0;
            let input_width = chat_width - input_padding * 2.0 - btn_space;
            let input_height = 30.0;
            let input_frame = NSRect::new(
                NSPoint::new(input_padding, (chat_height - input_height) / 2.0),
                NSSize::new(input_width, input_height),
            );

            let chat_input: id = msg_send![Class::get("NSTextField").unwrap(), alloc];
            let chat_input: id = msg_send![chat_input, initWithFrame: input_frame];
            let _: () = msg_send![chat_input, setEditable: YES];
            let _: () = msg_send![chat_input, setSelectable: YES];
            let _: () = msg_send![chat_input, setBordered: NO];
            let _: () = msg_send![chat_input, setDrawsBackground: NO];
            let _: () = msg_send![chat_input, setWantsLayer: YES];
            let _: () = msg_send![chat_input, setFocusRingType: 0_i64];

            // Black text
            let black_text: id = msg_send![Class::get("NSColor").unwrap(), blackColor];
            let _: () = msg_send![chat_input, setTextColor: black_text];

            // Modern system font
            let input_font: id = msg_send![Class::get("NSFont").unwrap(), systemFontOfSize: 16.0_f64 weight: 0.0_f64];
            let _: () = msg_send![chat_input, setFont: input_font];

            // Gray placeholder
            let placeholder_str = NSString::alloc(nil).init_str("Ask me anything...");
            let placeholder_color = create_color(160.0, 160.0, 160.0, 1.0);
            let fg_key = NSString::alloc(nil).init_str("NSColor");
            let font_key = NSString::alloc(nil).init_str("NSFont");
            let keys: [id; 2] = [fg_key, font_key];
            let objects: [id; 2] = [placeholder_color, input_font];
            let placeholder_dict: id = msg_send![Class::get("NSDictionary").unwrap(),
                dictionaryWithObjects: objects.as_ptr()
                forKeys: keys.as_ptr()
                count: 2_usize
            ];
            let attr_placeholder: id = msg_send![Class::get("NSAttributedString").unwrap(), alloc];
            let attr_placeholder: id = msg_send![attr_placeholder, initWithString:placeholder_str attributes:placeholder_dict];
            let _: () = msg_send![chat_input, setPlaceholderAttributedString: attr_placeholder];

            let _: () = msg_send![chat_container, addSubview: chat_input];

            // Create send button
            let btn_size = 40.0;
            let btn_frame = NSRect::new(
                NSPoint::new(chat_width - btn_size - 8.0, (chat_height - btn_size) / 2.0),
                NSSize::new(btn_size, btn_size),
            );

            let send_btn: id = msg_send![Class::get("NSButton").unwrap(), alloc];
            let send_btn: id = msg_send![send_btn, initWithFrame: btn_frame];
            let _: () = msg_send![send_btn, setWantsLayer: YES];
            let _: () = msg_send![send_btn, setBordered: NO];

            // Black circular button
            let btn_layer: id = msg_send![send_btn, layer];
            let _: () = msg_send![btn_layer, setCornerRadius: 20.0_f64];
            let cg_black_bg: id = msg_send![black, CGColor];
            let _: () = msg_send![btn_layer, setBackgroundColor: cg_black_bg];

            // White arrow on button
            let btn_title = NSString::alloc(nil).init_str("↑");
            let btn_font: id = msg_send![Class::get("NSFont").unwrap(), boldSystemFontOfSize: 18.0_f64];
            let white_text: id = msg_send![Class::get("NSColor").unwrap(), whiteColor];
            let btn_keys: [id; 2] = [fg_key, font_key];
            let btn_objects: [id; 2] = [white_text, btn_font];
            let btn_dict: id = msg_send![Class::get("NSDictionary").unwrap(),
                dictionaryWithObjects: btn_objects.as_ptr()
                forKeys: btn_keys.as_ptr()
                count: 2_usize
            ];
            let btn_attr: id = msg_send![Class::get("NSAttributedString").unwrap(), alloc];
            let btn_attr: id = msg_send![btn_attr, initWithString:btn_title attributes:btn_dict];
            let _: () = msg_send![send_btn, setAttributedTitle: btn_attr];

            let _: () = msg_send![chat_container, addSubview: send_btn];
            let _: () = msg_send![content_view, addSubview: chat_container];

            // Store references
            {
                let mut guard = CHAT_PANEL.lock().unwrap();
                *guard = Some(SafeId(chat_panel));
            }
            {
                let mut guard = CHAT_CONTAINER.lock().unwrap();
                *guard = Some(SafeId(chat_container));
            }
            {
                let mut guard = CHAT_INPUT.lock().unwrap();
                *guard = Some(SafeId(chat_input));
            }
            {
                let mut guard = SEND_BUTTON.lock().unwrap();
                *guard = Some(SafeId(send_btn));
            }
        }

        // Show the chat panel - get raw pointer first to avoid holding lock
        let (panel_ptr, input_ptr) = {
            let guard = CHAT_PANEL.lock().unwrap();
            let input_guard = CHAT_INPUT.lock().unwrap();
            let panel = guard.as_ref().map(|p| p.0);
            let input = input_guard.as_ref().map(|i| i.0);
            (panel, input)
        };

        if let Some(panel) = panel_ptr {
            eprintln!("[DEBUG] Showing chat panel...");
            log::info!("Showing chat panel at center of screen");

            // Get panel frame for debugging
            let frame: NSRect = msg_send![panel, frame];
            eprintln!("[DEBUG] Chat panel frame: x={}, y={}, w={}, h={}",
                frame.origin.x, frame.origin.y, frame.size.width, frame.size.height);
            log::info!("Chat panel frame: x={}, y={}, w={}, h={}",
                frame.origin.x, frame.origin.y, frame.size.width, frame.size.height);

            // Ensure panel is visible
            let _: () = msg_send![panel, setAlphaValue: 1.0_f64];
            let _: () = msg_send![panel, setIsVisible: YES];
            let _: () = msg_send![panel, orderFrontRegardless];

            // Make key window and focus input
            let app: id = msg_send![Class::get("NSApplication").unwrap(), sharedApplication];
            let _: () = msg_send![app, activateIgnoringOtherApps: YES];
            let _: () = msg_send![panel, makeKeyAndOrderFront: nil];

            if let Some(input) = input_ptr {
                let _: () = msg_send![input, selectText: nil];
                let _: () = msg_send![panel, makeFirstResponder: input];
            }
            eprintln!("[DEBUG] Chat panel shown and focused");
            log::info!("Chat panel shown and focused");
        } else {
            eprintln!("[DEBUG] ERROR: Chat panel is None!");
            log::info!("ERROR: Chat panel is None!");
        }
    }
    eprintln!("[DEBUG] show_chat_input complete");
    log::info!("show_chat_input complete");
}

/// Hide chat input box with fade out
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn hide_chat_input() {
    CHAT_STATE.store(0, Ordering::SeqCst); // Idle

    unsafe {
        let guard = CHAT_PANEL.lock().unwrap();
        if let Some(ref panel) = *guard {
            // Simply hide immediately without animation to avoid thread issues
            let _: () = msg_send![panel.0, setAlphaValue: 0.0_f64];
            let _: () = msg_send![panel.0, orderOut: nil];
        }
    }
}

/// Show thinking indicator
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn show_thinking() {
    CHAT_STATE.store(2, Ordering::SeqCst); // Thinking

    unsafe {
        // Hide the chat panel immediately
        let guard = CHAT_PANEL.lock().unwrap();
        if let Some(ref panel) = *guard {
            let _: () = msg_send![panel.0, setAlphaValue: 0.0_f64];
            let _: () = msg_send![panel.0, orderOut: nil];
        }
    }

    // Show thinking label immediately
    unsafe {
        let guard = THINKING_LABEL.lock().unwrap();
        if let Some(ref label) = *guard {
            let _: () = msg_send![label.0, setHidden: NO];
            let _: () = msg_send![label.0, setAlphaValue: 1.0_f64];
        }
    }
}

/// Hide thinking indicator
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn hide_thinking() {
    let guard = THINKING_LABEL.lock().unwrap();
    if let Some(ref label) = *guard {
        unsafe {
            let _: () = msg_send![label.0, setAlphaValue: 0.0_f64];
            let _: () = msg_send![label.0, setHidden: YES];
        }
    }
}

/// Show response box with typing effect
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn show_response_with_typing(text: String) {
    CHAT_STATE.store(3, Ordering::SeqCst); // Responding

    // Store the response text
    {
        let mut guard = CURRENT_RESPONSE.lock().unwrap();
        *guard = text.clone();
    }
    RESPONSE_CHAR_INDEX.store(0, Ordering::SeqCst);

    // Hide thinking
    hide_thinking();

    // Show response box immediately with full text (no typing animation for now)
    unsafe {
        let guard = RESPONSE_BOX.lock().unwrap();
        if let Some(ref box_) = *guard {
            let ns_str = NSString::alloc(nil).init_str(&text);
            let _: () = msg_send![box_.0, setStringValue: ns_str];
            let _: () = msg_send![box_.0, setHidden: NO];
            let _: () = msg_send![box_.0, setAlphaValue: 1.0_f64];
        }
    }
}

/// Start typing effect for response
#[cfg(target_os = "macos")]
#[allow(deprecated)]
fn start_typing_effect() {
    if TYPING_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::spawn(|| {
        while TYPING_RUNNING.load(Ordering::SeqCst) {
            let response = CURRENT_RESPONSE.lock().unwrap().clone();
            let idx = RESPONSE_CHAR_INDEX.fetch_add(1, Ordering::SeqCst);

            if idx >= response.len() {
                // Typing complete
                TYPING_RUNNING.store(false, Ordering::SeqCst);

                // After a delay, hide response and return to idle
                std::thread::sleep(std::time::Duration::from_secs(5));
                hide_response();
                break;
            }

            // Update displayed text
            let display_text: String = response.chars().take(idx + 1).collect();

            let guard = RESPONSE_BOX.lock().unwrap();
            if let Some(ref box_) = *guard {
                unsafe {
                    let ns_str = NSString::alloc(nil).init_str(&display_text);
                    let sel = sel!(setStringValue:);
                    let _: () = msg_send![box_.0,
                        performSelectorOnMainThread: sel
                        withObject: ns_str
                        waitUntilDone: NO
                    ];
                }
            }
            drop(guard);

            // Typing speed: 30ms per character
            std::thread::sleep(std::time::Duration::from_millis(30));
        }
    });
}

/// Hide response box
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn hide_response() {
    TYPING_RUNNING.store(false, Ordering::SeqCst);

    // Hide response box immediately
    {
        let guard = RESPONSE_BOX.lock().unwrap();
        if let Some(ref box_) = *guard {
            unsafe {
                let _: () = msg_send![box_.0, setAlphaValue: 0.0_f64];
                let _: () = msg_send![box_.0, setHidden: YES];
            }
        }
    }

    // Disable mouse events on panel (pass through)
    {
        let panel_guard = OVERLAY_PANEL.lock().unwrap();
        if let Some(ref panel) = *panel_guard {
            unsafe {
                let _: () = msg_send![panel.0, setIgnoresMouseEvents: YES];
            }
        }
    }

    CHAT_STATE.store(0, Ordering::SeqCst); // Back to idle
}

/// Send message to Groq API
#[cfg(target_os = "macos")]
pub fn send_to_groq(message: String) {
    show_thinking();

    std::thread::spawn(move || {
        // Use tokio runtime for async HTTP request
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let api_key = GROQ_API_KEY.lock().unwrap().clone();

            if api_key.is_empty() {
                log::error!("Groq API key not set");
                show_response_with_typing("Please set a Groq API key first!".to_string());
                return;
            }

            let client = reqwest::Client::new();

            let body = serde_json::json!({
                "model": "llama-3.3-70b-versatile",
                "messages": [
                    {
                        "role": "system",
                        "content": "You are a helpful, friendly cat assistant named Mitten. Keep your responses concise and friendly - max 2-3 sentences. Add a playful cat-like touch to your responses."
                    },
                    {
                        "role": "user",
                        "content": message
                    }
                ],
                "max_tokens": 150,
                "temperature": 0.7
            });

            match client.post("https://api.groq.com/openai/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
            {
                Ok(response) => {
                    match response.json::<serde_json::Value>().await {
                        Ok(json) => {
                            if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                                show_response_with_typing(content.to_string());
                            } else {
                                show_response_with_typing("Meow? I couldn't understand that...".to_string());
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to parse Groq response: {}", e);
                            show_response_with_typing("Meow... something went wrong!".to_string());
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to call Groq API: {}", e);
                    show_response_with_typing("Meow... I couldn't reach the server!".to_string());
                }
            }
        });
    });
}

/// Get current input text and send to Groq
#[cfg(target_os = "macos")]
#[allow(deprecated)]
pub fn submit_chat_input() {
    let input_guard = CHAT_INPUT.lock().unwrap();
    if let Some(ref input) = *input_guard {
        unsafe {
            let text: id = msg_send![input.0, stringValue];
            let c_str: *const i8 = msg_send![text, UTF8String];
            if !c_str.is_null() {
                let message = std::ffi::CStr::from_ptr(c_str).to_string_lossy().to_string();
                if !message.is_empty() {
                    // Clear input
                    let _: () = msg_send![input.0, setStringValue: NSString::alloc(nil).init_str("")];
                    drop(input_guard);
                    send_to_groq(message);
                    return;
                }
            }
        }
    }
}

// Stubs for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub fn create_overlay(_width: f64, _height: f64) {}
#[cfg(not(target_os = "macos"))]
pub fn show_overlay() {}
#[cfg(not(target_os = "macos"))]
pub fn hide_overlay() {}
#[cfg(not(target_os = "macos"))]
pub fn close_overlay() {}
#[cfg(not(target_os = "macos"))]
pub fn is_visible() -> bool { false }
#[cfg(not(target_os = "macos"))]
pub fn move_to_active_screen() {}
#[cfg(not(target_os = "macos"))]
pub fn start_screen_monitor() {}
#[cfg(not(target_os = "macos"))]
pub fn stop_screen_monitor() {}
#[cfg(not(target_os = "macos"))]
pub fn start_animation() {}
#[cfg(not(target_os = "macos"))]
pub fn stop_animation() {}
#[cfg(not(target_os = "macos"))]
pub fn show_speech_bubble(_text: &str) {}
#[cfg(not(target_os = "macos"))]
pub fn hide_speech_bubble() {}
#[cfg(not(target_os = "macos"))]
pub fn start_click_monitor() {}
#[cfg(not(target_os = "macos"))]
pub fn stop_click_monitor() {}
#[cfg(not(target_os = "macos"))]
pub fn show_chat_input() {}
#[cfg(not(target_os = "macos"))]
pub fn hide_chat_input() {}
#[cfg(not(target_os = "macos"))]
pub fn show_thinking() {}
#[cfg(not(target_os = "macos"))]
pub fn hide_thinking() {}
#[cfg(not(target_os = "macos"))]
pub fn show_response_with_typing(_text: String) {}
#[cfg(not(target_os = "macos"))]
pub fn hide_response() {}
#[cfg(not(target_os = "macos"))]
pub fn send_to_groq(_message: String) {}
#[cfg(not(target_os = "macos"))]
pub fn submit_chat_input() {}
#[cfg(not(target_os = "macos"))]
pub fn set_groq_api_key(_key: &str) {}

