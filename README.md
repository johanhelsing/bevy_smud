# bevy_smud

Sdf 2d shape rendering for bevy.

Some people might be offended by generating lots of shaders at runtime, minimizing draw calls and all that stuff... but you know what they say:

> When everything is a nail, all you need is a hammer!

However, if you keep the number of different sdf shape and fill combinations relatively low it's still blazingly fast.

## Thanks!

Little of this crate is original work. It's mostly a mishmash of [`bevy_sprite`](https://github.com/bevyengine/bevy/tree/main/crates/bevy_sprite) and [Inigo Quilez sdf rendering primitives](https://iquilezles.org/www/articles/distfunctions2d/distfunctions2d.htm) ported to wgsl. I just the two together in a way I found convenient.

## Usage

See [examples](examples). In particular, the [basic](examples/basic.rs) should be a good place to start.
