-- Initialize mod
ModLoader.init({
    name = "Example Mod",
    version = "0.0.1",
    author = "Kuzm1ch88"
})

-- Register blocks
ModLoader.register_block({
    id = "example:ruby_block",
    name = "Ruby Block",
    texture = "assets/texture/block/ruby.png",
    solid = true,
    transparent = false
})

print("Example Mod loaded!")
