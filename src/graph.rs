use amethyst::{
    ecs::{
        ReadExpect, SystemData, World,
    },
    renderer::{
        pass::DrawShadedDesc,
        types::DefaultBackend,
        Factory, Format, GraphBuilder, GraphCreator, Kind,
        RenderGroupDesc, SubpassBuilder,
        rendy::graph::render::{SimpleGraphicsPipeline,RenderGroupBuilder},
    },
    window::{ScreenDimensions, Window },
};
//use crate::fxaa::DrawFXAADesc;

#[derive(Default)]
pub struct RenderGraph {
    dimensions: Option<ScreenDimensions>,
    dirty: bool,
}

impl GraphCreator<DefaultBackend> for RenderGraph {
    // indicate if it should be rebuilt
    fn rebuild(&mut self, world: &World) -> bool {
        // Rebuild when dimensions change, but wait until at least two frames have the same.
        let new_dimensions = world.try_fetch::<ScreenDimensions>();
        use std::ops::Deref;
        if self.dimensions.as_ref() != new_dimensions.as_deref() {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.deref().clone());
            return false;
        }
        self.dirty
    }


    // This is the core of a RenderGraph, which is building the actual graph with subpasses and target
    // images.
    fn builder(
        &mut self,
        factory: &mut Factory<DefaultBackend>,
        world: &World,
    ) -> GraphBuilder<DefaultBackend, World> {
        use amethyst::renderer::rendy::{
            graph::present::PresentNode,
            hal::command::{ClearDepthStencil, ClearValue},
        };

        self.dirty = false;

        // Retrieve a reference to the target window, which is created by the WindowBundle
        let window = <ReadExpect<'_, Window>>::fetch(world);
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind = Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);

        // Create a new drawing surface in our window
        let surface = factory.create_surface(&window);
        let surface_format = factory.get_surface_format(&surface);

        // Begin building our RenderGraph
        let mut graph_builder = GraphBuilder::new();

        // HDR color output
        let hdr = graph_builder.create_image(
            Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1),
            1,
            Format::Rgba8Unorm,
            Some(ClearValue::Color([0.0, 0.0, 0.0, 1.0].into())),
        );

        // Color and depth outputs
        let color = graph_builder.create_image(
            window_kind,
            1,
            surface_format,
            Some(ClearValue::Color([0.0, 0.0, 0.0, 1.0].into())),
        );
        let depth = graph_builder.create_image(
            window_kind,
            1,
            Format::D32Sfloat,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        // Main render pass
        let main_pass = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawShadedDesc::default().builder())
                .with_color(hdr)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        // FXAA pass
        let fxaa_pass = graph_builder.add_node(
            crate::fxaa::Pipeline::builder()
                .with_image(hdr)
                .into_subpass()
                .with_dependency(main_pass)
                .with_color(color)
                .into_pass()
        );

        // Finally, add the pass to the graph
        let _present = graph_builder
            .add_node(PresentNode::builder(factory, surface, color).with_dependency(fxaa_pass));

        graph_builder
    }
}