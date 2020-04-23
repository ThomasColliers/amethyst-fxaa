mod fxaa;
mod graph;

use amethyst::{
    assets::{
        PrefabLoader, PrefabLoaderSystemDesc, RonFormat, PrefabData, ProgressCounter,
        Processor,
    },
    core::{
        Transform,TransformBundle,
        shrev::{EventChannel, ReaderId},
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
    derive::SystemDesc,
    ecs::prelude::{Read, Write, System, SystemData },
    input::{InputEvent},
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

        // set fxaa enabled
        data.world.insert(FxaaSettings{ enabled:true });
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
        .with_system_desc(FxaaSystemDesc::default(), "fxaa", &["input_system"])
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

// resource to keep track if fxaa is enabled
#[derive(Default)]
pub struct FxaaSettings {
    pub enabled: bool,
}

// simple system to toggle fxaa
#[derive(SystemDesc)]
#[system_desc(name(FxaaSystemDesc))]
pub struct FxaaSystem {
    #[system_desc(event_channel_reader)]
    event_reader: ReaderId<InputEvent<StringBindings>>,
}

impl FxaaSystem {
    pub fn new(event_reader: ReaderId<InputEvent<StringBindings>>) -> Self {
        Self { event_reader:event_reader }
    }
}

impl<'s> System<'s> for FxaaSystem {
    type SystemData = (
        Read<'s, EventChannel<InputEvent<StringBindings>>>,
        Write<'s, FxaaSettings>,
    );

    fn run(&mut self, (events,mut fxaa_settings): Self::SystemData) {
        for event in events.read(&mut self.event_reader) {
            match event {
                InputEvent::KeyPressed { key_code:VirtualKeyCode::F, .. } => {
                    fxaa_settings.enabled = !fxaa_settings.enabled;
                },
                _ => (),
            };
        }
    }
}