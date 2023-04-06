use bevy::{
    prelude::*,
    sprite::{Material2d, Material2dKey, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
    render::{render_resource::{RenderPipelineDescriptor, SpecializedMeshPipelineError, AsBindGroup, ShaderRef, TextureDimension, TextureUsages, TextureFormat, TextureDescriptor, Extent3d},
    mesh::MeshVertexBufferLayout, texture::BevyDefault, view::RenderLayers, camera::RenderTarget},
    reflect::TypeUuid,
    window::WindowResized,
    ecs::query::QuerySingleError
};

#[derive(Component)] pub struct PostProcessCamera;

#[derive(Default, Resource)]
pub struct PostProcessConfig
{
    pub material_handle: Handle<PostProcessingMaterial>,
    pub image_handle: Handle<Image>,
    pub quad_entity: Option<Entity>
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "2fe1c4d9-8d0f-4321-a6eb-2eade51b647c"]
#[bind_group_data(TrippyKey)]
pub struct PostProcessingMaterial
{
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,
    pub is_trippy: bool
}

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct TrippyKey {
    is_trippy: bool,
}

impl From<&PostProcessingMaterial> for TrippyKey {
    fn from(material: &PostProcessingMaterial) -> Self {
        Self {
            is_trippy: material.is_trippy
        }
    }
}

impl Material2d for PostProcessingMaterial
{
    fn fragment_shader() -> ShaderRef { "shaders/trippy.wgsl".into() }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError>
    {
        if key.bind_group_data.is_trippy {
            let fragment = descriptor.fragment.as_mut().unwrap();
            fragment.shader_defs.push("IS_TRIPPY".into());
        }
        Ok(())
    }
}

fn setup_post_processing(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window>,
    mut post_processing_materials: ResMut<Assets<PostProcessingMaterial>>,
    mut post_process_config: ResMut<PostProcessConfig>
)
{
    let window = windows.single();

    let size = Extent3d {
        width: window.resolution.physical_width(),
        height: window.resolution.physical_height(),
        ..default()
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::bevy_default(),
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);

    let image_handle = images.add(image);

    let render_layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

    let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        size.width as f32,
        size.height as f32,
    ))));

    let material_handle = post_processing_materials.add(PostProcessingMaterial {
        source_image: image_handle.clone(),
        is_trippy: false
    });

    let quad_entity = commands.spawn((
        MaterialMesh2dBundle {
            mesh: quad_handle.into(),
            material: material_handle.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                ..default()
            },
            ..default()
        },
        render_layer
    ))
    .id();

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1,
                ..default()
            },
            ..Camera2dBundle::default()
        },
        render_layer,
    ));

    *post_process_config = PostProcessConfig {
        material_handle,
        image_handle,
        quad_entity: Some(quad_entity)
    }
}

fn setup_post_process_camera(
    mut cameras: Query<&mut Camera, Added<PostProcessCamera>>,
    post_process_config: Res<PostProcessConfig>,
)
{
    let mut camera = match cameras.get_single_mut() {
        Ok(camera) => camera,
        Err(QuerySingleError::MultipleEntities(_)) => panic!("Multiple cameras detected!"),
        Err(QuerySingleError::NoEntities(_)) => { return ; },
    };

    camera.target = RenderTarget::Image(post_process_config.image_handle.clone());
}

fn fix_resize(
    mut commands: Commands,
    mut ev_resize: EventReader<WindowResized>,
    post_process_config: Res<PostProcessConfig>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    windows: Query<&Window>,
    mut image_events: EventWriter<AssetEvent<Image>>,
    mut post_processing_materials: ResMut<Assets<PostProcessingMaterial>>
)
{
    if let Some(ev) = ev_resize.iter().last() {
        let window = windows.get(ev.window).unwrap();

        let size = Extent3d {
            width: window.resolution.physical_width(),
            height: window.resolution.physical_height(),
            ..default()
        };

        let image = images.get_mut(&post_process_config.image_handle).unwrap();
        image.resize(size);

        let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
            size.width as f32,
            size.height as f32,
        ))));

        commands
            .get_entity(post_process_config.quad_entity.unwrap())
            .unwrap()
            .remove::<Mesh2dHandle>()
            .insert(
                Mesh2dHandle(quad_handle.into())
            );

        image_events.send(AssetEvent::Modified {
            handle: post_process_config.image_handle.clone(),
        });
        post_processing_materials.get_mut(&post_process_config.material_handle);
    }
}

pub struct PostProcessingPlugin;

impl Plugin for PostProcessingPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .init_resource::<PostProcessConfig>()
            .add_startup_system(setup_post_processing)
            .add_system(setup_post_process_camera)
            .add_system(fix_resize)
            .add_plugin(Material2dPlugin::<PostProcessingMaterial>::default())
        ;
    }
}

