# Deployment

This docoment is a deployment report for the project. It contains the deployment process, the tools used, and the deployment environment. Since I am using Ubuntu as my primary operating system, this document mainly focuses on the deployment process on Linux, but I will provide some links for Windows and Mac users as well if you are interested in deploying the project on those operating systems.

## Deployment Process

### Step 1: Install Rust

Let's start off by installing cargo. Cargo is the Rust package manager and it is used to build and run Rust projects. You can install cargo by running the following command (for other operating systems, you can check the installation guide [here](https://www.rust-lang.org/tools/install)):

```bash
curl https://sh.rustup.rs -sSf | sh
```

It will download and install the latest version of Rust and Cargo. After installing cargo, you can check the version of cargo by running the following command:

```bash
cargo --version
```

### Step 2: Install OpenCV

Next, we need to install OpenCV. OpenCV is an open-source computer vision and machine learning software library. You can install OpenCV by running the following command (for other operating systems, you can check the installation guide [here](https://github.com/twistedfall/opencv-rust/blob/master/INSTALL.md)):

```bash
sudo apt-get install libopencv-dev clang libclang-dev
```

### Step 3: Install SDL2

Next, we need to install SDL2. SDL2 is a cross-platform development library designed to provide low-level access to audio, keyboard, mouse, joystick, and graphics hardware via OpenGL and Direct3D. You can install SDL2 by running the following command.

```bash
sudo apt install libasound2-dev libudev-dev
```

### Step 4: Install dependencies

Now, we need to install the dependencies for the project. You can install the dependencies by running the following command:

```bash
cd computational_art_and_design
cargo build
```

### Step 5: Run the project

Finally, you can run the project by running the following command:

```bash
cargo run -- file data/test_staing_midle.mp4 data/test_walking_LR_moving.mp4
```
