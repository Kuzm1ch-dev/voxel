#[derive(Debug, Clone, PartialEq)]
pub struct BlockType {
    pub id: String,
    pub textures: BlockTextures,
}

impl BlockType {
    pub fn new(id: String, textures: BlockTextures) -> Self {
        BlockType {
            id,
            textures,
        }
    }
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
    pub fn new(top: String, bottom: String, front: String, back: String, left: String, right: String) -> Self {
        BlockTextures {
            top,
            bottom,
            front,
            back,
            left,
            right,
        }
    }
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