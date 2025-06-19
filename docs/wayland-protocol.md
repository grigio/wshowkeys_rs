# Wayland Protocol Integration for wshowkeys_rs

## Overview

This document explains how wshowkeys_rs integrates with Wayland to display keystrokes on screen. The implementation uses the Wayland Layer Shell protocol to create an overlay surface that shows keypresses in real-time.

## Architecture

### Main Components

1. **WaylandDisplay**: Main class managing Wayland connection and display
2. **AppData**: Application state shared between Wayland event handlers
3. **Layer Shell Integration**: Uses wlr-layer-shell protocol for overlay display
4. **Shared Memory Rendering**: Direct rendering to Wayland shared memory buffers

### Connection Flow

```
Application Start
       ↓
Connect to Wayland Compositor
       ↓
Initialize Registry & Bind Protocols
       ↓
Create Layer Surface
       ↓
Configure Surface Properties
       ↓
Main Event Loop (Render & Display)
```

## Wayland Protocols Used

### Core Protocols

- **wl_compositor**: Creates surfaces and manages composition
- **wl_shm**: Shared memory protocol for buffer allocation
- **wl_surface**: Represents drawable surface
- **wl_buffer**: Memory buffer for pixel data
- **wl_output**: Monitor/display information

### Extension Protocols

- **wlr_layer_shell_v1**: Layer shell protocol for overlay surfaces
  - Allows positioning surfaces above/below other windows
  - Provides anchor positioning (top, bottom, left, right)
  - Manages exclusive zones and margins

## Implementation Details

### 1. Connection Initialization

```rust
// Connect to Wayland compositor via environment socket
let connection = Connection::connect_to_env()?;

// Initialize registry to discover available protocols
let (globals, queue) = registry_queue_init(&connection)?;
```

### 2. Protocol Binding

The application binds to required Wayland protocols:

- **Compositor**: For surface creation
- **SharedMemory**: For buffer allocation
- **LayerShell**: For overlay positioning
- **Output**: For multi-monitor support

### 3. Layer Surface Creation

```rust
// Create a surface from compositor
let surface = compositor_state.create_surface(&qh);

// Create layer surface on overlay layer
let layer_surface = layer_shell.create_layer_surface(
    &qh,
    surface.clone(),
    Layer::Overlay,        // Above normal windows
    Some("wshowkeys_rs"),  // Application identifier
    None,                  // All outputs
);
```

### 4. Surface Configuration

The layer surface is configured with:

- **Anchor Position**: Where to position on screen (corners/edges)
- **Margins**: Distance from screen edges
- **Keyboard Interactivity**: Disabled (passthrough)
- **Exclusive Zone**: No space reservation

### 5. Rendering Pipeline

#### 5.1 Buffer Allocation

```rust
// Create temporary file for shared memory
let temp_file = tempfile::tempfile()?;
temp_file.set_len(buffer_size as u64)?;

// Create Wayland shared memory pool
let fd = unsafe { BorrowedFd::borrow_raw(temp_file.as_raw_fd()) };
let pool = shm.wl_shm().create_pool(fd, buffer_size as i32, &qh, ());

// Create buffer with ARGB32 format
let buffer = pool.create_buffer(
    0,                                    // offset
    width as i32,
    height as i32,
    (width * 4) as i32,                  // stride (4 bytes per pixel)
    wl_shm::Format::Argb8888,
    &qh,
    (),
);
```

#### 5.2 Direct Cairo Rendering

```rust
// Memory map the shared buffer
let mut mmap = unsafe { memmap2::MmapMut::map_mut(&temp_file)? };

// Create Cairo surface directly on shared memory
let cairo_surface = unsafe {
    cairo::ImageSurface::create_for_data_unsafe(
        mmap.as_mut_ptr(),
        cairo::Format::ARgb32,
        width as i32,
        height as i32,
        (width * 4) as i32,
    )
}?;

// Render content using Cairo
let context = cairo::Context::new(&cairo_surface)?;
// ... rendering operations ...
```

