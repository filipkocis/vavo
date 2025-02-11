# Vavo Game Engine

Vavo is a simple, fast and modular ECS game engine built in Rust. Designed for performance, scalability, and ease of use, while being completely free and open-source!

## Features

- **Entity-Component-System (ECS)**: Built with an efficient ECS architecture for clean separation of data and logic.
- **Query System**: Provides fine-grained access to entity components, with various filters and change detection.
- **2D/3D Renderer**: Supports both 2D and 3D rendering with a flexible pipeline.
- **PBR & Blinn-Phong Lighting**: Default lighting system combines Physically Based Rendering (PBR) with Blinn-Phong for high-quality visuals.
- **Asset Management**: Efficiently loads, caches, and manages textures, models, and other resources.
- **Modular Design**: Easily extend and customize the engine to suit different game types.
- **Minimal Boilerplate**: Focus on game logic rather than engine intricacies.

## Roadmap

- [ ] Audio support
- [ ] Skeletal animations
- [ ] Implement physics engine integration
- [ ] Expand documentation and tutorials
- [ ] Add better 2D rendering support
- [ ] Improve tooling for debugging and profiling
- [ ] Multithreading and concurrency support
- [ ] Improved DX for custom render node creation and management
- [ ] Scene management and serialization
- [ ] Scripting support (LUA)

## Installation

To use Vavo in your Rust project, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
vavo = { git = "https://github.com/filipkocis/vavo.git", branch = "main" }
# vavo = "0.1.0"
```

Then, include it in your Rust code:

```rust
use vavo::prelude::*;
```

## Getting Started

### Basic Window Creation

Here’s a simple example demonstrating the creation of a basic window:

```rust
use vavo::prelude::*;

fn main() {
    App::build()
        .add_plugin(DefaultPlugin)
        .run();
}
```

### System Creation Example

This example shows how to create a basic system in Vavo:

```rust
use vavo::prelude::*;

struct Position {
    x: f32,
    y: f32,
}

fn setup_system(ctx: &mut SystemsContext, _: Query<()>) {
    ctx.commands.spawn_empty()
        .insert(Position {
            x: 0.,
            y: 0.,
        });
}

fn movement_system(_: &mut SystemsContext, mut query: Query<&mut Position>) {
    for pos in query.iter_mut() {
        pos.y += 42.;
    }
}

fn main() {
    App::build()
        .add_plugin(DefaultPlugin)
        .add_startup_system(setup_system)
        .add_system(movement_system)
        .run()
}
```

## Contributing

Contributions are welcome! Feel free to submit pull requests, report issues, or suggest features.

## License

Vavo is licensed under the MIT License. See `LICENSE` for more details.

## Contact

For questions or support, reach out via GitHub issues or discussions.
