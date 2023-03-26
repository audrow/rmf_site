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
    interaction::{
        ChangeMode, HeadlightToggle, Hover, MoveTo, PickingBlockers, Select, SpawnPreview,
    },
    occupancy::CalculateGrid,
    recency::ChangeRank,
    site::{
        AssociatedGraphs, Change, ConsiderAssociatedGraph, ConsiderLocationTag, CurrentLevel,
        CurrentSite, Delete, ExportLights, FloorVisibility, PhysicalLightToggle, SaveNavGraphs,
        SiteState, ToggleLiftDoorAvailability,
    },
};
use bevy::{ecs::system::SystemParam, prelude::*, window::PrimaryWindow};
use bevy_egui::{
    egui::{self, CollapsingHeader},
    EguiContext,
};
use rmf_site_format::*;

pub mod create;
use create::CreateWidget;

pub mod view_layers;
use view_layers::*;

pub mod view_levels;
use view_levels::{LevelDisplay, LevelParams, ViewLevels};

pub mod view_lights;
use view_lights::*;

pub mod view_nav_graphs;
use view_nav_graphs::*;

pub mod view_occupancy;
use view_occupancy::*;

pub mod icons;
pub use icons::*;

pub mod inspector;
use inspector::{InspectorParams, InspectorWidget};

pub mod move_layer;
pub use move_layer::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub enum UiUpdateLabel {
    DrawUi,
}

#[derive(Default)]
pub struct StandardUiLayout;

impl Plugin for StandardUiLayout {
    fn build(&self, app: &mut App) {
        app.init_resource::<Icons>()
            .init_resource::<LevelDisplay>()
            .init_resource::<NavGraphDisplay>()
            .init_resource::<LightDisplay>()
            .init_resource::<OccupancyDisplay>()
            .add_system_set(SystemSet::on_enter(SiteState::Display).with_system(init_ui_style))
            .add_system_set(
                SystemSet::on_update(SiteState::Display)
                    .with_system(standard_ui_layout.label(UiUpdateLabel::DrawUi)),
            )
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::on_update(SiteState::Display)
                    .with_system(resolve_light_export_file)
                    .with_system(resolve_nav_graph_import_export_files),
            );
    }
}

#[derive(SystemParam)]
pub struct ChangeEvents<'w> {
    pub lane_motion: EventWriter<'w, Change<Motion>>,
    pub lane_reverse: EventWriter<'w, Change<ReverseLane>>,
    pub name: EventWriter<'w, Change<NameInSite>>,
    pub label: EventWriter<'w, Change<Label>>,
    pub pose: EventWriter<'w, Change<Pose>>,
    pub door: EventWriter<'w, Change<DoorType>>,
    pub lift_cabin: EventWriter<'w, Change<LiftCabin<Entity>>>,
    pub asset_source: EventWriter<'w, Change<AssetSource>>,
    pub pixels_per_meter: EventWriter<'w, Change<PixelsPerMeter>>,
    pub physical_camera_properties: EventWriter<'w, Change<PhysicalCameraProperties>>,
    pub light: EventWriter<'w, Change<LightKind>>,
    pub level_props: EventWriter<'w, Change<LevelProperties>>,
    pub color: EventWriter<'w, Change<DisplayColor>>,
    pub visibility: EventWriter<'w, Change<Visibility>>,
    pub associated_graphs: EventWriter<'w, Change<AssociatedGraphs<Entity>>>,
    pub location_tags: EventWriter<'w, Change<LocationTags>>,
}

#[derive(SystemParam)]
pub struct PanelResources<'w> {
    pub level: ResMut<'w, LevelDisplay>,
    pub nav_graph: ResMut<'w, NavGraphDisplay>,
    pub light: ResMut<'w, LightDisplay>,
    pub occupancy: ResMut<'w, OccupancyDisplay>,
}

#[derive(SystemParam)]
pub struct Requests<'w> {
    pub hover: ResMut<'w, Events<Hover>>,
    pub select: ResMut<'w, Events<Select>>,
    pub move_to: EventWriter<'w, MoveTo>,
    pub current_level: ResMut<'w, CurrentLevel>,
    pub current_site: ResMut<'w, CurrentSite>,
    pub change_mode: ResMut<'w, Events<ChangeMode>>,
    pub delete: EventWriter<'w, Delete>,
    pub toggle_door_levels: EventWriter<'w, ToggleLiftDoorAvailability>,
    pub toggle_headlights: ResMut<'w, HeadlightToggle>,
    pub toggle_physical_lights: ResMut<'w, PhysicalLightToggle>,
    pub spawn_preview: EventWriter<'w, SpawnPreview>,
    pub export_lights: EventWriter<'w, ExportLights>,
    pub save_nav_graphs: EventWriter<'w, SaveNavGraphs>,
    pub calculate_grid: EventWriter<'w, CalculateGrid>,
    pub consider_tag: EventWriter<'w, ConsiderLocationTag>,
    pub consider_graph: EventWriter<'w, ConsiderAssociatedGraph>,
}

