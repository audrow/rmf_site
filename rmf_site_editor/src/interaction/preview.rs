/*
 * Copyright (C) 2022 Open Source Robotics Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
*/

use bevy::{
    prelude::*,
    render::{
        camera::{Projection, RenderTarget},
        view::RenderLayers,
    },
    window::{PresentMode, WindowClosed, WindowResolution},
};

use rmf_site_format::{NameInSite, PhysicalCameraProperties, PreviewableMarker};

/// Instruction to spawn a preview for the given entity
/// TODO None to encode "Clear all"
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SpawnPreview {
    pub entity: Option<Entity>,
}

impl SpawnPreview {
    pub fn new(entity: Option<Entity>) -> Self {
        Self { entity }
    }
}

fn create_camera_window(
    commands: &mut Commands,
    entity: Entity,
    camera_name: &String,
    camera_properties: &PhysicalCameraProperties,
) {
    commands
        .entity(entity)
        .insert(Window {
            present_mode: PresentMode::AutoNoVsync,
            resolution: WindowResolution::new(camera_properties.width as f32, camera_properties.height as f32),
            title: "Camera preview: ".to_string() + camera_name,
            ..default()
        })
        .insert(Camera {
            target: RenderTarget::Window(Window::Entity(entity)),
            is_active: true,
            ..default()
        })
        .insert(RenderLayers::layer(0));
}

// TODO consider renaming this manage_camera_previews and
// use other systems for other previews
pub fn manage_previews(
    mut commands: Commands,
    mut preview_events: EventReader<SpawnPreview>,
    previewable: Query<
        (&Children, &NameInSite, &PhysicalCameraProperties),
        (With<PreviewableMarker>, Without<Window>),
    >,
    mut camera_children: Query<(Entity, &mut Projection), With<Camera>>,
) {
    for event in preview_events.iter() {
        match event.entity {
            None => { // TODO clear all previews
            }
            Some(e) => {
                if let Ok((children, camera_name, camera_properties)) = previewable.get(e) {
                    // Get the child of the root entity
                    // Assumes each physical camera has one and only one child for the sensor
                    if let Ok((child_entity, mut projection)) =
                        camera_children.get_mut(children[0])
                    {
                        // Update the camera to the right fov first
                        if let Projection::Perspective(perspective_projection) =
                            &mut (*projection)
                        {
                            let aspect_ratio = (camera_properties.width as f32)
                                / (camera_properties.height as f32);
                            perspective_projection.fov =
                                camera_properties.horizontal_fov.radians() / aspect_ratio;
                        }
                        create_camera_window(
                            &mut commands,
                            child_entity,
                            &camera_name,
                            &camera_properties,
                        );
                    }
                }
            }
        }
    }
}

pub fn update_physical_camera_preview(
    updated_camera_previews: Query<
        (&Children, &PhysicalCameraProperties, &mut Window),
        Changed<PhysicalCameraProperties>,
    >,
    mut camera_children: Query<&mut Projection, With<Camera>>,
) {
    for (children, camera_properties, window) in updated_camera_previews.iter() {
        // Update fov first
        if let Ok(mut projection) = camera_children.get_mut(children[0]) {
            if let Projection::Perspective(perspective_projection) = &mut (*projection) {
                let aspect_ratio =
                    (camera_properties.width as f32) / (camera_properties.height as f32);
                perspective_projection.fov =
                    camera_properties.horizontal_fov.radians() / aspect_ratio;
            }
        }
        window.set_resolution(
            camera_properties.width as f32,
            camera_properties.height as f32,
        );
    }
}

pub fn handle_preview_window_close(
    mut commands: Commands,
    preview_windows: Query<(Entity, With<Window>)>,
    mut closed_windows: EventReader<WindowClosed>,
) {
    for closed in closed_windows.iter() {
        for e in &preview_windows {
            if e == closed.id {
                commands.entity(e).remove::<Window>();
            }
        }
    }
}
