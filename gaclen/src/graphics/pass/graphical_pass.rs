use super::builder::GraphicalPassBuilder;

use vulkano::descriptor::PipelineLayoutAbstract;
use vulkano::descriptor::descriptor_set::{FixedSizeDescriptorSetsPool, PersistentDescriptorSet, PersistentDescriptorSetBuilder};
use vulkano::format::ClearValue;
use vulkano::framebuffer::{Framebuffer, FramebufferBuilder};
use vulkano::framebuffer::{AttachmentDescription, PassDescription, RenderPassDesc, RenderPassDescClearValues, PassDependencyDescription};
use vulkano::image::ImageLayout;

use std::sync::Arc;

/// A GraphicalPass defines the device configuration used to execute draw commands.
pub struct GraphicalPass<P : ?Sized> {
	pub(in crate::graphics) pipeline: Arc<P>,
}

impl GraphicalPass<()> {
	/// Begin building a GraphicalPass.
	pub fn start() -> GraphicalPassBuilder<(), (), (), (), ()> { GraphicalPassBuilder::new() }
}

#[cfg(feature="expose-underlying-vulkano")]
impl<P: ?Sized> GraphicalPass<P> {
	#[inline]
	pub fn pipeline(&self) -> Arc<P> { self.pipeline }
}

impl<P : ?Sized> GraphicalPass<P>
{
	/// Start building a new persistent descriptor set.
	/// 
	/// # Panic
	/// 
	/// Panics if the pipeline does not expect a descriptor set at given index.
	pub fn start_persistent_descriptor_set(&self, index: usize) -> PersistentDescriptorSetBuilder<()>
	where
		P : PipelineLayoutAbstract,
	{
		PersistentDescriptorSet::start(self.pipeline.descriptor_set_layout(index).unwrap().clone())
	}

	/// Allocate a pool of fixed-size descriptor sets.
	/// 
	/// # Panic
	/// 
	/// Panics if the pipeline does not expect a descriptor set at given index.
	pub fn create_fixed_size_descriptor_set_pool(&self, index: usize) -> FixedSizeDescriptorSetsPool
	where
		P : PipelineLayoutAbstract,
	{
		FixedSizeDescriptorSetsPool::new(self.pipeline.descriptor_set_layout(index).unwrap().clone())
	}

	/// Start building a framebuffer for this pass.
	pub fn start_framebuffer(&self) -> FramebufferBuilder<Arc<P>, ()> {
		Framebuffer::start(self.pipeline.clone())
	}
}

#[derive(Debug, Clone)]
pub(in crate::graphics) struct GraphicalRenderPassDescription {
	/// Image attachments of the render pass.
	pub attachments: Vec<AttachmentDescription>,
	/// Depth stencil attachment index.
	pub depth_attachment: Option<usize>,
}

unsafe impl RenderPassDesc for GraphicalRenderPassDescription {
	#[inline]
	fn num_attachments(&self) -> usize { self.attachments.len() }
	
	#[inline]
	fn attachment_desc(&self, num: usize) -> Option<AttachmentDescription> {
		match num < self.attachments.len() {
			true => Some(self.attachments[num].clone()),
			false => None,
		}
	}

	#[inline]
	fn num_subpasses(&self) -> usize { 1 }

	#[inline]
	fn subpass_desc(&self, num: usize) -> Option<PassDescription> {
		if num == 0 {
			let color_attachments = {
				if let Some(depth_index) = self.depth_attachment {
					let mut color_attachments = Vec::with_capacity(self.attachments.len() - 1);
					for i in 0..depth_index {
						color_attachments.push((i, ImageLayout::ColorAttachmentOptimal));
					}
					for i in depth_index + 1 .. self.attachments.len() {
						color_attachments.push((i, ImageLayout::ColorAttachmentOptimal));
					}
					color_attachments
				} else {
					let mut color_attachments = Vec::with_capacity(self.attachments.len());
					for i in 0..self.attachments.len() {
						color_attachments.push((i, ImageLayout::ColorAttachmentOptimal))
					};
					color_attachments
				}
			};
			let depth_stencil = match self.depth_attachment {
				Some(index) => Some((index, ImageLayout::DepthStencilAttachmentOptimal)),
				None => None,
			};
			Some(PassDescription{
				color_attachments,
				depth_stencil,
				input_attachments: Vec::default(),
				resolve_attachments: Vec::default(),
				preserve_attachments: Vec::default(),
			})
		} else {
			None
		}
	}

	fn num_dependencies(&self) -> usize { 0 }

	fn dependency_desc(&self, _: usize) -> Option<PassDependencyDescription> { None }
}

unsafe impl RenderPassDescClearValues<Vec<ClearValue>> for GraphicalRenderPassDescription {
	// TODO/vulkano: find out what this is supposed to do.
	fn convert_clear_values(&self, values: Vec<ClearValue>) -> Box<dyn Iterator<Item = ClearValue>> { Box::new(values.into_iter()) }
}
