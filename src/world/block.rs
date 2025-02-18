#[derive(Debug, Clone, PartialEq)]
pub struct BlockType {
    pub id: String,
    pub textures: BlockTextures,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockTextures {
    pub top: String,    // texture name for top face
    pub bottom: String, // texture name for bottom face
    pub front: String,  // texture name for front face
    pub back: String,   // texture name for back face
    pub left: String,   // texture name for left face
    pub right: String,  // texture name for right face
}

impl BlockTextures {
    pub fn uniform(texture: String) -> Self {
        // Helper for blocks that use the same texture on all faces
        BlockTextures {
            top: texture.clone(),
            bottom: texture.clone(),
            front: texture.clone(),
            back: texture.clone(),
            left: texture.clone(),
            right: texture,
        }
    }
}