#[derive(SystemParam)]
pub struct LayerEvents<'w> {
    pub floors: EventWriter<'w, ChangeRank<FloorMarker>>,
    pub drawings: EventWriter<'w, ChangeRank<DrawingMarker>>,
    pub nav_graphs: EventWriter<'w, ChangeRank<NavGraphMarker>>,
    pub change_floor_vis: EventWriter<'w, Change<FloorVisibility>>,
    pub global_floor_vis: ResMut<'w, FloorVisibility>,
}

/// We collect all the events into its own SystemParam because we are not
/// allowed to receive more than one EventWriter of a given type per system call
/// (for borrow-checker reasons). Bundling them all up into an AppEvents
/// parameter at least makes the EventWriters easy to pass around.
#[derive(SystemParam)]
pub struct AppEvents<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub change: ChangeEvents<'w>,
    pub display: PanelResources<'w>,
    pub request: Requests<'w>,
    pub layers: LayerEvents<'w>,
}

fn standard_ui_layout(
    mut egui_context: Query<&mut EguiContext, With<PrimaryWindow>>,
    mut picking_blocker: Option<ResMut<PickingBlockers>>,
    inspector_params: InspectorParams,
    levels: LevelParams,
    lights: LightParams,
    nav_graphs: NavGraphParams,
    layers: LayersParams,
    mut events: AppEvents,
) {
    egui::SidePanel::right("right_panel")
        .resizable(true)
        .show(egui_context.single_mut().get_mut(), |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        CollapsingHeader::new("Levels")
                            .default_open(true)
                            .show(ui, |ui| {
                                ViewLevels::new(&levels, &mut events).show(ui);
                            });
                        ui.separator();
                        CollapsingHeader::new("Navigation Graphs")
                            .default_open(true)
                            .show(ui, |ui| {
                                ViewNavGraphs::new(&nav_graphs, &mut events).show(ui);
                            });
                        ui.separator();
                        // TODO(MXG): Consider combining Nav Graphs and Layers
                        CollapsingHeader::new("Layers")
                            .default_open(false)
                            .show(ui, |ui| {
                                ViewLayers::new(&layers, &mut events).show(ui);
                            });
                        ui.separator();
                        CollapsingHeader::new("Inspect")
                            .default_open(true)
                            .show(ui, |ui| {
                                InspectorWidget::new(&inspector_params, &mut events).show(ui);
                            });
                        ui.separator();
                        CollapsingHeader::new("Create")
                            .default_open(false)
                            .show(ui, |ui| {
                                CreateWidget::new(&mut events).show(ui);
                            });
                        ui.separator();
                        CollapsingHeader::new("Lights")
                            .default_open(false)
                            .show(ui, |ui| {
                                ViewLights::new(&lights, &mut events).show(ui);
                            });
                        ui.separator();
                        CollapsingHeader::new("Occupancy")
                            .default_open(false)
                            .show(ui, |ui| {
                                ViewOccupancy::new(&mut events).show(ui);
                            });
                    });
                });
        });

    let egui_context = egui_context.single_mut().get_mut();
    let ui_has_focus = egui_context.wants_pointer_input()
        || egui_context.wants_keyboard_input()
        || egui_context.is_pointer_over_area();

    if let Some(picking_blocker) = &mut picking_blocker {
        picking_blocker.ui = ui_has_focus;
    }

    if ui_has_focus {
        // If the UI has focus and there were no hover events emitted by the UI,
        // then we should emit a None hover event
        if events.request.hover.is_empty() {
            events.request.hover.send(Hover(None));
        }
    }
}

fn init_ui_style(mut egui_context: Query<&mut EguiContext, With<PrimaryWindow>>) {
    // I think the default egui dark mode text color is too dim, so this changes
    // it to a brighter white.
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(egui::Color32::from_rgb(250, 250, 250));
    egui_context.single_mut().get_mut().set_visuals(visuals);
}
