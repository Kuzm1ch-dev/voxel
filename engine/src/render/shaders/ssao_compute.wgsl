// Глобальные ресурсы
@group(0) @binding(0) var depthTexture: texture_depth_2d; // Глубинная текстура
@group(0) @binding(1) var ssaoTexture: texture_storage_2d<rgba16float, write>;

// Ядро SSAO (массив из 64 векторов)
var<private> kernel: array<vec3f, 64>;

// Функция для генерации случайного ядра
fn generateKernel() {
    for (var i = 0u; i < 64; i = i + 1u) {
        // Создаем случайные векторы для ядра
        let randomVec = fract(sin(vec4(f32(i) * 17.0, f32(i) * 13.0, f32(i) * 19.0, 0.0)) * 43758.5453);
        kernel[i] = vec3(randomVec.xy * 2.0 - 1.0, 0.1); // Нормализуем и добавляем Z-компоненту
    }
}
// Основной вычислительный шейдер
@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    // Проверяем границы текстуры
    if (GlobalInvocationID.x >= 1920u || GlobalInvocationID.y >= 1080u) {
        return;
    }

    // Вычисляем UV координаты
    let uv = vec2(f32(GlobalInvocationID.x) / 1920.0, f32(GlobalInvocationID.y) / 1080.0);

    // Читаем значение глубины из глубинной текстуры
    let depth = textureLoad(depthTexture, vec2<i32>(GlobalInvocationID.xy), 0);

    // Инициализируем переменную для окклюзии
    var occlusion: f32 = 0.0;

    // Генерируем ядро SSAO
    generateKernel();

    // Проходим по всем векторам ядра
    for (var i = 0u; i < 64; i = i + 1u) {
        // Вычисляем смещение для текущего вектора ядра
        let samplePos = uv + kernel[i].xy * 0.01;

        // Преобразуем смещенные UV координаты в пиксельные координаты
        let sampleCoord = vec2<i32>(samplePos * vec2(1920.0, 1080.0));

        // Читаем значение глубины из соседнего пикселя
        let sampleDepth = textureLoad(depthTexture, sampleCoord, 0);

        // Вычисляем окклюзию на основе разницы глубин
        occlusion += step(0.01, depth - sampleDepth);
    }

    // Нормализуем окклюзию
    occlusion = 1.0 - (occlusion / f32(64));

    // Записываем результат в SSAO-текстуру
    textureStore(ssaoTexture, vec2<i32>(GlobalInvocationID.xy), vec4(occlusion, occlusion, occlusion, 1.0));
}