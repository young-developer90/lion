@echo off
echo ============================================
echo   Zamin vs Python Benchmarks
echo ============================================
echo.

echo.
echo --- Running Zamin benchmark ---
echo.
cargo run --bin zamin -- run benchmarks/bench_zamin.zamin 2>&1

echo.
echo.
echo --- Running Python benchmark ---
echo.
python benchmarks/bench_python.py
