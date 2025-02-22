// Entry point for non-wasm
#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    run().await;
}

use three_d::*;

pub async fn run() {
    let window = Window::new(WindowSettings {
        title: "PBR!".to_string(),
        min_size: (512, 512),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();
    let context = window.gl().unwrap();

    let mut camera = Camera::new_perspective(
        &context,
        window.viewport().unwrap(),
        vec3(-3.0, 1.0, 2.5),
        vec3(0.0, 0.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    )
    .unwrap();
    let mut control = OrbitControl::new(*camera.target(), 1.0, 100.0);
    let mut gui = three_d::GUI::new(&context).unwrap();

    let mut loaded = three_d_asset::io::load_async(&[
        "examples/assets/gltf/DamagedHelmet.glb", // Source: https://github.com/KhronosGroup/glTF-Sample-Models/tree/master/2.0
        "examples/assets/chinese_garden_4k.hdr",  // Source: https://polyhaven.com/
    ])
    .await
    .unwrap();

    let environment_map = loaded.deserialize("chinese").unwrap();
    let skybox = Skybox::new_from_equirectangular(&context, &environment_map).unwrap();

    let mut cpu_model: CpuModel = loaded.deserialize("DamagedHelmet").unwrap();
    cpu_model
        .geometries
        .iter_mut()
        .for_each(|m| m.compute_tangents().unwrap());
    let model = Model::<PhysicalMaterial>::new(&context, &cpu_model)
        .unwrap()
        .remove(0);

    let light =
        AmbientLight::new_with_environment(&context, 1.0, Color::WHITE, skybox.texture()).unwrap();

    // main loop
    let mut normal_map_enabled = true;
    let mut occlusion_map_enabled = true;
    let mut metallic_roughness_enabled = true;
    let mut albedo_map_enabled = true;
    let mut emissive_map_enabled = true;
    window
        .render_loop(move |mut frame_input| {
            let mut panel_width = 0.0;
            gui.update(&mut frame_input, |gui_context| {
                use three_d::egui::*;
                SidePanel::left("side_panel").show(gui_context, |ui| {
                    ui.heading("Debug Panel");
                    ui.checkbox(&mut albedo_map_enabled, "Albedo map");
                    ui.checkbox(&mut metallic_roughness_enabled, "Metallic roughness map");
                    ui.checkbox(&mut normal_map_enabled, "Normal map");
                    ui.checkbox(&mut occlusion_map_enabled, "Occlusion map");
                    ui.checkbox(&mut emissive_map_enabled, "Emissive map");
                });
                panel_width = gui_context.used_size().x as f64;
            })
            .unwrap();

            let viewport = Viewport {
                x: (panel_width * frame_input.device_pixel_ratio) as i32,
                y: 0,
                width: frame_input.viewport.width
                    - (panel_width * frame_input.device_pixel_ratio) as u32,
                height: frame_input.viewport.height,
            };
            camera.set_viewport(viewport).unwrap();
            control
                .handle_events(&mut camera, &mut frame_input.events)
                .unwrap();

            frame_input
                .screen()
                .clear(ClearState::color_and_depth(0.5, 0.5, 0.5, 1.0, 1.0))
                .unwrap()
                .render(&camera, &[&skybox], &[])
                .unwrap()
                .write(|| {
                    let material = PhysicalMaterial {
                        name: model.material.name.clone(),
                        albedo: model.material.albedo,
                        albedo_texture: if albedo_map_enabled {
                            model.material.albedo_texture.clone()
                        } else {
                            None
                        },
                        metallic: model.material.metallic,
                        roughness: model.material.roughness,
                        metallic_roughness_texture: if metallic_roughness_enabled {
                            model.material.metallic_roughness_texture.clone()
                        } else {
                            None
                        },
                        normal_scale: model.material.normal_scale,
                        normal_texture: if normal_map_enabled {
                            model.material.normal_texture.clone()
                        } else {
                            None
                        },
                        occlusion_strength: model.material.occlusion_strength,
                        occlusion_texture: if occlusion_map_enabled {
                            model.material.occlusion_texture.clone()
                        } else {
                            None
                        },
                        emissive: if emissive_map_enabled {
                            model.material.emissive
                        } else {
                            Color::BLACK
                        },
                        emissive_texture: if emissive_map_enabled {
                            model.material.emissive_texture.clone()
                        } else {
                            None
                        },
                        render_states: model.material.render_states,
                        is_transparent: model.material.is_transparent,
                        lighting_model: LightingModel::Cook(
                            NormalDistributionFunction::TrowbridgeReitzGGX,
                            GeometryFunction::SmithSchlickGGX,
                        ),
                    };
                    model.render_with_material(&material, &camera, &[&light])?;
                    gui.render()?;
                    Ok(())
                })
                .unwrap();

            FrameOutput::default()
        })
        .unwrap();
}
