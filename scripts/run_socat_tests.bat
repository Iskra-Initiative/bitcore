@echo off
REM -- script to run socat-based integration tests on Windows

echo bitcore socat integration tests
echo ===============================

REM check if socat is installed
where socat >nul 2>nul
if %errorlevel% neq 0 (
    echo error: socat is not installed or not in PATH
    echo.
    echo to install socat on Windows:
    echo   1. install msys2 from https://www.msys2.org/
    echo   2. run: pacman -S socat
    echo   3. add msys2 bin directory to PATH
    echo.
    echo alternatively, use WSL with Ubuntu and install socat there
    echo.
    exit /b 1
)

echo ✓ socat found
echo.

echo running socat integration tests...
echo note: these tests create virtual serial port pairs using socat
echo.

REM run the socat tests
cargo test --test socat_tests -- --ignored --nocapture

echo.
echo socat integration tests completed!
