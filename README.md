# Bounding Box Generation for Bevy

[![CI](https://github.com/aevyrie/bevy_mod_bounding/workflows/CI/badge.svg?branch=master)](https://github.com/aevyrie/bevy_mod_bounding/actions?query=workflow%3A%22CI%22+branch%3Amaster)
[![crates.io](https://img.shields.io/crates/v/bevy_mod_bounding)](https://crates.io/crates/bevy_mod_bounding)
[![docs.rs](https://docs.rs/bevy_mod_picking/badge.svg)](https://docs.rs/bevy_mod_bounding)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-main-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)

Unofficial plugin for generating bounding volumes.

![demo](docs/animation.png)

## Status

| Status | Bounding Volume |
|:-:|----------------------------|
| ✅ | Bounding Sphere            |
| ✅ | Axis Aligned Bounding Box  |
| ✅ | Oriented Bounding Box      |
| ❌ | Convex Hull                |

## Demo

Run the demo with:

```shell
cargo run --example demo --features="ex"
```
