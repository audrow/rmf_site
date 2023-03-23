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

use bevy::prelude::*;
use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::utils::BoxedFuture;
use bevy::reflect::TypeUuid;

use urdf_rs::Robot;
use thiserror::Error;

pub struct UrdfPlugin;

impl Plugin for UrdfPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<UrdfLoader>()
            .add_asset::<UrdfRoot>();
            //.init_asset_loader::<XacroLoader>();
    }
}

#[derive(Default)]
struct UrdfLoader;

#[derive(Default)]
struct XacroLoader;

impl AssetLoader for UrdfLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move { Ok(load_urdf(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["urdf"];
        EXTENSIONS
    }
}

impl AssetLoader for XacroLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move { Ok(load_xacro(bytes, load_context).await?) })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["xacro"];
        EXTENSIONS
    }
}

#[derive(Error, Debug)]
pub enum UrdfError {
    #[error("Failed to load Urdf")]
    ParsingError,
    //Io(#[from] std::io::Error),
}

#[derive(Component, Clone, Debug, Deref, DerefMut, TypeUuid)]
#[uuid = "fe707f9e-c6f3-11ed-afa1-0242ac120002"]
pub struct UrdfRoot(Robot);

async fn load_urdf<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), UrdfError> {
    if let Some(res) = std::str::from_utf8(bytes).ok().and_then(|utf| urdf_rs::read_from_string(utf).ok()) {
        load_context.set_default_asset(LoadedAsset::new(UrdfRoot(res)));
        return Ok(());
    } else {
        return Err(UrdfError::ParsingError);
    }
}

// TODO(luca) write to a tempfile then call the urdf-rs xacro utility to get the urdf
async fn load_xacro<'a, 'b>(
    bytes: &'a [u8],
    load_context: &'a mut LoadContext<'b>,
) -> Result<(), UrdfError> {
    return Err(UrdfError::ParsingError);
}
