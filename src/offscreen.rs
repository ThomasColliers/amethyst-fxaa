use amethyst::renderer::{
    bundle::{Target, RenderPlan, RenderPlugin, ImageOptions, OutputColor, TargetPlanOutputs },
    Backend, Factory,
    Kind,
};
use amethyst::{
    prelude::*,
    window::ScreenDimensions,
    error::Error,
};
use rendy::{
    hal::{ format::Format, command::ClearValue, command::ClearDepthStencil },
};

// plugin
#[derive(Default, Debug)]
pub struct RenderOffscreen {
    target: Target,
    dirty: bool,
    dimensions: Option<ScreenDimensions>,
}

impl<B: Backend> RenderPlugin<B> for RenderOffscreen {
    fn should_rebuild(&mut self, world: &World) -> bool {
        let new_dimensions = world.try_fetch::<ScreenDimensions>();
        if self.dimensions.as_ref() != new_dimensions.as_deref() {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| (*d).clone());
            return false;
        }
        self.dirty
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World
    ) -> Result<(), Error> {
        self.dirty = false;

        // add the offscreen target
        let dimensions = self.dimensions.as_ref().unwrap();
        let kind = Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);
        let depth_options = ImageOptions {
            kind: kind,
            levels: 1,
            format: Format::D32Sfloat,
            clear: Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        };
        plan.add_root(Target::Custom("offscreen"));
        plan.define_pass(
            Target::Custom("offscreen"),
            TargetPlanOutputs {
                colors: vec![OutputColor::Image(ImageOptions {
                    kind:kind,
                    levels: 1,
                    format: Format::Rgba8Unorm,
                    clear: Some(ClearValue::Color([0.0, 0.0, 0.0, 1.0].into())),
                })],
                depth: Some(depth_options)
            }
        )?;

        Ok(())
    }
}