#### 5.3 Surface Update

```rust
// Attach buffer to surface
surface.attach(Some(&buffer), 0, 0);

// Mark damaged region
surface.damage_buffer(0, 0, width as i32, height as i32);

// Commit changes to compositor
surface.commit();
```

## Event Handling

### Required Trait Implementations

The application implements several Wayland event handler traits:

1. **CompositorHandler**: Surface lifecycle events
2. **LayerShellHandler**: Layer surface configuration
3. **ShmHandler**: Shared memory events
4. **OutputHandler**: Monitor events
5. **ProvidesRegistryState**: Protocol discovery

### Layer Surface Configuration Events

```rust
fn configure(
    &mut self,
    _conn: &Connection,
    _qh: &QueueHandle<AppData>,
    _layer: &LayerSurface,
    configure: LayerSurfaceConfigure,
    _serial: u32,
) {
    // Update surface dimensions
    self.width = configure.new_size.0.max(1);
    self.height = configure.new_size.1.max(1);
    self.configured = true;
}
```

## Main Event Loop Integration

### Async Event Processing

The main event loop in `main.rs` uses `tokio::select!` to handle:

1. **Input Events**: Keypress detection from input devices
2. **Render Timer**: Fixed-interval rendering (~60 FPS)
3. **Wayland Events**: Protocol message dispatch

### Rendering Flow

```rust
tokio::select! {
    // Handle keypress events
    input_event = input_manager.next_event() => {
        // Process keypress and update buffer
        key_buffer.add_keypress(keypress);
        buffer_changed = true;
    }
    
    // Fixed-interval rendering
    _ = render_timer.tick() => {
        // Clean expired keypresses
        key_buffer.cleanup_expired();
        
        // Render if buffer changed
        if buffer_changed {
            render_and_display(&renderer, &mut wayland_display, &key_buffer).await?;
            buffer_changed = false;
        }
        
        // Dispatch Wayland events
        wayland_display.dispatch_events()?;
    }
}
```

## Error Handling

### Connection Errors

- **Connection Failure**: Fallback to environment variables
- **Protocol Missing**: Check compositor capabilities
- **Surface Creation**: Retry with different parameters

### Rendering Errors

- **Buffer Allocation**: Handle memory constraints
- **Cairo Operations**: Graceful degradation
- **Surface Attachment**: Queue operations safely

## Multi-Monitor Support

The implementation supports multiple monitors through:

1. **Output Discovery**: Automatic detection of connected displays
2. **Surface Replication**: Optional per-output surfaces
3. **Configuration**: User-specified output targeting

## Performance Considerations

### Optimization Strategies

1. **Conditional Rendering**: Only render when content changes
2. **Buffer Reuse**: Minimize allocation overhead
3. **Direct Memory Access**: Zero-copy rendering to shared buffers
4. **Event Batching**: Efficient Wayland event processing

### Resource Management

- **File Descriptors**: Proper cleanup of temporary files
- **Memory Mapping**: Safe unmapping on destruction
- **Surface Lifecycle**: Clean resource deallocation

## Security Considerations

### Permissions Required

- **Input Device Access**: Read access to `/dev/input/event*`
- **Wayland Socket**: Connection to compositor
- **Temporary Files**: Write access for shared memory

### Privilege Escalation

The application may require special permissions for input device access:
- Group membership: `input`
- Capabilities: `CAP_DAC_READ_SEARCH`
- Sudo/setuid: For system-wide access

## Debugging

### Common Issues

1. **Surface Not Visible**: Check layer shell support
2. **Input Not Working**: Verify device permissions
3. **Rendering Artifacts**: Validate Cairo operations
4. **Performance Issues**: Monitor memory usage

### Debug Output

Enable debug logging to trace:
- Wayland protocol messages
- Surface configuration events
- Buffer allocation/deallocation
- Rendering operations
