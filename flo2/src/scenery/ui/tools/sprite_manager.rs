use flo_scene::*;
use flo_scene::programs::*;
use flo_draw::canvas::*;

use futures::prelude::*;

use serde::*;

///
/// Sprite manager requests
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SpriteManager {
    /// Assign a sprite and send the assigned sprite to the specified target
    RequestSprite(StreamTarget),

    /// Indicates that a sprite is finished with and can be reallocated to another program
    ReturnSprite(SpriteId),
}

///
/// Response from the sprite manager indicating an assigned sprite ID
///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssignedSprite(pub SpriteId);

impl SceneMessage for SpriteManager {
    fn default_target() -> StreamTarget {
        SubProgramId::called("flowbetween::sprite_manager").into()
    }

    fn initialise(init_context: &impl SceneInitialisationContext) {
        init_context.add_subprogram(SubProgramId::called("flowbetween::sprite_manager"), sprite_manager_subprogram, 20);
        init_context.connect_programs(FilterHandle::for_filter(|stream: InputStream<Query<AssignedSprite>>| stream.map(|msg| SpriteManager::RequestSprite(msg.target()))), SubProgramId::called("flowbetween::sprite_manager"), StreamId::with_message_type::<Query<AssignedSprite>>()).unwrap();
    }
}

impl SceneMessage for AssignedSprite {
    fn initialise(init_context: &impl SceneInitialisationContext) {
        init_context.connect_programs(FilterHandle::for_filter(|stream: InputStream<Query<AssignedSprite>>| stream.map(|msg| SpriteManager::RequestSprite(msg.target()))), SubProgramId::called("flowbetween::sprite_manager"), StreamId::with_message_type::<Query<AssignedSprite>>()).unwrap();
    }
}

///
/// The sprite manager subprogram is used to assign unique sprite IDs. It's used for things like the tool palette programs
/// which have to ensure they don't re-use each other's sprite IDs
///
pub async fn sprite_manager_subprogram(input: InputStream<SpriteManager>, context: SceneContext) {
    // Sprite IDs that are not assigned yet
    let mut unused_sprites = vec![];

    // The next ID to assign a sprite
    let mut next_id = 0;

    let mut input = input;
    while let Some(msg) = input.next().await {
        match msg {
            SpriteManager::RequestSprite(target) => {
                let Ok(mut target) = context.send(target) else { continue };

                if let Some(sprite_id) = unused_sprites.pop() {
                    // Re-use a sprite that has been returned
                    target.send(QueryResponse::with_data(AssignedSprite(sprite_id))).await.ok();
                } else {
                    // Assign a new sprite
                    let sprite_id = SpriteId(next_id);
                    next_id += 1;
                    target.send(QueryResponse::with_data(AssignedSprite(sprite_id))).await.ok();
                }
            }

            SpriteManager::ReturnSprite(sprite_id) => {
                // Robustness: recover from a situation where a sprite is returned multiple times by not allowing it in the unused list more than once
                if !unused_sprites.contains(&sprite_id) {
                    unused_sprites.push(sprite_id);
                }
            }
        }
    }
}
