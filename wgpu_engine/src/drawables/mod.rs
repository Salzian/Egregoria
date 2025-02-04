use crate::GfxContext;
use wgpu::RenderPass;

mod instanced_mesh;
mod lit_mesh;
mod multispritebatch;
mod spritebatch;
pub mod terrain;
mod water;

pub use instanced_mesh::*;
pub use lit_mesh::*;
pub use multispritebatch::*;
pub use spritebatch::*;
pub use water::*;

use std::sync::Arc;

pub type IndexType = u32;

pub trait Drawable: Sync {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>);

    #[allow(unused)]
    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_map: bool,
        proj: &'a wgpu::BindGroup,
    ) {
    }
}

impl<T: Drawable + Send> Drawable for Arc<T> {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        let s: &T = self;
        s.draw(gfx, rp);
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_map: bool,
        proj: &'a wgpu::BindGroup,
    ) {
        let s: &T = self;
        s.draw_depth(gfx, rp, shadow_map, proj);
    }
}

impl<T: Drawable> Drawable for Option<T> {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        if let Some(s) = self {
            s.draw(gfx, rp);
        }
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_map: bool,
        proj: &'a wgpu::BindGroup,
    ) {
        if let Some(s) = self {
            s.draw_depth(gfx, rp, shadow_map, proj);
        }
    }
}

impl<T: Drawable> Drawable for [T] {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        for s in self {
            s.draw(gfx, rp);
        }
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_map: bool,
        proj: &'a wgpu::BindGroup,
    ) {
        for s in self {
            s.draw_depth(gfx, rp, shadow_map, proj);
        }
    }
}

impl<T: Drawable, U: Drawable> Drawable for (T, U) {
    fn draw<'a>(&'a self, gfx: &'a GfxContext, rp: &mut RenderPass<'a>) {
        self.0.draw(gfx, rp);
        self.1.draw(gfx, rp);
    }

    fn draw_depth<'a>(
        &'a self,
        gfx: &'a GfxContext,
        rp: &mut RenderPass<'a>,
        shadow_map: bool,
        proj: &'a wgpu::BindGroup,
    ) {
        self.0.draw_depth(gfx, rp, shadow_map, proj);
        self.1.draw_depth(gfx, rp, shadow_map, proj);
    }
}
