# test branch

This branch is used for **unstable experiments with new features**. Most test features will be experimented with and archived separately on this branch.

## Notes

- Experimental code may be rewritten, rolled back, or discarded at any time.
- Experimental code is not guaranteed to run stably. Some content may be abandoned after only an initial attempt, but will still be archived.
- All experimental code is not guaranteed to be merged into the actual project even after it becomes stable.
- The code on this branch is not part of the actual project code.

## Experimental Branch Environment and Development Requirements

1. All experimental projects on this branch share the same Rust project environment.
2. The Rust project environment configuration on this branch should remain consistent with the development branch at all times.
3. Experimental projects should remain independent of one another.
4. All Rust experimental projects should be placed in the `experiments/` directory, with subdirectories used to keep experimental projects isolated.
5. Experimental mappings in other languages should be placed in the `other/[code_language]/` directory. The remaining environment requirements, experimental isolation, and other such requirements are consistent with the four points above.

---

# test 分支

本分支用于**新功能的不稳定实验**，多数测试功能会在此分支单独实验和留档。

## 说明

- 实验代码可能随时被重写、回退或丢弃。
- 实验代码并不保证可以稳定运行，部分内容可能仅在初次尝试后就被弃用，但仍会留档。
- 所有实验性代码并不保证在稳定后一定会并入实际工程当中。
- 该分支代码不属于实际工程代码的一部分。

## 实验分支环境与开发要求

1. 该分支所有实验项目共用同一个 Rust 项目环境。
2. 该分支 Rust 项目环境配置应当与开发分支时刻保持一致。
3. 实验项目之间应保持相互独立。
4. Rust 实验项目应全部放在 `experiments/` 目录且用子级目录保持实验项目隔离。
5. 其他语言映射实验应当放在 `other/[code_language]/` 目录下，其余环境要求、实验隔离等与上述四条内容一致。