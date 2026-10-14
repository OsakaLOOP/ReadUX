"""编译独立 GiNZA 推理解码器；编译结果保存在实验目录。"""
from pathlib import Path
import os
from setuptools import Extension, setup
from Cython.Build import cythonize
import numpy

ROOT = Path(__file__).resolve().parents[1]
os.chdir(ROOT)
setup(
    name="readux-ginza-decode",
    ext_modules=cythonize(
        [Extension("ginza_decode", ["scripts/ginza_decode.pyx"], language="c++", include_dirs=[numpy.get_include()],
                   extra_compile_args=["/O2", "/fp:strict"] if os.name == "nt"
                   else ["-O3", "-ffp-contract=off"])],
        build_dir=".agents/analysis/ginza-build", compiler_directives={"language_level": 3},
    ),
    script_args=["build_ext", "--build-lib", "experiments/ginza-optimized",
                 "--build-temp", ".agents/analysis/ginza-build/temp"],
)
