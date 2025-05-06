# levenshtein_distance

Репозиторий посвящен алгоритму Левенштейна на языке Rust при использовании compute shader'ов для реализации вычислений на видеокарте. 

---

### Stack проекта состоит из:
* Spir-V
* wgpu
* Языка Rust и др. его библиотек

Используется версия cargo 1.71.0-nightly (64fb38c97 2023-05-23) см. rust-toolchain.toml

---

### Полезные ссылки для погружения в контекст:
* Первичный гайд для знакомства с [rust-gpu](https://rust-gpu.github.io/rust-gpu/book/platform-support.html)
* Репозиторий, посвященный [rust-gpu](https://github.com/Rust-GPU/rust-gpu?tab=readme-ov-file)
* Пример работы wgpu с Spir-V [hello-compute](https://github.com/gfx-rs/wgpu-rs/blob/master/examples/hello-compute/main.rs)
* [Расстояние Левенштейна](https://ru.wikipedia.org/wiki/Расстояние_Левенштейна#Формула)
* [WebGpu fundamentals](https://webgpufundamentals.org)
* Документация [WebGpu](https://www.w3.org/TR/webgpu/)

---
### Запуск и компиляция шейдеров

Проект представляет собой workspace, шейдеры, написанные для Spir-V используют аттрибут #![no_std] и являются динамическими библиотеками, [dylib] в cargo.toml крейта. 

В нынешнем состоянии в проекте находятся два компилируемых шейдера Spir-V: levenshtein_shader и my_shader, второй представляет из себя пример hello-compute для Spir-V.

В build.rs находятся инструкции для компиляции файла Spir-V, шейдер создается после команды cargo build, после файл с расширением .spv появляется в проекте по адресу: target/spirv-builder/<таргет>/release/deps/<название шейдера>. Для компиляции название шейдера подставляется в SpirvBuilder::new("<название шейдера>", "<таргет>"), второй аргумент представляет собой target и по умолчанию выставлен как: spirv-unknown-spv1.4

Запуск шейдера происходит в main.rs. Константа SHADER содержит скомпилированный код Spir-V в байтах, он выглядит как: include_bytes!(env!("<имя шейдера.spv>")).