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

use crate::*;
#[cfg(feature = "bevy")]
use bevy::prelude::{Bundle, Component, Entity};
use serde::{Deserialize, Serialize};

/// Bundle used to spawn and move whole workcells in site editor mode
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "bevy", derive(Bundle))]
pub struct Workcell {
    /// Used in site editor to assign a unique name
    pub name: NameInSite,
    /// Pose of the workcell once spawned in site editor
    pub pose: Pose,
    // TODO(luca) add source, might need asset loader specialization
    // since workcells might be saved as ron files
    // pub source: AssetSource,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "bevy", derive(Bundle))]
pub struct WorkcellElement {
    /// Unique name to identify the element
    pub name: NameInSite,
    /// Workcell elements are normal meshes, point to where the mesh is stored
    pub source: AssetSource,
    /// Workcell element poses are defined relative to anchors
    pub pose_constraint: WorkcellPoseConstraint,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub struct WorkcellPoseConstraint {
    /// Anchor entity the pose is relative to
    pub relative_to: Entity,
    /// Relative transform
    // TODO(luca) change this to Transform instead?
    pub pose: Pose,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub struct AnchorPoseConstraint {
    /// Model entity origin the pose is relative to
    pub relative_to: Entity,
    /// Relative transform
    // TODO(luca) change this to Transform instead?
    pub pose: Pose,
    // TODO(luca) implement options such as the one below to specify transforms
    // relative to mesh elements such as faces, edges, vertices
    // pub vertex_id: Option<u32>,
}
