# Система модинга

## Структура

```
game/
├── mods/
│   └── example_mod/
│       └── init.lua
```

## API для модов

### Game.register_block(block_table)

Регистрирует новый блок в игре.

**Параметры:**
- `id` (string) - уникальный идентификатор блока (например: "mymod:stone")
- `name` (string) - отображаемое имя блока
- `texture` (string) - путь к текстуре блока
- `solid` (bool, optional) - является ли блок твердым (по умолчанию: true)
- `transparent` (bool, optional) - является ли блок прозрачным (по умолчанию: false)

**Пример:**
```lua
-- Initialize mod
Game.init({
    name = "My Awesome Mod",
    version = "1.0.0",
    author = "YourName"
})

-- Register blocks
Game.register_block({
    id = "mymod:ruby_block",
    name = "Ruby Block",
    texture = "assets/textures/block/ruby.png",
    solid = true,
    transparent = false
})
```

## Создание мода

1. Создайте папку в `game/mods/` с именем вашего мода
2. Создайте файл `init.lua` в этой папке
3. Используйте API для регистрации блоков
4. Запустите игру - моды загрузятся автоматически

## Использование блоков из модов

После регистрации блоки можно использовать по их ID:
```rust
world.place_block(engine, pos, "mymod:ruby_block");
```
