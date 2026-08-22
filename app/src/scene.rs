use fyrox::{
    asset::untyped::ResourceKind,
    core::{
        algebra::{Matrix4, UnitQuaternion, Vector3},
        color::Color,
        pool::Handle,
        uuid::Uuid,
    },
    engine::Engine,
    graph::SceneGraph,
    material::{Material, MaterialResource},
    scene::{
        base::BaseBuilder,
        camera::CameraBuilder,
        light::{directional::DirectionalLightBuilder, BaseLightBuilder},
        mesh::{
            surface::{SurfaceBuilder, SurfaceData, SurfaceResource},
            MeshBuilder,
        },
        node::Node,
        transform::TransformBuilder,
        Scene,
    },
};

/// A minimal 3D scene demo: a colored cube that can spin, with a camera that can
/// orbit it. Rendered full-window behind the UI (this Fyrox version has no
/// viewport widget). The scene only renders when `scene_need_render` is set by
/// the update pass — never on a timer — so a static scene costs zero frames.
pub struct Scene3D {
    pub scene: Handle<Scene>,
    pub cube: Handle<Node>,
    pub camera: Handle<Node>,
    pub spin: bool,
    pub orbit: bool,
    pub always_spin: bool,
    angle: f32,
    orbit_angle: f32,
}

impl Scene3D {
    pub fn build(engine: &mut Engine) -> Self {
        let mut scene = Scene::new();

        // Camera: positioned at (0, 1.2, 4), pitched slightly down toward the origin.
        let camera = CameraBuilder::new(
            BaseBuilder::new().with_local_transform(
                TransformBuilder::new()
                    .with_local_position(Vector3::new(0.0, 1.2, 4.0))
                    .with_local_rotation(UnitQuaternion::from_euler_angles(0.29, 0.0, 0.0))
                    .build(),
            ),
        )
        .with_fov(45.0f32.to_radians())
        .with_z_far(100.0)
        .build(&mut scene.graph)
        .transmute();

        let _light = DirectionalLightBuilder::new(BaseLightBuilder::new(
            BaseBuilder::new().with_local_transform(
                TransformBuilder::new()
                    .with_local_position(Vector3::new(0.0, 5.0, 0.0))
                    .with_local_rotation(UnitQuaternion::from_euler_angles(-1.0, 0.0, 0.0))
                    .build(),
            ),
        ))
        .build(&mut scene.graph);

        // Colored cube material.
        let mut cube_material = Material::standard();
        cube_material.set_property("diffuseColor", Color::opaque(200, 120, 40));

        let cube = MeshBuilder::new(
            BaseBuilder::new().with_local_transform(
                TransformBuilder::new()
                    .with_local_position(Vector3::new(0.0, 0.0, 0.0))
                    .build(),
            ),
        )
        .with_surfaces(vec![SurfaceBuilder::new(SurfaceResource::new_ok(
            Uuid::new_v4(),
            ResourceKind::Embedded,
            SurfaceData::make_cube(Matrix4::identity()),
        ))
        .with_material(MaterialResource::new_ok(
            Uuid::new_v4(),
            Default::default(),
            cube_material,
        ))
        .build()])
        .build(&mut scene.graph)
        .transmute();

        let scene_handle = engine.scenes.add(scene);

        Self {
            scene: scene_handle,
            cube,
            camera,
            spin: false,
            orbit: false,
            always_spin: false,
            angle: 0.0,
            orbit_angle: 0.0,
        }
    }

    /// Advances the animation according to the toggles. Returns true when the
    /// scene changed and needs a render.
    pub fn update(&mut self, dt: f32) -> bool {
        let (spin, orbit, always) = (self.spin, self.orbit, self.always_spin);
        if !spin && !orbit && !always {
            return false;
        }

        if always {
            // Continuous animation: spin + orbit every frame.
            self.angle += dt;
            self.orbit_angle += dt * 0.5;
        } else {
            if spin {
                self.angle += dt;
            }
            if orbit {
                self.orbit_angle += dt * 0.5;
            }
        }
        true
    }

    /// Applies the current angles to the scene graph (called when a render is
    /// going to happen).
    pub fn apply(&mut self, engine: &mut Engine) {
        if let Ok(scene) = engine.scenes.try_get_mut(self.scene) {
            if let Ok(cube) = scene.graph.try_get_mut(self.cube) {
                cube.local_transform_mut()
                    .set_rotation(UnitQuaternion::from_euler_angles(0.0, self.angle, 0.0));
            }
            if let Ok(camera) = scene.graph.try_get_mut(self.camera) {
                let x = 4.0 * self.orbit_angle.cos();
                let z = 4.0 * self.orbit_angle.sin();
                let yaw = self.orbit_angle - std::f32::consts::FRAC_PI_2;
                camera
                    .local_transform_mut()
                    .set_position(Vector3::new(x, 1.2, z));
                camera
                    .local_transform_mut()
                    .set_rotation(UnitQuaternion::from_euler_angles(0.29, yaw, 0.0));
            }
        }
    }
}
