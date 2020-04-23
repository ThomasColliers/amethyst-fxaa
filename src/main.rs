mod fxaa;
mod offscreen;
mod graph;

use amethyst::{
    assets::{
        PrefabLoader, PrefabLoaderSystemDesc, RonFormat, PrefabData, ProgressCounter,
        Processor,
    },
    core::{
        Transform,TransformBundle,
    },
    derive::{PrefabData},
    ecs::{Entity, WorldExt},
    prelude::{
        Application, Builder, GameData, GameDataBuilder, SimpleState, SimpleTrans, StateData,
        StateEvent, Trans,
    },
    renderer::{
        camera::{CameraPrefab},
        formats::GraphicsPrefab,
        mtl::Material,
        light::LightPrefab,
        rendy::mesh::{Normal, Position, Tangent, TexCoord},
        types::DefaultBackend,
        MeshProcessorSystem, TextureProcessorSystem, visibility::VisibilitySortingSystem,
        RenderingSystem
    },
    utils::{
        application_root_dir, 
        auto_fov::{AutoFov, AutoFovSystem},
    },
    input::{
        is_close_requested, is_key_down, InputBundle, StringBindings
    },
    controls::{ArcBallControlBundle, ControlTagPrefab},
    winit::VirtualKeyCode,
    Error,
    window::{WindowBundle},
};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, PrefabData, Serialize)]
#[serde(default)]
struct ScenePrefab {
    graphics: Option<GraphicsPrefab<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>>,
    transform: Option<Transform>,
    light: Option<LightPrefab>,
    camera: Option<CameraPrefab>,
    auto_fov: Option<AutoFov>,
    control_tag: Option<ControlTagPrefab>,
}

#[derive(Default)]
struct MainState;

impl SimpleState for MainState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        // load the scene from the ron file
        let handle = data.world.exec(|loader: PrefabLoader<'_, ScenePrefab>| {
            loader.load("scene.ron", RonFormat, ())
        });
        data.world.create_entity().with(handle).build();
    }

    fn handle_event(&mut self, _data: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Window(ref event) = event {
            if is_close_requested(event) || is_key_down(event, VirtualKeyCode::Escape) {
                Trans::Quit
            } else {
                Trans::None
            }
        } else {
            Trans::None
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("assets");
    let config_dir = app_root.join("config");
    let display_config_path = config_dir.join("display.ron");

    let game_data = GameDataBuilder::default()
        .with_system_desc(
            PrefabLoaderSystemDesc::<ScenePrefab>::default(), 
            "scene_loader", 
            &[]
        )
        .with_bundle(TransformBundle::new())?
        .with(AutoFovSystem::new(), "auto_fov", &["scene_loader"])
        .with_bundle(
            InputBundle::<StringBindings>::new(),
        )?
        .with_bundle(ArcBallControlBundle::<StringBindings>::new())?
        .with(
            VisibilitySortingSystem::new(),
            "visibility_sorting_system",
            &[],
        )
        .with(
            MeshProcessorSystem::<DefaultBackend>::default(),
            "mesh_processor",
            &[],
        )
        .with(
            TextureProcessorSystem::<DefaultBackend>::default(),
            "texture_processor",
            &[],
        )
        .with(Processor::<Material>::new(), "material_processor", &[])
        .with_bundle(WindowBundle::from_config_path(display_config_path)?)?
        .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            graph::RenderGraph::default(),
        ));

    let mut game = Application::new(assets_dir, MainState::default(), game_data)?;
    game.run();

    Ok(())
}
