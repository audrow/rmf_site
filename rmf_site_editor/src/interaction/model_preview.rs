/*
 * Copyright (C) 2023 Open Source Robotics Foundation
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

use crate::interaction::MODEL_PREVIEW_LAYER;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::{camera::RenderTarget, primitives::Aabb, view::RenderLayers};
use bevy_egui::{egui::TextureId, EguiContext};

#[derive(Resource)]
pub struct ModelPreviewCamera {
    pub camera_entity: Entity,
    pub egui_handle: TextureId,
    pub model_entity: Entity,
}

pub struct ModelPreviewPlugin;

// TODO(luca) implement this system to scale the view based on the model's Aabb
fn scale_preview_for_model_bounding_box(
    aabbs: Query<&Aabb, Changed<Aabb>>,
    parents: Query<&Parent>,
    model_preview: Res<ModelPreviewCamera>,
    camera_transforms: Query<&mut Transform, With<Camera>>,
) {
}

impl FromWorld for ModelPreviewCamera {
    fn from_world(world: &mut World) -> Self {
        // camera
        let image_size = Extent3d {
            width: 320,
            height: 240,
            depth_or_array_layers: 1,
        };
        let mut preview_image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size: image_size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
            },
            ..default()
        };
        preview_image.resize(image_size);
        let mut images = world.get_resource_mut::<Assets<Image>>().unwrap();
        let preview_image = images.add(preview_image);
        let mut egui_context = world.get_resource_mut::<EguiContext>().unwrap();
        // Attach the bevy image to the egui image
        let egui_handle = egui_context.add_image(preview_image.clone());
        let camera_entity = world
            .spawn(Camera3dBundle {
                transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Z),
                camera: Camera {
                    target: RenderTarget::Image(preview_image),
                    ..default()
                },
                ..default()
            })
            .insert(RenderLayers::from_layers(&[MODEL_PREVIEW_LAYER]))
            .id();
        let model_entity = world
            .spawn(RenderLayers::from_layers(&[MODEL_PREVIEW_LAYER]))
            .id();

        Self {
            camera_entity,
            egui_handle,
            model_entity,
        }
    }
}

impl Plugin for ModelPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModelPreviewCamera>()
            .add_system(scale_preview_for_model_bounding_box);
    }
}
