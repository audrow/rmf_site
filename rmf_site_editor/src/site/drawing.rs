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

use crate::{
    interaction::Selectable,
    shapes::make_flat_rect_mesh,
    site::{
        get_current_workspace_path, Category, DefaultFile, FloorVisibility, RecencyRank,
        FLOOR_LAYER_START,
    },
    CurrentWorkspace,
};
use bevy::{math::Affine3A, prelude::*, utils::HashMap};
use rmf_site_format::{AssetSource, DrawingMarker, PixelsPerMeter, Pose};

pub const DRAWING_LAYER_START: f32 = 0.0;

#[derive(Debug, Clone, Copy, Component)]
pub struct DrawingSegments {
    leaf: Entity,
}

// We need to keep track of the drawing data until the image is loaded
// since we will need to scale the mesh according to the size of the image
#[derive(Default, Resource)]
pub struct LoadingDrawings(pub HashMap<Handle<Image>, (Entity, Pose, PixelsPerMeter)>);

fn drawing_layer_height(rank: Option<&RecencyRank<DrawingMarker>>) -> f32 {
    rank.map(|r| r.proportion() * (FLOOR_LAYER_START - DRAWING_LAYER_START) + DRAWING_LAYER_START)
        .unwrap_or(DRAWING_LAYER_START)
}

pub fn add_drawing_visuals(
    new_drawings: Query<(Entity, &AssetSource, &Pose, &PixelsPerMeter), Added<DrawingMarker>>,
    asset_server: Res<AssetServer>,
    mut loading_drawings: ResMut<LoadingDrawings>,
    current_workspace: Res<CurrentWorkspace>,
    site_files: Query<&DefaultFile>,
    mut default_floor_vis: ResMut<FloorVisibility>,
) {
    let file_path = match get_current_workspace_path(current_workspace, site_files) {
        Some(file_path) => file_path,
        None => return,
    };
    for (e, source, pose, pixels_per_meter) in &new_drawings {
        // Append file name to path if it's a local file
        // TODO(luca) cleanup
        let asset_source = match source {
            AssetSource::Local(name) => AssetSource::Local(String::from(
                file_path.with_file_name(name).to_str().unwrap(),
            )),
            _ => source.clone(),
        };
        let texture_handle: Handle<Image> = asset_server.load(&String::from(&asset_source));
        loading_drawings
            .0
            .insert(texture_handle, (e, pose.clone(), pixels_per_meter.clone()));
    }

    if !new_drawings.is_empty() {
        *default_floor_vis = FloorVisibility::new_semi_transparent();
    }
}

// Asset event handler for loaded drawings
pub fn handle_loaded_drawing(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Image>>,
    assets: Res<Assets<Image>>,
    mut loading_drawings: ResMut<LoadingDrawings>,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rank: Query<&RecencyRank<DrawingMarker>>,
    mut segments: Query<(&DrawingSegments, &mut Transform)>,
    mut mesh_handles: Query<&mut Handle<Mesh>>,
) {
    for ev in ev_asset.iter() {
        if let AssetEvent::Created { handle } = ev {
            if let Some((entity, pose, pixels_per_meter)) = loading_drawings.0.remove(handle) {
                let img = assets.get(handle).unwrap();
                let width = img.texture_descriptor.size.width as f32;
                let height = img.texture_descriptor.size.height as f32;

                // We set this up so that the origin of the drawing is in
                let mesh = make_flat_rect_mesh(width, height).transform_by(
                    Affine3A::from_translation(Vec3::new(width / 2.0, -height / 2.0, 0.0)),
                );
                let mesh = mesh_assets.add(mesh.into());
                let pose = pose.clone();
                let transform = pose.transform().with_scale(Vec3::new(
                    1.0 / pixels_per_meter.0,
                    1.0 / pixels_per_meter.0,
                    1.,
                ));

                if let Ok((segment, mut tf)) = segments.get_mut(entity) {
                    *tf = transform;
                    if let Ok(mut mesh_handle) = mesh_handles.get_mut(segment.leaf) {
                        *mesh_handle = mesh;
                    } else {
                        println!("DEV ERROR: Partially-constructed Drawing entity detected");
                    }
                    // We can ignore the layer height here since that update
                    // will be handled by another system.
                } else {
                    let z = drawing_layer_height(rank.get(entity).ok());
                    let mut cmd = commands.entity(entity);
                    let leaf = cmd.add_children(|p| {
                        p.spawn(PbrBundle {
                            mesh,
                            material: materials.add(StandardMaterial {
                                base_color_texture: Some(handle.clone()),
                                ..default()
                            }),
                            transform: Transform::from_xyz(0.0, 0.0, z),
                            ..default()
                        })
                        .id()
                    });

                    cmd.insert(SpatialBundle {
                        transform,
                        ..default()
                    })
                    .insert(DrawingSegments { leaf })
                    .insert(Selectable::new(entity))
                    .insert(Category::Drawing);
                }
            }
        }
    }
}

pub fn update_drawing_visuals(
    changed_drawings: Query<(Entity, &AssetSource, &Pose, &PixelsPerMeter), Changed<AssetSource>>,
    asset_server: Res<AssetServer>,
    mut loading_drawings: ResMut<LoadingDrawings>,
    current_workspace: Res<CurrentWorkspace>,
    site_files: Query<&DefaultFile>,
) {
    let file_path = match get_current_workspace_path(current_workspace, site_files) {
        Some(file_path) => file_path,
        None => return,
    };
    for (e, source, pose, pixels_per_meter) in &changed_drawings {
        let asset_source = match source {
            AssetSource::Local(name) => AssetSource::Local(String::from(
                file_path.with_file_name(name).to_str().unwrap(),
            )),
            _ => source.clone(),
        };
        let texture_handle: Handle<Image> = asset_server.load(&String::from(&asset_source));
        loading_drawings
            .0
            .insert(texture_handle, (e, pose.clone(), pixels_per_meter.clone()));
    }
}

pub fn update_drawing_rank(
    changed_rank: Query<
        (&DrawingSegments, &RecencyRank<DrawingMarker>),
        Changed<RecencyRank<DrawingMarker>>,
    >,
    mut transforms: Query<&mut Transform>,
) {
    for (segments, rank) in &changed_rank {
        if let Ok(mut tf) = transforms.get_mut(segments.leaf) {
            let z = drawing_layer_height(Some(rank));
            tf.translation.z = z;
        }
    }
}

pub fn update_drawing_pixels_per_meter(
    mut changed_drawings: Query<(&mut Transform, &PixelsPerMeter), Changed<PixelsPerMeter>>,
) {
    for (mut tf, pixels_per_meter) in &mut changed_drawings {
        tf.scale = Vec3::new(1.0 / pixels_per_meter.0, 1.0 / pixels_per_meter.0, 1.);
    }
}
