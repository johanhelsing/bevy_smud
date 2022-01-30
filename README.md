# bevy_smud

Sdf 2d shape rendering for [Bevy](https://bevyengine.org).

![screenshot of a bird drawn with bevy_smud](https://johanhelsing.studio/assets/bevy_smud.png)

Bevy smud is a way to conveniently construct and render sdf shapes with Bevy.

Given a shape function/expression, a fill type, it generates shaders at run-time.

That might rub some people the wrong way... minimizing draw calls and all that stuff. However, if you keep the number of different sdf and fill combinations relatively low it's still blazingly fast. My machine easily handles 100k shapes at 60 fps, even with 40 different shape/fill combinations in randomized order (see [gallery](examples/gallery) example).

## Usage

See [examples](examples). In particular, the [basic](examples/basic.rs) example should be a good place to start.

There are many built-in sdf primitives, which are automatically imported when using the single expression or body shorthand for adding sdfs.

In .wgsl files, the shape library can be imported through the [`bevy_smud::shapes`](assets/shapes.wgsl) import. Most of the shapes are direct ports of the ones on [this page](https://iquilezles.org/www/articles/distfunctions2d/distfunctions2d.htm), which includes screenshots of the various shapes.

For instance:

```wgsl
sd_circle(p, 50.)
```

Will return the distance from p to the edge of a circle with radius 50, with negative values being inside the circle.

So to put together a shape, you can do:

```rust
fn spawn_circle(
    mut commands: Commands,
    mut shaders: ResMut<Assets<Shader>>,
) {
    let circle = shaders.add_sdf_expr("sd_circle(p, 50.)");
T
    commands.spawn_bundle(ShapeBundle {
        shape: SmudShape {
            color: Color::TOMATO,
            sdf: circle,
            frame: Frame::Quad(55.),
            ..Default::default()
        },
        ..Default::default()
    });
}
```

Make sure you reuse the shaders, i.e. don't call `add_sdf_expr` every frame.

Other than that, make sure you understand how to combine shapes, use symmetries and change domains. For instance, the [bevy](assets/bevy.wgsl) above is built up of circles, ellipses, and a vesica for the beak.

## Word of caution

This crate should still be considered highly experimental.

If you want something more finished, you should probably check out [bevy_prototype_lyon](https://github.com/Nilirad/bevy_prototype_lyon).

## Bevy version support

The `main` branch targets the latest bevy release.

I intend to support the `main` branch of Bevy in the `bevy-main` branch.

|bevy|bevy_smud|
|---|---|
|0.6|main|

## Thanks!

Little of this crate is original work. It's mostly a mishmash of [`bevy_sprite`](https://github.com/bevyengine/bevy/tree/main/crates/bevy_sprite) and [Inigo Quilez sdf rendering primitives](https://iquilezles.org/www/articles/distfunctions2d/distfunctions2d.htm) ported to wgsl. I just put the two together in a way I found convenient.